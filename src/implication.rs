use pyo3::prelude::*;
use pyo3::types::{PyList, PySlice};
use bit_set::BitSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::labelset::LabelSet;
use crate::errors;

// ---------------------------------------------------------------------------
// ImplicationList — eagerly collected Vec<(BitSet,BitSet)>
// ---------------------------------------------------------------------------

#[pyclass]
pub struct ImplicationList {
    pub(crate) data: Vec<(BitSet, BitSet)>,
    pub(crate) attributes: Arc<Vec<String>>,
}

#[pymethods]
impl ImplicationList {
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
            let imp = self.make_implication(i as usize);
            return Ok(Py::new(py, imp)?.into_py(py));
        }
        if let Ok(sl) = idx.downcast::<PySlice>() {
            let indices = sl.indices(self.data.len() as isize)?;
            let mut out: Vec<PyObject> = Vec::new();
            let mut i = indices.start;
            while if indices.step > 0 { i < indices.stop } else { i > indices.stop } {
                let imp = self.make_implication(i as usize);
                out.push(Py::new(py, imp)?.into_py(py));
                i += indices.step;
            }
            return Ok(PyList::new_bound(py, &out).into_py(py));
        }
        Err(pyo3::exceptions::PyTypeError::new_err("indices must be integers or slices"))
    }

    fn __iter__(&self) -> ImplicationIterator {
        ImplicationIterator {
            data: self.data.clone(),
            pos: 0,
            attributes: Arc::clone(&self.attributes),
        }
    }

    fn __repr__(&self) -> String {
        format!("ImplicationList({} implications)", self.data.len())
    }

    fn __eq__(&self, other: &ImplicationList) -> bool {
        self.data == other.data
    }

    fn to_python(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
        self.data
            .iter()
            .map(|(prem_bits, conc_bits)| {
                let prem: Vec<&str> = prem_bits.iter().map(|i| self.attributes[i].as_str()).collect();
                let conc: Vec<&str> = conc_bits.iter().map(|i| self.attributes[i].as_str()).collect();
                let pair = (
                    pyo3::types::PyFrozenSet::new_bound(py, &prem)?,
                    pyo3::types::PyFrozenSet::new_bound(py, &conc)?,
                );
                Ok(pair.into_py(py))
            })
            .collect()
    }
}

impl ImplicationList {
    pub fn make_implication(&self, i: usize) -> Implication {
        Implication {
            premise: LabelSet::new(self.data[i].0.clone(), Arc::clone(&self.attributes)),
            conclusion: LabelSet::new(self.data[i].1.clone(), Arc::clone(&self.attributes)),
        }
    }
}

// ---------------------------------------------------------------------------
// ImplicationIterator
// ---------------------------------------------------------------------------

#[pyclass]
pub struct ImplicationIterator {
    pub(crate) data: Vec<(BitSet, BitSet)>,
    pub(crate) pos: usize,
    pub(crate) attributes: Arc<Vec<String>>,
}

#[pymethods]
impl ImplicationIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<Implication> {
        if self.pos >= self.data.len() {
            return None;
        }
        let imp = Implication {
            premise: LabelSet::new(self.data[self.pos].0.clone(), Arc::clone(&self.attributes)),
            conclusion: LabelSet::new(self.data[self.pos].1.clone(), Arc::clone(&self.attributes)),
        };
        self.pos += 1;
        Some(imp)
    }
}

// ---------------------------------------------------------------------------
// ImplicationGenerator — lazy, blocks mutations while alive
// ---------------------------------------------------------------------------

#[pyclass]
pub struct ImplicationGenerator {
    pub(crate) data: Vec<(BitSet, BitSet)>,
    pub(crate) pos: usize,
    pub(crate) attributes: Arc<Vec<String>>,
    pub(crate) mutation_gen: Arc<AtomicU64>,
    pub(crate) gen_at_creation: u64,
    pub(crate) active_lazy_counter: Arc<AtomicU32>,
}

#[pymethods]
impl ImplicationGenerator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> PyResult<Option<Implication>> {
        let current_gen = self.mutation_gen.load(Ordering::SeqCst);
        if current_gen != self.gen_at_creation {
            return Err(errors::mutation_during_lazy(
                self.active_lazy_counter.load(Ordering::SeqCst),
            ));
        }
        if self.pos >= self.data.len() {
            return Ok(None);
        }
        let imp = Implication {
            premise: LabelSet::new(self.data[self.pos].0.clone(), Arc::clone(&self.attributes)),
            conclusion: LabelSet::new(self.data[self.pos].1.clone(), Arc::clone(&self.attributes)),
        };
        self.pos += 1;
        Ok(Some(imp))
    }
}

impl Drop for ImplicationGenerator {
    fn drop(&mut self) {
        self.active_lazy_counter.fetch_sub(1, Ordering::SeqCst);
    }
}

// ---------------------------------------------------------------------------
// Implication — (premise: LabelSet, conclusion: LabelSet)
// ---------------------------------------------------------------------------

#[pyclass]
#[derive(Clone)]
pub struct Implication {
    pub(crate) premise: LabelSet,
    pub(crate) conclusion: LabelSet,
}

#[pymethods]
impl Implication {
    #[getter]
    fn premise(&self) -> LabelSet {
        self.premise.clone()
    }

    #[getter]
    fn conclusion(&self) -> LabelSet {
        self.conclusion.clone()
    }

    fn __iter__(&self) -> ImplicationPairIterator {
        ImplicationPairIterator {
            items: [Some(self.premise.clone()), Some(self.conclusion.clone())],
            pos: 0,
        }
    }

    fn __len__(&self) -> usize {
        2
    }

    fn __getitem__(&self, idx: isize) -> PyResult<LabelSet> {
        match idx {
            0 | -2 => Ok(self.premise.clone()),
            1 | -1 => Ok(self.conclusion.clone()),
            _ => Err(pyo3::exceptions::PyIndexError::new_err("index out of range")),
        }
    }

    fn __contains__(&self, label: &str) -> bool {
        self.premise.bits.iter().any(|i| self.premise.labels[i] == label)
            || self.conclusion.bits.iter().any(|i| self.conclusion.labels[i] == label)
    }

    fn __repr__(&self) -> String {
        let prem: Vec<&str> = self.premise.bits.iter().map(|i| self.premise.labels[i].as_str()).collect();
        let conc: Vec<&str> = self.conclusion.bits.iter().map(|i| self.conclusion.labels[i].as_str()).collect();
        format!("Implication(premise={:?}, conclusion={:?})", prem, conc)
    }

    fn __eq__(&self, other: &Implication) -> bool {
        self.premise.bits == other.premise.bits && self.conclusion.bits == other.conclusion.bits
    }

    fn to_python(&self, py: Python<'_>) -> PyResult<PyObject> {
        let prem: Vec<&str> = self.premise.bits.iter().map(|i| self.premise.labels[i].as_str()).collect();
        let conc: Vec<&str> = self.conclusion.bits.iter().map(|i| self.conclusion.labels[i].as_str()).collect();
        let pair = (
            pyo3::types::PyFrozenSet::new_bound(py, &prem)?,
            pyo3::types::PyFrozenSet::new_bound(py, &conc)?,
        );
        Ok(pair.into_py(py))
    }
}

// ---------------------------------------------------------------------------
// ImplicationPairIterator — yields the 2-element (premise, conclusion) sequence
// ---------------------------------------------------------------------------

#[pyclass]
pub struct ImplicationPairIterator {
    pub(crate) items: [Option<LabelSet>; 2],
    pub(crate) pos: usize,
}

#[pymethods]
impl ImplicationPairIterator {
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
