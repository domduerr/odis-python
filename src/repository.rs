//! Python bindings for `odis::repository` — the FCA repository at <https://fcarepository.org/>.
//!
//! The core API is async, because on `wasm32` it can only go through the browser's
//! fetch API. Python is synchronous, so every entry point here drives the same future
//! to completion on a shared Tokio runtime, releasing the GIL while it waits.

use std::sync::OnceLock;

use pyo3::exceptions::{PyConnectionError, PyValueError};
use pyo3::prelude::*;
use tokio::runtime::{Builder, Runtime};

use odis::repository::{self, RepositoryError};
use odis::FormalContext;

use crate::context::PyFormalContext;
use crate::errors;

/// Runtime the downloads run on. Built once; a fresh one per call would rebuild the
/// TLS and IO machinery every time.
fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build the Tokio runtime for repository downloads")
    })
}

/// Maps a `RepositoryError` to the matching Python exception.
fn repo_err_to_py(err: RepositoryError) -> PyErr {
    match err {
        RepositoryError::InvalidCatalog => {
            PyValueError::new_err("The repository catalogue could not be parsed")
        }
        RepositoryError::Format(e) => errors::format_err_to_py(e),
        RepositoryError::Http(message) => {
            PyConnectionError::new_err(format!("Download from the FCA repository failed: {message}"))
        }
    }
}

/// Downloads a context, releasing the GIL for the duration of the request.
fn download_context(py: Python<'_>, filename: &str) -> PyResult<FormalContext<String>> {
    py.detach(|| runtime().block_on(repository::fetch_context(filename)))
        .map_err(repo_err_to_py)
}

/// One dataset in the FCA repository catalogue.
#[pyclass(name = "RepositoryEntry", module = "odis", frozen)]
pub struct PyRepositoryEntry {
    /// File name of the context, e.g. `livingbeings_en.cxt`.
    #[pyo3(get)]
    filename: String,
    /// Human-readable name of the dataset.
    #[pyo3(get)]
    title: String,
    /// Bibliographic references the context was published in.
    #[pyo3(get)]
    source: Vec<String>,
    /// Number of objects, as stated by the catalogue.
    #[pyo3(get)]
    objects: Option<usize>,
    /// Number of attributes, as stated by the catalogue.
    #[pyo3(get)]
    attributes: Option<usize>,
    /// Language the object and attribute names are written in.
    #[pyo3(get)]
    language: Option<String>,
    /// Short description of what the context is about.
    #[pyo3(get)]
    description: Option<String>,
    /// Further remarks, e.g. how this context relates to another one.
    #[pyo3(get)]
    note: Vec<String>,
}

#[pymethods]
impl PyRepositoryEntry {
    /// URL the context file is downloaded from.
    #[getter]
    fn url(&self) -> String {
        repository::context_url(&self.filename)
    }

    /// Downloads this context.
    fn load(&self, py: Python<'_>) -> PyResult<PyFormalContext> {
        Ok(PyFormalContext::wrap(download_context(py, &self.filename)?))
    }

    fn __repr__(&self) -> String {
        format!(
            "RepositoryEntry(filename={:?}, title={:?})",
            self.filename, self.title
        )
    }
}

/// Downloads the repository catalogue and returns one entry per available context.
#[pyfunction]
pub fn repository_catalog(py: Python<'_>) -> PyResult<Vec<PyRepositoryEntry>> {
    let entries = py
        .detach(|| runtime().block_on(repository::fetch_catalog()))
        .map_err(repo_err_to_py)?;

    Ok(entries
        .into_iter()
        .map(|e| PyRepositoryEntry {
            filename: e.filename,
            title: e.title,
            source: e.source,
            objects: e.objects,
            attributes: e.attributes,
            language: e.language,
            description: e.description,
            note: e.note,
        })
        .collect())
}

/// Backs `FormalContext.from_repository`.
pub fn from_repository(py: Python<'_>, filename: &str) -> PyResult<PyFormalContext> {
    Ok(PyFormalContext::wrap(download_context(py, filename)?))
}
