use pyo3::prelude::*;
use pyo3::types::PyFrozenSet;
use bit_set::BitSet;
use std::sync::Arc;

/// A Rust-backed set of string labels.
///
/// Holds a cloned BitSet and a reference-counted snapshot of the originating context's
/// label vector, ensuring label stability even after the parent context is mutated.
#[pyclass]
#[derive(Clone)]
pub struct LabelSet {
    pub(crate) bits: BitSet,
    pub(crate) labels: Arc<Vec<String>>,
}

impl LabelSet {
    pub fn new(bits: BitSet, labels: Arc<Vec<String>>) -> Self {
        LabelSet { bits, labels }
    }
}

#[pymethods]
impl LabelSet {
    fn __iter__(slf: PyRef<'_, Self>) -> LabelSetIterator {
        let strings: Vec<String> = slf.bits.iter().map(|i| slf.labels[i].clone()).collect();
        LabelSetIterator { data: strings, pos: 0 }
    }

    fn __contains__(&self, name: &str) -> bool {
        self.labels.iter().position(|l| l == name)
            .map(|idx| self.bits.contains(idx))
            .unwrap_or(false)
    }

    fn __len__(&self) -> usize {
        self.bits.len()
    }

    fn __repr__(&self) -> String {
        let labels: Vec<&str> = self.bits.iter().map(|i| self.labels[i].as_str()).collect();
        format!("LabelSet({{{}}})", labels.join(", "))
    }

    fn __eq__(&self, other: &LabelSet) -> bool {
        // Value equality: same string labels, independent of originating context
        if self.bits.len() != other.bits.len() {
            return false;
        }
        // Compare by iterating labels — handles different label vectors with same content
        let self_labels: std::collections::HashSet<&str> =
            self.bits.iter().map(|i| self.labels[i].as_str()).collect();
        let other_labels: std::collections::HashSet<&str> =
            other.bits.iter().map(|i| other.labels[i].as_str()).collect();
        self_labels == other_labels
    }

    fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        // Hash over sorted label list for determinism
        let mut sorted: Vec<&str> = self.bits.iter().map(|i| self.labels[i].as_str()).collect();
        sorted.sort_unstable();
        let mut hasher = DefaultHasher::new();
        sorted.hash(&mut hasher);
        hasher.finish()
    }

    fn to_frozenset<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyFrozenSet>> {
        let labels: Vec<String> = self.bits.iter().map(|i| self.labels[i].clone()).collect();
        PyFrozenSet::new_bound(py, &labels)
    }
}

#[pyclass]
pub struct LabelSetIterator {
    data: Vec<String>,
    pos: usize,
}

#[pymethods]
impl LabelSetIterator {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<String> {
        if self.pos < self.data.len() {
            let s = self.data[self.pos].clone();
            self.pos += 1;
            Some(s)
        } else {
            None
        }
    }
}
