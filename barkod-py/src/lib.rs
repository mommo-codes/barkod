//! Python bindings for barkod.
//!
//! Thin by design: every answer comes from the core crate, and nothing here
//! decides anything about GTINs. The one job this layer does have is carrying
//! the [`GtinKey`](barkod::GtinKey) type distinction across the boundary,
//! because a binding that returned a bare `str` would throw away the guard
//! that stops a match key being exported as a GTIN.

use barkod::{Allocation, Encoding, Indicator, Reason};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyList;

mod convert;
use convert::as_text;

/// A GTIN in canonical GTIN-14 form.
#[pyclass(name = "Gtin14", frozen, module = "barkod")]
#[derive(Clone)]
pub struct PyGtin14(barkod::Gtin14);

#[pymethods]
impl PyGtin14 {
    /// The canonical form: fourteen digits, check digit preserved.
    #[getter]
    fn value(&self) -> &str {
        self.0.as_str()
    }

    /// The shortest standard form — GTIN-8, -12, -13 or -14.
    #[getter]
    fn shortest(&self) -> &str {
        self.0.shortest()
    }

    /// Which standard encoding `shortest` occupies: `"GTIN-8"`, `"GTIN-12"`,
    /// `"GTIN-13"` or `"GTIN-14"`.
    #[getter]
    fn encoding(&self) -> String {
        encoding_name(self.0.encoding()).to_string()
    }

    /// Digits that are not leading zeros.
    #[getter]
    fn significant_digits(&self) -> usize {
        self.0.significant_digits()
    }

    /// The check digit the value carries.
    #[getter]
    fn check_digit(&self) -> u8 {
        self.0.check_digit()
    }

    /// The check digit GS1 Mod-10 says it should carry.
    #[getter]
    fn expected_check_digit(&self) -> u8 {
        self.0.expected_check_digit()
    }

    /// Whether the check digit is correct. A classification, not a verdict:
    /// variable-weight in-store codes fail this on purpose.
    #[getter]
    fn check_digit_is_valid(&self) -> bool {
        self.0.check_digit_is_valid()
    }

    /// `"variable_measure"`, `"grouping"`, or `None` for a padded value that
    /// has no indicator digit at all.
    #[getter]
    fn indicator(&self) -> Option<&'static str> {
        self.0.indicator().map(|i| {
            if matches!(i, Indicator::VariableMeasure) {
                "variable_measure"
            } else {
                "grouping"
            }
        })
    }

    /// The indicator digit itself, or `None`.
    #[getter]
    fn indicator_digit(&self) -> Option<u8> {
        self.0.indicator().map(|i| match i {
            Indicator::Grouping(digit) => digit,
            // VariableMeasure is the digit 9 by definition.
            _ => 9,
        })
    }

    /// What GS1 allocated this range for, as a stable snake_case name.
    #[getter]
    fn allocation(&self) -> &'static str {
        allocation_name(self.0.allocation())
    }

    /// GS1's own wording for the range.
    #[getter]
    fn allocation_description(&self) -> &'static str {
        self.0.allocation().description()
    }

    /// Whether the range is restricted-circulation — in-store or regional.
    #[getter]
    fn is_restricted_circulation(&self) -> bool {
        self.0.allocation().is_restricted_circulation()
    }

    /// Whether the range identifies a publication or coupon rather than a
    /// trade item.
    #[getter]
    fn is_publication_or_coupon(&self) -> bool {
        self.0.allocation().is_publication_or_coupon()
    }

    /// Fewer significant digits than GTIN-8 can express, so never issued by
    /// GS1. A guess at "internal PLU", named as one.
    #[getter]
    fn looks_like_internal_code(&self) -> bool {
        self.0.looks_like_internal_code()
    }

    /// Restricted-circulation, ends in five zeros, check digit stale. A guess
    /// at "variable weight", named as one.
    #[getter]
    fn measure_field_looks_zeroed(&self) -> bool {
        self.0.measure_field_looks_zeroed()
    }

    /// The match key. Returns a `GtinKey`, never a `str` — see that class.
    fn key(&self) -> PyGtinKey {
        PyGtinKey(self.0.key())
    }

    /// The same first thirteen digits with a correct check digit appended.
    ///
    /// Export use only. This changes which product the code identifies
    /// whenever the original check digit was not a mistake.
    fn with_recomputed_check_digit(&self) -> PyGtin14 {
        PyGtin14(self.0.with_recomputed_check_digit())
    }

    fn __str__(&self) -> &str {
        self.0.as_str()
    }

    fn __repr__(&self) -> String {
        format!("Gtin14('{}')", self.0.as_str())
    }

    fn __eq__(&self, other: &PyGtin14) -> bool {
        self.0 == other.0
    }

    fn __hash__(&self) -> u64 {
        let mut hash = 0u64;
        for byte in self.0.as_str().bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(u64::from(byte));
        }
        hash
    }
}

/// A check-digit-agnostic match key. **Not a GTIN.**
///
/// Deliberately awkward to turn into a string. It has no `__str__`, so
/// `f"{key}"` and `str(key)` both produce `GtinKey('07390525907740')` rather
/// than a bare number — a key that leaks into an export shows up as visibly
/// wrong instead of as a plausible, fabricated identifier. The only way to
/// the digits is `as_key_str()`, which says what it is at the call site.
#[pyclass(name = "GtinKey", frozen, module = "barkod")]
#[derive(Clone)]
pub struct PyGtinKey(barkod::GtinKey);

#[pymethods]
impl PyGtinKey {
    /// The key as fourteen characters. Storing or exporting this stores or
    /// exports a match key.
    fn as_key_str(&self) -> &str {
        self.0.as_key_str()
    }

    fn __repr__(&self) -> String {
        format!("GtinKey('{}')", self.0.as_key_str())
    }

    fn __eq__(&self, other: &PyGtinKey) -> bool {
        self.0 == other.0
    }

    fn __hash__(&self) -> u64 {
        let mut hash = 0u64;
        for byte in self.0.as_key_str().bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(u64::from(byte));
        }
        hash
    }
}

/// The result of reading one cell.
#[pyclass(name = "Parsed", frozen, module = "barkod", get_all)]
pub struct PyParsed {
    /// The input, verbatim, whatever it was.
    pub raw: String,
    /// The canonical form, or `None`.
    pub gtin: Option<PyGtin14>,
    /// `"empty"`, `"non_numeric"`, `"too_short"`, `"too_long"`, or `None`.
    pub reason: Option<&'static str>,
    /// A sentence fit to show a user, or `None`.
    pub message: Option<&'static str>,
    /// How many digits were found, when the count is why it was refused.
    pub digits: Option<usize>,
    /// What to write when storing: canonical for a GTIN, the raw input
    /// untouched for anything else. Never blank unless the input was.
    pub store_form: String,
    /// Whether anything had to be removed to read the value.
    pub was_cleaned: bool,
    /// Whitespace was removed.
    pub removed_whitespace: bool,
    /// Dashes were removed.
    pub removed_separators: bool,
    /// A fractional part was dropped — the value arrived as a number.
    pub dropped_fraction: bool,
}

#[pymethods]
impl PyParsed {
    /// Whether this is a GTIN.
    #[getter]
    fn is_gtin(&self) -> bool {
        self.gtin.is_some()
    }

    /// The match key, or `None` outside the domain.
    fn key(&self) -> Option<PyGtinKey> {
        self.gtin.as_ref().map(PyGtin14::key)
    }

    fn __repr__(&self) -> String {
        match &self.gtin {
            Some(g) => format!("Parsed(gtin={})", g.__repr__()),
            None => format!(
                "Parsed(raw={:?}, reason={:?})",
                self.raw,
                self.reason.unwrap_or("")
            ),
        }
    }
}

fn build(input: &str) -> PyParsed {
    let parsed = barkod::parse(input);
    let cleaning = parsed.cleaning();
    let (reason, message, digits) = match parsed.reason() {
        None => (None, None, None),
        Some(r) => {
            let name = match r {
                Reason::Empty => "empty",
                Reason::NonNumeric => "non_numeric",
                Reason::TooShort { .. } => "too_short",
                Reason::TooLong { .. } => "too_long",
                _ => "unknown",
            };
            let count = match r {
                Reason::TooShort { digits } | Reason::TooLong { digits } => Some(digits),
                _ => None,
            };
            (Some(name), Some(r.message()), count)
        }
    };

    PyParsed {
        raw: parsed.raw().to_owned(),
        gtin: parsed.gtin().map(PyGtin14),
        reason,
        message,
        digits,
        store_form: parsed.store_form().to_owned(),
        was_cleaned: cleaning.is_some_and(|c| !c.is_pristine()),
        removed_whitespace: cleaning.is_some_and(|c| c.removed_whitespace()),
        removed_separators: cleaning.is_some_and(|c| c.removed_separators()),
        dropped_fraction: cleaning.is_some_and(|c| c.dropped_fraction()),
    }
}

fn encoding_name(encoding: Encoding) -> &'static str {
    match encoding {
        Encoding::Gtin8 => "GTIN-8",
        Encoding::Gtin12 => "GTIN-12",
        Encoding::Gtin13 => "GTIN-13",
        Encoding::Gtin14 => "GTIN-14",
    }
}

fn allocation_name(allocation: Allocation) -> &'static str {
    match allocation {
        Allocation::RestrictedCirculationCompany => "restricted_circulation_company",
        Allocation::RestrictedCirculationRegion => "restricted_circulation_region",
        Allocation::CompanyPrefix => "company_prefix",
        Allocation::UpcCompanyPrefix => "upc_company_prefix",
        Allocation::ReservedGs1Us => "reserved_gs1_us",
        Allocation::UnusedForGtin8 => "unused_for_gtin8",
        Allocation::Gtin8Issuance => "gtin8_issuance",
        Allocation::EpcGeneralManager => "epc_general_manager",
        Allocation::Demonstration => "demonstration",
        Allocation::Issn => "issn",
        Allocation::Isbn => "isbn",
        Allocation::Ismn => "ismn",
        Allocation::RefundReceipt => "refund_receipt",
        Allocation::Coupon => "coupon",
        Allocation::ReservedCoupon => "reserved_coupon",
        Allocation::ReservedFutureUse => "reserved_future_use",
        _ => "unknown",
    }
}

/// Read one cell.
#[pyfunction]
#[pyo3(name = "parse")]
fn py_parse(value: &Bound<'_, PyAny>) -> PyResult<PyParsed> {
    Ok(build(&as_text(value)?))
}

/// What to write when storing this value.
#[pyfunction]
#[pyo3(name = "store_form")]
fn py_store_form(value: &Bound<'_, PyAny>) -> PyResult<String> {
    let text = as_text(value)?;
    Ok(barkod::store_form(&text).into_owned())
}

/// The match key for this value, or `None` outside the domain.
#[pyfunction]
#[pyo3(name = "key")]
fn py_key(value: &Bound<'_, PyAny>) -> PyResult<Option<PyGtinKey>> {
    let text = as_text(value)?;
    Ok(barkod::key(&text).map(PyGtinKey))
}

/// Storage forms for a whole column. Releases the GIL for the work.
#[pyfunction]
#[pyo3(name = "store_form_many")]
fn py_store_form_many(py: Python<'_>, values: &Bound<'_, PyList>) -> PyResult<Vec<String>> {
    let inputs = collect(values)?;
    Ok(py.allow_threads(|| {
        inputs
            .iter()
            .map(|v| barkod::store_form(v).into_owned())
            .collect()
    }))
}

/// Match keys for a whole column, as strings, with `None` where there is no
/// key.
///
/// Named `key_strings` rather than `keys` on purpose. This is the one place
/// the type distinction is deliberately dropped — a dataframe column needs
/// strings — so the call site has to say so.
#[pyfunction]
#[pyo3(name = "key_strings")]
fn py_key_strings(py: Python<'_>, values: &Bound<'_, PyList>) -> PyResult<Vec<Option<String>>> {
    let inputs = collect(values)?;
    Ok(py.allow_threads(|| {
        inputs
            .iter()
            .map(|v| barkod::key(v).map(|k| k.as_key_str().to_owned()))
            .collect()
    }))
}

/// Parse a whole column.
#[pyfunction]
#[pyo3(name = "parse_many")]
fn py_parse_many(values: &Bound<'_, PyList>) -> PyResult<Vec<PyParsed>> {
    let inputs = collect(values)?;
    Ok(inputs.iter().map(|v| build(v)).collect())
}

fn collect(values: &Bound<'_, PyList>) -> PyResult<Vec<String>> {
    values
        .iter()
        .map(|item| as_text(&item))
        .collect::<PyResult<Vec<_>>>()
        .map_err(|e| {
            PyTypeError::new_err(format!("every item must be str, int, float or None: {e}"))
        })
}

/// barkod — GTIN parsing, canonicalisation, match keys and GS1 classification.
#[pymodule]
fn _barkod(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add(
        "__doc__",
        "GTIN parsing, canonicalisation, match keys and GS1 classification.",
    )?;
    m.add("DOMAIN_MIN_DIGITS", barkod::DOMAIN_MIN_DIGITS)?;
    m.add("DOMAIN_MAX_DIGITS", barkod::DOMAIN_MAX_DIGITS)?;
    m.add_class::<PyGtin14>()?;
    m.add_class::<PyGtinKey>()?;
    m.add_class::<PyParsed>()?;
    m.add_function(wrap_pyfunction!(py_parse, m)?)?;
    m.add_function(wrap_pyfunction!(py_store_form, m)?)?;
    m.add_function(wrap_pyfunction!(py_key, m)?)?;
    m.add_function(wrap_pyfunction!(py_store_form_many, m)?)?;
    m.add_function(wrap_pyfunction!(py_key_strings, m)?)?;
    m.add_function(wrap_pyfunction!(py_parse_many, m)?)?;
    Ok(())
}
