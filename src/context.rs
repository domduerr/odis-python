use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use bit_set::BitSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use odis::FormalContext;

use crate::concept::{ConceptCollection, ConceptGenerator};
use crate::implication::{Implication, ImplicationGenerator, ImplicationList};
use crate::errors;
use crate::labelset::LabelSet;

/// Python-facing wrapper around `FormalContext<String>`.
///
/// Holds Arc snapshots of the label vectors so that existing `ConceptCollection`,
/// `ImplicationList`, and `LabelSet` wrappers remain valid after mutation.
#[pyclass(name = "FormalContext")]
pub struct PyFormalContext {
    pub(crate) inner: FormalContext<String>,
    pub(crate) arc_objects: Arc<Vec<String>>,
    pub(crate) arc_attributes: Arc<Vec<String>>,
    /// Incremented on every mutation. Live generators detect staleness via this.
    pub(crate) mutation_gen: Arc<AtomicU64>,
    /// Count of live lazy generators. Mutations are blocked when this is > 0.
    pub(crate) active_lazy: Arc<AtomicU32>,
}

impl PyFormalContext {
    /// Wraps a freshly constructed `FormalContext<String>`.
    pub fn wrap(inner: FormalContext<String>) -> Self {
        let arc_objects = Arc::new(inner.objects.clone());
        let arc_attributes = Arc::new(inner.attributes.clone());
        PyFormalContext {
            inner,
            arc_objects,
            arc_attributes,
            mutation_gen: Arc::new(AtomicU64::new(0)),
            active_lazy: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Refresh Arc snapshots and increment `mutation_gen` after a mutation.
    pub(crate) fn refresh_arcs(&mut self) {
        self.arc_objects = Arc::new(self.inner.objects.clone());
        self.arc_attributes = Arc::new(self.inner.attributes.clone());
        self.mutation_gen.fetch_add(1, Ordering::SeqCst);
    }

    /// Check mutation guard; return Err if lazy generators are active.
    pub(crate) fn guard_mutation(&self) -> PyResult<()> {
        let n = self.active_lazy.load(Ordering::SeqCst);
        if n > 0 {
            // Increment gen so existing generators see invalidation
            self.mutation_gen.fetch_add(1, Ordering::SeqCst);
            Err(errors::mutation_during_lazy(n))
        } else {
            Ok(())
        }
    }

    /// Lookup object index by name.
    fn obj_idx(&self, name: &str) -> PyResult<usize> {
        self.inner.objects.iter().position(|o| o == name)
            .ok_or_else(|| errors::unknown_object(name))
    }

    /// Lookup attribute index by name.
    fn attr_idx(&self, name: &str) -> PyResult<usize> {
        self.inner.attributes.iter().position(|a| a == name)
            .ok_or_else(|| errors::unknown_attribute(name))
    }

    /// Convert a Python set/LabelSet into an object-index BitSet (silently drops unknowns).
    fn names_to_obj_bits<'py>(&self, names: Bound<'py, PyAny>) -> PyResult<BitSet> {
        let mut bits = BitSet::new();
        if let Ok(ls) = names.extract::<LabelSet>() {
            for name in ls.bits.iter().map(|i| ls.labels[i].as_str()) {
                if let Some(idx) = self.inner.objects.iter().position(|o| o == name) {
                    bits.insert(idx);
                }
            }
        } else {
            for item in names.iter()? {
                let s: String = item?.extract()?;
                if let Some(idx) = self.inner.objects.iter().position(|o| o == &s) {
                    bits.insert(idx);
                }
            }
        }
        Ok(bits)
    }

    /// Convert a Python set/LabelSet of attribute names into attribute-index BitSet.
    fn names_to_attr_bits_any<'py>(&self, names: Bound<'py, PyAny>) -> PyResult<BitSet> {
        let mut bits = BitSet::new();
        if let Ok(ls) = names.extract::<LabelSet>() {
            for name in ls.bits.iter().map(|i| ls.labels[i].as_str()) {
                if let Some(idx) = self.inner.attributes.iter().position(|a| a == name) {
                    bits.insert(idx);
                }
            }
        } else {
            for item in names.iter()? {
                let s: String = item?.extract()?;
                if let Some(idx) = self.inner.attributes.iter().position(|a| a == &s) {
                    bits.insert(idx);
                }
            }
        }
        Ok(bits)
    }
}

#[pymethods]
impl PyFormalContext {
    // ---- Name ----

    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[setter]
    fn set_name(&mut self, name: String) {
        self.inner.name = name;
    }

    // ---- Construction ----

    #[new]
    fn new_empty() -> Self {
        PyFormalContext::wrap(FormalContext::new())
    }

    #[staticmethod]
    fn from_file(path: &str) -> PyResult<Self> {
        let bytes = std::fs::read(path).map_err(errors::io_err_to_py)?;
        let inner = FormalContext::<String>::from(&bytes).map_err(errors::format_err_to_py)?;
        Ok(PyFormalContext::wrap(inner))
    }

    #[staticmethod]
    fn from_dict<'py>(mapping: Bound<'py, PyDict>) -> PyResult<Self> {
        let mut inner = FormalContext::<String>::new();
        let mut seen_attrs: Vec<String> = Vec::new();
        let mut obj_attr_names: Vec<(String, Vec<String>)> = Vec::new();

        for (k, v) in mapping.iter() {
            let obj_name: String = k.extract()?;
            let mut attrs: Vec<String> = Vec::new();
            for a in v.iter()? {
                let attr: String = a?.extract()?;
                if !seen_attrs.contains(&attr) {
                    seen_attrs.push(attr.clone());
                }
                attrs.push(attr);
            }
            obj_attr_names.push((obj_name, attrs));
        }

        for attr in &seen_attrs {
            let bits = BitSet::new();
            inner.add_attribute(attr.clone(), &bits);
        }

        for (obj_name, attrs) in obj_attr_names {
            let mut bits = BitSet::new();
            for a in &attrs {
                if let Some(idx) = inner.attributes.iter().position(|x| x == a) {
                    bits.insert(idx);
                }
            }
            inner.add_object(obj_name, &bits);
        }

        Ok(PyFormalContext::wrap(inner))
    }

    // ---- Introspection ----

    #[getter]
    fn objects<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        Ok(PyList::new_bound(py, self.inner.objects.iter().map(|s| s.as_str())))
    }

    #[getter]
    fn attributes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyList>> {
        Ok(PyList::new_bound(py, self.inner.attributes.iter().map(|s| s.as_str())))
    }

    #[getter]
    fn shape(&self) -> (usize, usize) {
        (self.inner.objects.len(), self.inner.attributes.len())
    }

    fn __len__(&self) -> usize {
        self.inner.objects.len()
    }

    fn __contains__(&self, obj: &str) -> bool {
        self.inner.objects.contains(&obj.to_string())
    }

    fn __repr__(&self) -> String {
        format!(
            "FormalContext(objects={}, attributes={}, objects={:?}, attributes={:?})",
            self.inner.objects.len(),
            self.inner.attributes.len(),
            self.inner.objects,
            self.inner.attributes,
        )
    }

    // ---- Incidence access ----

    fn __getitem__(&self, key: (String, String)) -> PyResult<bool> {
        let g = self.obj_idx(&key.0)?;
        let m = self.attr_idx(&key.1)?;
        Ok(self.inner.incidence.contains(&(g, m)))
    }

    fn __setitem__(&mut self, key: (String, String), value: bool) -> PyResult<()> {
        self.guard_mutation()?;
        let g = self.obj_idx(&key.0)?;
        let m = self.attr_idx(&key.1)?;
        self.inner.set_cross(g, m, value);
        self.refresh_arcs();
        Ok(())
    }

    // ---- Serialization ----

    fn to_file(&self, path: &str) -> PyResult<()> {
        self.inner.to_file(path).map_err(errors::io_err_to_py)
    }

    // ---- Copy ----

    fn copy(&self) -> Self {
        PyFormalContext {
            inner: self.inner.clone(),
            arc_objects: Arc::new(self.inner.objects.clone()),
            arc_attributes: Arc::new(self.inner.attributes.clone()),
            mutation_gen: Arc::new(AtomicU64::new(0)),
            active_lazy: Arc::new(AtomicU32::new(0)),
        }
    }

    // ---- Mutation methods ----

    #[pyo3(signature = (name, attributes=None))]
    fn add_object<'py>(&mut self, name: String, attributes: Option<Bound<'py, PyAny>>) -> PyResult<()> {
        self.guard_mutation()?;
        if self.inner.objects.contains(&name) {
            return Err(errors::duplicate_object(&name));
        }
        // Build attribute bits; auto-create unknown attribute names
        let mut bits = BitSet::new();
        if let Some(attr_set) = attributes {
            for item in attr_set.iter()? {
                let attr: String = item?.extract()?;
                let idx = if let Some(i) = self.inner.attributes.iter().position(|a| a == &attr) {
                    i
                } else {
                    // Auto-create + refresh arcs within mutation is fine here
                    let new_attr_bits = BitSet::new();
                    self.inner.add_attribute(attr, &new_attr_bits);
                    self.inner.attributes.len() - 1
                };
                bits.insert(idx);
            }
        }
        self.inner.add_object(name, &bits);
        self.refresh_arcs();
        Ok(())
    }

    fn add_attribute(&mut self, name: String) -> PyResult<()> {
        self.guard_mutation()?;
        if self.inner.attributes.contains(&name) {
            return Err(errors::duplicate_attribute(&name));
        }
        let bits = BitSet::new();
        self.inner.add_attribute(name, &bits);
        self.refresh_arcs();
        Ok(())
    }

    fn remove_object(&mut self, name: String) -> PyResult<()> {
        self.guard_mutation()?;
        let idx = self.obj_idx(&name)?;
        self.inner.remove_object(idx);
        self.refresh_arcs();
        Ok(())
    }

    fn remove_attribute(&mut self, name: String) -> PyResult<()> {
        self.guard_mutation()?;
        let idx = self.attr_idx(&name)?;
        self.inner.remove_attribute(idx);
        self.refresh_arcs();
        Ok(())
    }

    fn rename_object(&mut self, old_name: String, new_name: String) -> PyResult<()> {
        self.guard_mutation()?;
        let idx = self.obj_idx(&old_name)?;
        if self.inner.objects.contains(&new_name) {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Object '{}' already exists", new_name
            )));
        }
        self.inner.change_object_name(new_name, idx);
        self.refresh_arcs();
        Ok(())
    }

    fn rename_attribute(&mut self, old_name: String, new_name: String) -> PyResult<()> {
        self.guard_mutation()?;
        let idx = self.attr_idx(&old_name)?;
        if self.inner.attributes.contains(&new_name) {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Attribute '{}' already exists", new_name
            )));
        }
        self.inner.change_attribute_name(new_name, idx);
        self.refresh_arcs();
        Ok(())
    }

    // ---- Algorithms ----

    #[pyo3(signature = (*, lazy=false))]
    fn concepts(&mut self, lazy: bool) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            if lazy {
                let data: Vec<(BitSet, BitSet)> = self.inner.index_concepts().collect();
                let gen = self.active_lazy.clone();
                gen.fetch_add(1, Ordering::SeqCst);
                let generator = ConceptGenerator {
                    data,
                    pos: 0,
                    objects: Arc::clone(&self.arc_objects),
                    attributes: Arc::clone(&self.arc_attributes),
                    mutation_gen: Arc::clone(&self.mutation_gen),
                    gen_at_creation: self.mutation_gen.load(Ordering::SeqCst),
                    active_lazy_counter: Arc::clone(&self.active_lazy),
                };
                Ok(Py::new(py, generator)?.into_py(py))
            } else {
                let data: Vec<(BitSet, BitSet)> = self.inner.index_concepts().collect();
                let coll = ConceptCollection {
                    data,
                    objects: Arc::clone(&self.arc_objects),
                    attributes: Arc::clone(&self.arc_attributes),
                };
                Ok(Py::new(py, coll)?.into_py(py))
            }
        })
    }

    fn extent<'py>(&self, attributes: Bound<'py, PyAny>) -> PyResult<LabelSet> {
        let bits = self.names_to_attr_bits_any(attributes)?;
        let result = self.inner.index_extent(&bits);
        Ok(LabelSet::new(result, Arc::clone(&self.arc_objects)))
    }

    fn intent<'py>(&self, objects: Bound<'py, PyAny>) -> PyResult<LabelSet> {
        let bits = self.names_to_obj_bits(objects)?;
        let result = self.inner.index_intent(&bits);
        Ok(LabelSet::new(result, Arc::clone(&self.arc_attributes)))
    }

    fn attribute_hull<'py>(&self, attributes: Bound<'py, PyAny>) -> PyResult<LabelSet> {
        let bits = self.names_to_attr_bits_any(attributes)?;
        let result = self.inner.index_attribute_hull(&bits);
        Ok(LabelSet::new(result, Arc::clone(&self.arc_attributes)))
    }

    fn object_hull<'py>(&self, objects: Bound<'py, PyAny>) -> PyResult<LabelSet> {
        let bits = self.names_to_obj_bits(objects)?;
        let result = self.inner.index_object_hull(&bits);
        Ok(LabelSet::new(result, Arc::clone(&self.arc_objects)))
    }

    fn upper_neighbor<'py>(&self, extent: Bound<'py, PyAny>) -> PyResult<LabelSet> {
        let bits = self.names_to_obj_bits(extent)?;
        let result = odis::algorithms::upper_neighbor::index_upper_neighbor(&bits, &self.inner);
        Ok(LabelSet::new(result, Arc::clone(&self.arc_objects)))
    }

    // ---- Canonical basis ----

    #[pyo3(signature = (*, lazy=false))]
    fn canonical_basis(&mut self, lazy: bool) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            if lazy {
                let data = self.inner.index_canonical_basis();
                let gen = self.active_lazy.clone();
                gen.fetch_add(1, Ordering::SeqCst);
                let generator = ImplicationGenerator {
                    data,
                    pos: 0,
                    attributes: Arc::clone(&self.arc_attributes),
                    mutation_gen: Arc::clone(&self.mutation_gen),
                    gen_at_creation: self.mutation_gen.load(Ordering::SeqCst),
                    active_lazy_counter: Arc::clone(&self.active_lazy),
                };
                Ok(Py::new(py, generator)?.into_py(py))
            } else {
                let data = self.inner.index_canonical_basis();
                let list = ImplicationList {
                    data,
                    attributes: Arc::clone(&self.arc_attributes),
                };
                Ok(Py::new(py, list)?.into_py(py))
            }
        })
    }

    #[pyo3(signature = (*, lazy=false))]
    fn canonical_basis_optimised(&mut self, lazy: bool) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            if lazy {
                let data = self.inner.index_canonical_basis_optimised();
                let gen = self.active_lazy.clone();
                gen.fetch_add(1, Ordering::SeqCst);
                let generator = ImplicationGenerator {
                    data,
                    pos: 0,
                    attributes: Arc::clone(&self.arc_attributes),
                    mutation_gen: Arc::clone(&self.mutation_gen),
                    gen_at_creation: self.mutation_gen.load(Ordering::SeqCst),
                    active_lazy_counter: Arc::clone(&self.active_lazy),
                };
                Ok(Py::new(py, generator)?.into_py(py))
            } else {
                let data = self.inner.index_canonical_basis_optimised();
                let list = ImplicationList {
                    data,
                    attributes: Arc::clone(&self.arc_attributes),
                };
                Ok(Py::new(py, list)?.into_py(py))
            }
        })
    }

    fn next_preclosure<'py>(
        &self,
        implications: Bound<'py, PyAny>,
        current: Bound<'py, PyAny>,
    ) -> PyResult<LabelSet> {
        // Extract implication data as Vec<(BitSet, BitSet)>
        let impl_data: Vec<(BitSet, BitSet)> =
            if let Ok(il) = implications.extract::<PyRef<ImplicationList>>() {
                il.data.clone()
            } else {
                // list of Implication objects
                let mut v = Vec::new();
                for item in implications.iter()? {
                    let imp = item?.extract::<PyRef<Implication>>()?;
                    v.push((imp.premise.bits.clone(), imp.conclusion.bits.clone()));
                }
                v
            };

        // Translate `current` (set[str] or LabelSet) to attribute-index BitSet
        let current_bits = self.names_to_attr_bits_any(current)?;

        let result = self.inner.index_next_preclosure(&impl_data, &current_bits);
        Ok(LabelSet::new(result, Arc::clone(&self.arc_attributes)))
    }

    // ---- Attribute exploration with Python callback ----

    fn attribute_exploration(&mut self, callback: PyObject) -> PyResult<ImplicationList> {
        use std::cell::RefCell;

        // Arc snapshots captured by the closure
        let arc_attributes = Arc::clone(&self.arc_attributes);
        // Capture snapshot of attribute names (for bit translation)  
        let attr_snap: Vec<String> = (*arc_attributes).clone();

        // Cell for any Python error that occurs inside the closure
        let error_cell: RefCell<Option<PyErr>> = RefCell::new(None);

        let result = self.inner.index_attribute_exploration_with_callback(
            |premise_bits: &BitSet, conclusion_bits: &BitSet| -> Option<(String, BitSet)> {
                // Short-circuit if we already have an error
                if error_cell.borrow().is_some() {
                    return None;
                }

                Python::with_gil(|py| {
                    // Build LabelSet args
                    let pls = LabelSet::new(premise_bits.clone(), Arc::clone(&arc_attributes));
                    let cls = LabelSet::new(conclusion_bits.clone(), Arc::clone(&arc_attributes));

                    let py_premise = match Py::new(py, pls) {
                        Ok(v) => v,
                        Err(e) => { *error_cell.borrow_mut() = Some(e); return None; }
                    };
                    let py_conclusion = match Py::new(py, cls) {
                        Ok(v) => v,
                        Err(e) => { *error_cell.borrow_mut() = Some(e); return None; }
                    };

                    let ret = match callback.call1(py, (py_premise, py_conclusion)) {
                        Ok(v) => v,
                        Err(e) => { *error_cell.borrow_mut() = Some(e); return None; }
                    };

                    // Try to extract as (str, iterable[str]) → reject + counterexample
                    if let Ok((name, attr_names)) =
                        ret.extract::<(String, std::collections::HashSet<String>)>(py)
                    {
                        let bits: BitSet = attr_names
                            .iter()
                            .filter_map(|a| attr_snap.iter().position(|x| x == a))
                            .collect();
                        return Some((name, bits));
                    }

                    // Anything else (True, None, …) → accept
                    None
                })
            },
        );

        // Re-raise any Python error that occurred inside the closure
        if let Some(err) = error_cell.into_inner() {
            return Err(err);
        }

        // Refresh arcs — exploration may have added objects and attributes
        self.refresh_arcs();

        Ok(ImplicationList {
            data: result,
            attributes: Arc::clone(&self.arc_attributes),
        })
    }

    // ---- Layout and drawing ----

    #[pyo3(signature = (algorithm = "dimdraw"))]
    fn draw(&self, py: Python<'_>, algorithm: &str) -> PyResult<Option<Py<crate::drawing::Drawing>>> {
        let drawing = crate::drawing::make_drawing(self, algorithm)?;
        match drawing {
            Some(d) => Ok(Some(Py::new(py, d)?)),
            None => Ok(None),
        }
    }

    #[pyo3(signature = (algorithm = "dimdraw", width = 800, height = 600))]
    fn draw_svg(&self, algorithm: &str, width: i64, height: i64) -> PyResult<String> {
        if width <= 0 || height <= 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "width and height must be positive integers",
            ));
        }
        match crate::drawing::make_drawing(self, algorithm)? {
            Some(d) => Ok(crate::drawing::render_svg_pub(&d, width as usize, height as usize)),
            None => Ok(format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\"></svg>"
            )),
        }
    }
}

