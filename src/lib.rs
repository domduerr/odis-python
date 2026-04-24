mod errors;
mod labelset;
mod concept;
mod implication;
mod context;
mod drawing;
mod titanic;

use pyo3::prelude::*;

#[pymodule]
fn odis(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<context::PyFormalContext>()?;
    m.add_class::<concept::ConceptCollection>()?;
    m.add_class::<concept::Concept>()?;
    m.add_class::<concept::ConceptPairIterator>()?;
    m.add_class::<concept::ConceptIterator>()?;
    m.add_class::<concept::ConceptGenerator>()?;
    m.add_class::<implication::ImplicationList>()?;
    m.add_class::<implication::Implication>()?;
    m.add_class::<implication::ImplicationPairIterator>()?;
    m.add_class::<implication::ImplicationIterator>()?;
    m.add_class::<implication::ImplicationGenerator>()?;
    m.add_class::<labelset::LabelSet>()?;
    m.add_class::<labelset::LabelSetIterator>()?;
    m.add_class::<titanic::PyTitanic>()?;
    m.add_class::<drawing::Drawing>()?;
    m.add_class::<drawing::DrawingNode>()?;
    m.add_class::<drawing::PyPoset>()?;
    Ok(())
}
