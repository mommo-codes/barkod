//! Inferences, kept apart from facts.
//!
//! Everything in [`allocation`](crate::allocation) and
//! [`encoding`](crate::encoding) is definitional: GS1 says what a range is
//! for, so those answers are right forever. The two predicates here are not.
//! They are patterns observed in production data, and they are named as
//! guesses — `looks_like`, not `is` — so a call site cannot mistake one for
//! the other.
//!
//! This is why barkod has no `VariableWeight` or `InternalPlu` classification.
//! Those names assert something the data cannot prove. A restricted-circulation
//! code with a zeroed measure field is *probably* a variable-weight item; a
//! number too short to have been allocated as a GTIN is *probably* an in-store
//! PLU. Naming the evidence instead of the conclusion keeps the uncertainty
//! visible at the call site, where the caller — who knows which market and
//! which import produced the row — can decide what it means.

use crate::gtin14::Gtin14;

impl Gtin14 {
    /// Fewer significant digits than the shortest GS1 encoding can express.
    ///
    /// GTIN-8 is the smallest allocated form, so a value with 7 or fewer
    /// significant digits was never issued as a GTIN by anyone. In a product
    /// register these turn out to be in-store PLU codes — counter items, sold
    /// by the cup or by the piece. 101 of 187,132 numeric rows were in this
    /// shape.
    ///
    /// A guess, because nothing in the number itself says "PLU". What it does
    /// say for certain is that no GS1 member organisation issued it.
    #[must_use]
    pub fn looks_like_internal_code(&self) -> bool {
        self.significant_digits() < 8
    }

    /// A restricted-circulation code whose trailing measure field looks
    /// zeroed, leaving the check digit stale.
    ///
    /// In-store variable-weight items carry an embedded weight or price in
    /// their trailing digits. When an export path zeroes that field, the
    /// check digit — computed over the original digits — no longer matches.
    /// The result is a real, scannable product that fails validation.
    ///
    /// Measured in one 14.8M-row product catalogue: a single market held
    /// 2,414 rows with a wrong check digit against roughly 5 in each of the
    /// others — a 500× outlier, so one mechanism rather than 2,414 accidents.
    /// 2,408 of them (99.75%) are in the `2x` restricted-circulation range and
    /// 2,387 (98.9%) end in five zeros.
    ///
    /// A guess, and a market-specific one. Another market in the same data
    /// holds 31,461 rows in that range with their measure fields intact, so
    /// the range alone means nothing — it is the combination that is
    /// suggestive.
    #[must_use]
    pub fn measure_field_looks_zeroed(&self) -> bool {
        self.allocation().is_restricted_circulation()
            && self.as_str().ends_with("00000")
            && !self.check_digit_is_valid()
    }
}
