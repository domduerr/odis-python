use pyo3::prelude::*;
use pyo3::types::{PyList, PySlice};
use bit_set::BitSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::labelset::LabelSet;
use crate::errors;

// ---------------------------------------------------------------------------
// ConceptCollection — eagerly computed, indexed, sliceable
// ---------------------------------------------------------------------------

#[pyclass]
pub struct ConceptCollection {
    pub(crate) data: Vec<(BitSet, BitSet)>,
    pub(crate) objects: Arc<Vec<String>>,
    pub(crate) attributes: Arc<Vec<String>>,
}

#[pymethods]
impl ConceptCollection {
    fn __len__(&self) -> usize {
        self.data.len()
    }

    fn __getitem__(&self, py: Python<'_>, idx: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        if let Ok(i) = idx.extract::<isize>() {
            let len = self.data.len() as isize;
            let i = if i < 0 { len + i } else { i };
            if i < 0 || i >= len {
                return Err(pyo3::exceptions::PyIndexError::new_err("index out of range"));
            }
            let concept = self.make_concept(i as usize);
            return Ok(Py::new(py, concept)?.into_py(py));
        }
        if let Ok(sl) = idx.downcast::<PySlice>() {
            let indices = sl.indices(self.data.len() as isize)?;
            let mut out: Vec<PyObject> = Vec::new();
            let mut i = indices.start;
            while if indices.step > 0 { i < indices.stop } else { i > indices.stop } {
                let concept = self.make_concept(i as usize);
                out.push(Py::new(py, concept)?.into_py(py));
                i += indices.step;
            }
            return Ok(PyList::new_bound(py, &out).into_py(py));
        }
        Err(pyo3::exceptions::PyTypeError::new_err("indices must be integers or slices"))
    }

    fn __iter__(&self) -> ConceptIterator {
        ConceptIterator {
            data: self.data.clone(),
            pos: 0,
            objects: Arc::clone(&self.objects),
            attributes: Arc::clone(&self.attributes),
        }
    }

    fn __repr__(&self) -> String {
        format!("ConceptCollection({} concepts)", self.data.len())
    }

    fn __eq__(&self, other: &ConceptCollection) -> bool {
        self.data == other.data
    }

    fn sorted(&self) -> Self {
        let n_attrs = self.attributes.len();
        let mut sorted_data = self.data.clone();
        sorted_data.sort_by(|(_, a), (_, b)| {
            for i in 0..n_attrs {
                let in_a = a.contains(i);
                let in_b = b.contains(i);
                if !in_a && in_b {
                    return std::cmp::Ordering::Less;
                }
                if in_a && !in_b {
                    return std::cmp::Ordering::Greater;
                }
            }
            std::cmp::Ordering::Equal
        });
        ConceptCollection {
            data: sorted_data,
            objects: Arc::clone(&self.objects),
            attributes: Arc::clone(&self.attributes),
        }
    }

    fn to_python(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        self.data
            .iter()
            .map(|(ext_bits, int_bits)| {
                let ext: Vec<&str> = ext_bits.iter().map(|i| self.objects[i].as_str()).collect();
                let int_: Vec<&str> = int_bits.iter().map(|i| self.attributes[i].as_str()).collect();
                let pair = (
                    pyo3::types::PyFrozenSet::new_bound(py, &ext)?,
                    pyo3::types::PyFrozenSet::new_bound(py, &int_)?,
                );
                Ok(pair.into_py(py))
            })
            .collect()
    }
}

impl ConceptCollection {
    pub fn make_concept(&self, i: usize) -> Concept {
        Concept {
            extent: LabelSet::new(self.data[i].0.clone(), Arc::clone(&self.objects)),
            intent: LabelSet::new(self.data[i].1.clone(), Arc::clone(&self.attributes)),
        }
    }
}

// ---------------------------------------------------------------------------
// ConceptIterator
// ---------------------------------------------------------------------------

#[pyclass]
pub struct ConceptIterator {
    pub(crate) data: Vec<(BitSet, BitSet)>,
    pub(crate) pos: usize,
    pub(crate) objects: Arc<Vec<String>>,
    pub(crate) attributes: Arc<Vec<String>>,
}

#[pymethods]
impl ConceptIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<Concept> {
        if self.pos >= self.data.len() {
            return None;
        }
        let concept = Concept {
            extent: LabelSet::new(self.data[self.pos].0.clone(), Arc::clone(&self.objects)),
            intent: LabelSet::new(self.data[self.pos].1.clone(), Arc::clone(&self.attributes)),
        };
        self.pos += 1;
        Some(concept)
    }
}

// ---------------------------------------------------------------------------
// ConceptGenerator — lazy iterator that blocks mutations while alive
// ---------------------------------------------------------------------------

#[pyclass]
pub struct ConceptGenerator {
    pub(crate) data: Vec<(BitSet, BitSet)>,
    pub(crate) pos: usize,
    pub(crate) objects: Arc<Vec<String>>,
    pub(crate) attributes: Arc<Vec<String>>,
    pub(crate) mutation_gen: Arc<AtomicU64>,
    pub(crate) gen_at_creation: u64,
    pub(crate) active_lazy_counter: Arc<AtomicU32>,
}

#[pymethods]
impl ConceptGenerator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> PyResult<Option<Concept>> {
        let current_gen = self.mutation_gen.load(Ordering::SeqCst);
        if current_gen != self.gen_at_creation {
            return Err(errors::mutation_during_lazy(
                self.active_lazy_counter.load(Ordering::SeqCst),
            ));
        }
        if self.pos >= self.data.len() {
            return Ok(None);
        }
        let concept = Concept {
            extent: LabelSet::new(self.data[self.pos].0.clone(), Arc::clone(&self.objects)),
            intent: LabelSet::new(self.data[self.pos].1.clone(), Arc::clone(&self.attributes)),
        };
        self.pos += 1;
        Ok(Some(concept))
    }
}

impl Drop for ConceptGenerator {
    fn drop(&mut self) {
        self.active_lazy_counter.fetch_sub(1, Ordering::SeqCst);
    }
}

// ---------------------------------------------------------------------------
// Concept — (extent: LabelSet, intent: LabelSet)
// ---------------------------------------------------------------------------

#[pyclass]
#[derive(Clone)]
pub struct Concept {
    pub(crate) extent: LabelSet,
    pub(crate) intent: LabelSet,
}

#[pymethods]
impl Concept {
    #[getter]
    fn extent(&self) -> LabelSet {
        self.extent.clone()
    }

    #[getter]
    fn intent(&self) -> LabelSet {
        self.intent.clone()
    }

    fn __iter__(&self) -> ConceptPairIterator {
        ConceptPairIterator {
            items: [Some(self.extent.clone()), Some(self.intent.clone())],
            pos: 0,
        }
    }

    fn __len__(&self) -> usize {
        2
    }

    fn __getitem__(&self, idx: isize) -> PyResult<LabelSet> {
        match idx {
            0 | -2 => Ok(self.extent.clone()),
            1 | -1 => Ok(self.intent.clone()),
            _ => Err(pyo3::exceptions::PyIndexError::new_err("index out of range")),
        }
    }

    fn __contains__(&self, label: &str) -> bool {
        self.extent.bits.iter().any(|i| self.extent.labels[i] == label)
            || self.intent.bits.iter().any(|i| self.intent.labels[i] == label)
    }

    fn __repr__(&self) -> String {
        let ext: Vec<&str> = self.extent.bits.iter().map(|i| self.extent.labels[i].as_str()).collect();
        let int_: Vec<&str> = self.intent.bits.iter().map(|i| self.intent.labels[i].as_str()).collect();
        format!("Concept(extent={:?}, intent={:?})", ext, int_)
    }

    fn __eq__(&self, other: &Concept) -> bool {
        self.extent.bits == other.extent.bits && self.intent.bits == other.intent.bits
    }

    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        let mut ext_labels: Vec<&str> = self.extent.bits.iter().map(|i| self.extent.labels[i].as_str()).collect();
        let mut int_labels: Vec<&str> = self.intent.bits.iter().map(|i| self.intent.labels[i].as_str()).collect();
        ext_labels.sort();
        int_labels.sort();
        (ext_labels, int_labels).hash(&mut hasher);
        hasher.finish()
    }

    fn to_python(&self, py: Python<'_>) -> PyResult<PyObject> {
        let ext: Vec<&str> = self.extent.bits.iter().map(|i| self.extent.labels[i].as_str()).collect();
        let int_: Vec<&str> = self.intent.bits.iter().map(|i| self.intent.labels[i].as_str()).collect();
        let pair = (
            pyo3::types::PyFrozenSet::new_bound(py, &ext)?,
            pyo3::types::PyFrozenSet::new_bound(py, &int_)?,
        );
        Ok(pair.into_py(py))
    }
}

// ---------------------------------------------------------------------------
// ConceptPairIterator — yields the 2-element (extent, intent) sequence for Concept
// ---------------------------------------------------------------------------

#[pyclass]
pub struct ConceptPairIterator {
    pub(crate) items: [Option<LabelSet>; 2],
    pub(crate) pos: usize,
}

#[pymethods]
impl ConceptPairIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<LabelSet> {
        if self.pos >= 2 {
            return None;
        }
        let item = self.items[self.pos].take();
        self.pos += 1;
        item
    }
}
