//! Getting a Python value to the text the core crate reads.
//!
//! Spreadsheet columns do not arrive as clean strings. A GTIN column read out
//! of an XLSX comes back as `float`, out of a database as `str` or `None`,
//! and out of a hand-written script as `int`. All four have to mean the same
//! thing, and exactly one of them is dangerous.

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyFloat, PyInt, PyString};

/// Render a Python value as the text barkod parses.
///
/// - `None` becomes `""`, which parses as *missing* rather than junk.
/// - `str` is passed through untouched, including its whitespace.
/// - `int` goes through Python's own `str()`, which is exact at any size.
///   Going via `f64` would silently corrupt anything past ~15 digits.
/// - `float` is rendered with its fractional part intact — `7318690123456.0`,
///   not `7318690123456` — so it takes the float-debris path and the result
///   records `dropped_fraction`. That flag is the only remaining trace that
///   the value arrived as a number, and the caller deserves to see it.
///
/// A float too large to render without an exponent comes out as
/// `3.7024510000000006e17`, which is refused as non-numeric. That is the
/// intended outcome: the precision was already lost before barkod saw it, and
/// refusing is better than returning a confident wrong number. The fix
/// belongs upstream — read the column as text.
pub(crate) fn as_text(value: &Bound<'_, PyAny>) -> PyResult<String> {
    if value.is_none() {
        return Ok(String::new());
    }
    if value.is_instance_of::<PyString>() {
        return value.extract::<String>();
    }
    if value.is_instance_of::<PyInt>() {
        // Python's str() on an int is exact however many digits it has.
        return value.str()?.extract::<String>();
    }
    if value.is_instance_of::<PyFloat>() {
        // Rust's Debug for f64 always writes a decimal point, where Display
        // drops it for integral values. Keeping it is what routes the value
        // through the float-debris rule instead of looking pristine.
        return Ok(format!("{:?}", value.extract::<f64>()?));
    }
    Err(PyTypeError::new_err(format!(
        "expected str, int, float or None, got {}",
        value.get_type().name()?
    )))
}
