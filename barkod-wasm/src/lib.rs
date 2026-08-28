//! WebAssembly bindings for barkod.
//!
//! The reason this target exists: the JavaScript side of the platform had its
//! own reimplementation of these rules, which self-documented as a drift risk
//! and drifted anyway. Compiling the same source to wasm is what stops the
//! next input box becoming the next reimplementation.
//!
//! Everything here is a thin shell. No rule is decided in this file.

use barkod::{Indicator, Reason};
use wasm_bindgen::prelude::*;

/// A GTIN in canonical GTIN-14 form.
#[wasm_bindgen]
pub struct Gtin14(barkod::Gtin14);

#[wasm_bindgen]
impl Gtin14 {
    /// The canonical form: fourteen digits, check digit preserved.
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> String {
        self.0.as_str().to_owned()
    }

    /// The shortest standard form.
    #[wasm_bindgen(getter)]
    pub fn shortest(&self) -> String {
        self.0.shortest().to_owned()
    }

    /// `"GTIN-8"`, `"GTIN-12"`, `"GTIN-13"` or `"GTIN-14"`.
    #[wasm_bindgen(getter)]
    pub fn encoding(&self) -> String {
        self.0.encoding().to_string()
    }

    /// Digits that are not leading zeros.
    #[wasm_bindgen(getter, js_name = significantDigits)]
    pub fn significant_digits(&self) -> usize {
        self.0.significant_digits()
    }

    /// The check digit the value carries.
    #[wasm_bindgen(getter, js_name = checkDigit)]
    pub fn check_digit(&self) -> u8 {
        self.0.check_digit()
    }

    /// The check digit GS1 Mod-10 says it should carry.
    #[wasm_bindgen(getter, js_name = expectedCheckDigit)]
    pub fn expected_check_digit(&self) -> u8 {
        self.0.expected_check_digit()
    }

    /// Whether the check digit is correct. A classification, not a verdict.
    #[wasm_bindgen(getter, js_name = checkDigitIsValid)]
    pub fn check_digit_is_valid(&self) -> bool {
        self.0.check_digit_is_valid()
    }

    /// `"grouping"`, `"variable_measure"`, or `undefined`.
    #[wasm_bindgen(getter)]
    pub fn indicator(&self) -> Option<String> {
        self.0.indicator().map(|i| {
            if matches!(i, Indicator::VariableMeasure) {
                "variable_measure".to_owned()
            } else {
                "grouping".to_owned()
            }
        })
    }

    /// What GS1 allocated this range for, as a stable snake_case name.
    #[wasm_bindgen(getter)]
    pub fn allocation(&self) -> String {
        allocation_name(self.0.allocation()).to_owned()
    }

    /// GS1's own wording for the range.
    #[wasm_bindgen(getter, js_name = allocationDescription)]
    pub fn allocation_description(&self) -> String {
        self.0.allocation().description().to_owned()
    }

    /// Whether the range is restricted-circulation.
    #[wasm_bindgen(getter, js_name = isRestrictedCirculation)]
    pub fn is_restricted_circulation(&self) -> bool {
        self.0.allocation().is_restricted_circulation()
    }

    /// Fewer significant digits than GTIN-8 can express. A named guess.
    #[wasm_bindgen(getter, js_name = looksLikeInternalCode)]
    pub fn looks_like_internal_code(&self) -> bool {
        self.0.looks_like_internal_code()
    }

    /// Restricted-circulation, ends in five zeros, stale check digit. A
    /// named guess.
    #[wasm_bindgen(getter, js_name = measureFieldLooksZeroed)]
    pub fn measure_field_looks_zeroed(&self) -> bool {
        self.0.measure_field_looks_zeroed()
    }

    /// The match key, as a string.
    ///
    /// Named `keyString` because wasm cannot carry Rust's type distinction
    /// across the boundary. The TypeScript wrapper brands the result so the
    /// compiler can refuse to store it where a GTIN belongs — see
    /// `ts/index.ts`. Reach for this only through that wrapper.
    #[wasm_bindgen(js_name = keyString)]
    pub fn key_string(&self) -> String {
        self.0.key().as_key_str().to_owned()
    }

    /// The same first thirteen digits with a correct check digit. Export use
    /// only: it changes which product the code identifies whenever the
    /// original check digit was not a mistake.
    #[wasm_bindgen(js_name = withRecomputedCheckDigit)]
    pub fn with_recomputed_check_digit(&self) -> Gtin14 {
        Gtin14(self.0.with_recomputed_check_digit())
    }
}

/// The result of reading one cell.
#[wasm_bindgen]
pub struct Parsed {
    raw: String,
    gtin: Option<barkod::Gtin14>,
    reason: Option<&'static str>,
    message: Option<&'static str>,
    digits: Option<usize>,
    store_form: String,
    was_cleaned: bool,
    removed_whitespace: bool,
    removed_separators: bool,
    dropped_fraction: bool,
}

#[wasm_bindgen]
impl Parsed {
    /// The input, verbatim.
    #[wasm_bindgen(getter)]
    pub fn raw(&self) -> String {
        self.raw.clone()
    }

    /// Whether this is a GTIN.
    #[wasm_bindgen(getter, js_name = isGtin)]
    pub fn is_gtin(&self) -> bool {
        self.gtin.is_some()
    }

    /// The canonical form, or `undefined`.
    #[wasm_bindgen(getter)]
    pub fn gtin(&self) -> Option<Gtin14> {
        self.gtin.map(Gtin14)
    }

    /// `"empty"`, `"non_numeric"`, `"too_short"`, `"too_long"`, or
    /// `undefined`.
    #[wasm_bindgen(getter)]
    pub fn reason(&self) -> Option<String> {
        self.reason.map(ToOwned::to_owned)
    }

    /// A sentence fit to show a user, or `undefined`.
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> Option<String> {
        self.message.map(ToOwned::to_owned)
    }

    /// How many digits were found, when the count is why it was refused.
    #[wasm_bindgen(getter)]
    pub fn digits(&self) -> Option<usize> {
        self.digits
    }

    /// What to write when storing. Never blank unless the input was.
    #[wasm_bindgen(getter, js_name = storeForm)]
    pub fn store_form(&self) -> String {
        self.store_form.clone()
    }

    /// Whether anything had to be removed to read the value.
    #[wasm_bindgen(getter, js_name = wasCleaned)]
    pub fn was_cleaned(&self) -> bool {
        self.was_cleaned
    }

    /// Whitespace was removed.
    #[wasm_bindgen(getter, js_name = removedWhitespace)]
    pub fn removed_whitespace(&self) -> bool {
        self.removed_whitespace
    }

    /// Dashes were removed.
    #[wasm_bindgen(getter, js_name = removedSeparators)]
    pub fn removed_separators(&self) -> bool {
        self.removed_separators
    }

    /// A fractional part was dropped — the value arrived as a number.
    #[wasm_bindgen(getter, js_name = droppedFraction)]
    pub fn dropped_fraction(&self) -> bool {
        self.dropped_fraction
    }
}

/// Read one cell.
#[wasm_bindgen]
pub fn parse(input: &str) -> Parsed {
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

    Parsed {
        raw: parsed.raw().to_owned(),
        gtin: parsed.gtin(),
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

/// What to write when storing this value.
#[wasm_bindgen(js_name = storeForm)]
pub fn store_form(input: &str) -> String {
    barkod::store_form(input).into_owned()
}

/// The match key, or `undefined` outside the domain. See [`Gtin14::key_string`].
#[wasm_bindgen(js_name = keyString)]
pub fn key_string(input: &str) -> Option<String> {
    barkod::key(input).map(|k| k.as_key_str().to_owned())
}

/// Storage forms for a whole column.
#[wasm_bindgen(js_name = storeFormMany)]
pub fn store_form_many(inputs: Vec<String>) -> Vec<String> {
    inputs
        .iter()
        .map(|v| barkod::store_form(v).into_owned())
        .collect()
}

fn allocation_name(allocation: barkod::Allocation) -> &'static str {
    use barkod::Allocation as A;
    match allocation {
        A::RestrictedCirculationCompany => "restricted_circulation_company",
        A::RestrictedCirculationRegion => "restricted_circulation_region",
        A::CompanyPrefix => "company_prefix",
        A::UpcCompanyPrefix => "upc_company_prefix",
        A::ReservedGs1Us => "reserved_gs1_us",
        A::UnusedForGtin8 => "unused_for_gtin8",
        A::Gtin8Issuance => "gtin8_issuance",
        A::EpcGeneralManager => "epc_general_manager",
        A::Demonstration => "demonstration",
        A::Issn => "issn",
        A::Isbn => "isbn",
        A::Ismn => "ismn",
        A::RefundReceipt => "refund_receipt",
        A::Coupon => "coupon",
        A::ReservedCoupon => "reserved_coupon",
        A::ReservedFutureUse => "reserved_future_use",
        _ => "unknown",
    }
}
