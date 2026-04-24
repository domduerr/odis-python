use pyo3::exceptions::{PyFileNotFoundError, PyKeyError, PyOSError, PyRuntimeError, PyValueError};
use pyo3::PyErr;
use std::io;
use odis::FormatError;

/// Map a Rust `io::Error` (from file operations) to the appropriate Python exception.
pub fn io_err_to_py(err: io::Error) -> PyErr {
    match err.kind() {
        io::ErrorKind::NotFound => {
            PyFileNotFoundError::new_err(format!("File not found: {}", err))
        }
        _ => PyOSError::new_err(format!("I/O error: {}", err)),
    }
}

/// Map an `odis` `FormatError` (from `.cxt` parsing) to the appropriate Python exception.
pub fn format_err_to_py(err: FormatError) -> PyErr {
    match err {
        FormatError::IoError(e) => io_err_to_py(e),
        FormatError::ParseError(e) => {
            PyValueError::new_err(format!("Failed to parse context file: {}", e))
        }
        FormatError::InvalidFormat => {
            PyValueError::new_err("Context file has invalid Burmeister (.cxt) format")
        }
    }
}

/// Raise `KeyError` for an unknown object name.
pub fn unknown_object(name: &str) -> PyErr {
    PyKeyError::new_err(format!("Unknown object: '{}'", name))
}

/// Raise `KeyError` for an unknown attribute name.
pub fn unknown_attribute(name: &str) -> PyErr {
    PyKeyError::new_err(format!("Unknown attribute: '{}'", name))
}

/// Raise `ValueError` for a duplicate object name.
pub fn duplicate_object(name: &str) -> PyErr {
    PyValueError::new_err(format!("Object '{}' already exists", name))
}

/// Raise `ValueError` for a duplicate attribute name.
pub fn duplicate_attribute(name: &str) -> PyErr {
    PyValueError::new_err(format!("Attribute '{}' already exists", name))
}

/// Raise `RuntimeError` when a mutation is attempted while lazy generators are active.
pub fn mutation_during_lazy(n: u32) -> PyErr {
    PyRuntimeError::new_err(format!(
        "Cannot mutate FormalContext while lazy generators are active: {} generator(s) still live",
        n
    ))
}
