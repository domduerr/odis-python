use pyo3::prelude::*;
use std::sync::atomic::Ordering;

use crate::concept::{ConceptCollection, ConceptGenerator};
use crate::context::PyFormalContext;

/// Python binding for the TITANIC iceberg-lattice algorithm.
#[pyclass(name = "Titanic")]
pub struct PyTitanic;

#[pymethods]
impl PyTitanic {
    #[new]
    pub fn new() -> Self {
        PyTitanic
    }

    #[pyo3(signature = (ctx, min_support, *, lazy=false))]
    fn enumerate(
        &self,
        py: Python<'_>,
        ctx: &mut PyFormalContext,
        min_support: u32,
        lazy: bool,
    ) -> PyResult<PyObject> {
        use odis::algorithms::Titanic;
        use odis::IcebergConceptEnumerator;

        let lattice = Titanic.enumerate(&ctx.inner, min_support);
        let data: Vec<_> = lattice
            .poset
            .nodes
            .into_iter()
            .map(|(e, i)| (e, i))
            .collect();

        if lazy {
            let gen = ctx.active_lazy.clone();
            gen.fetch_add(1, Ordering::SeqCst);
            let generator = ConceptGenerator {
                data,
                pos: 0,
                objects: std::sync::Arc::clone(&ctx.arc_objects),
                attributes: std::sync::Arc::clone(&ctx.arc_attributes),
                mutation_gen: std::sync::Arc::clone(&ctx.mutation_gen),
                gen_at_creation: ctx.mutation_gen.load(Ordering::SeqCst),
                active_lazy_counter: std::sync::Arc::clone(&ctx.active_lazy),
            };
            Ok(Py::new(py, generator)?.into_py(py))
        } else {
            let coll = ConceptCollection {
                data,
                objects: std::sync::Arc::clone(&ctx.arc_objects),
                attributes: std::sync::Arc::clone(&ctx.arc_attributes),
            };
            Ok(Py::new(py, coll)?.into_py(py))
        }
    }
}
