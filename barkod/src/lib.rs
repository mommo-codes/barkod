//! # barkod
//!
//! One implementation of what a GTIN is, callable from Rust, Python and
//! TypeScript.
//!
//! Not a general barcode library. barkod reads a string of digits and answers
//! questions about it: is this a GTIN, what is its canonical form, what key
//! should it match on, what has GS1 allocated its range for. It never renders
//! a barcode, never talks to a database, and never looks anything up.
//!
//! ## The law
//!
//! Every function agrees on the **domain** and differs only on the
//! **transformation** within it.
//!
//! The domain is **8 to 14 digits** — GTIN-8 is the shortest real encoding,
//! GTIN-14 the canonical form. Outside it, no GTIN operation is defined.
//! Inside it, transformations diverge freely: pad-only for the canonical
//! form, pad-and-zero-the-check-digit for the match key, drop-leading-zeros
//! for the shortest form.
//!
//! A lossy transformation is not a licence to be more permissive about what
//! counts as a GTIN. [`GtinKey`] discards the check digit so two spellings of
//! one product match despite a wrong or missing one — which only means
//! anything for a value that *has* a check digit. **Lossiness is a property
//! of the transformation, never of the domain.**
//!
//! The two refusals differ, and that difference is also the law:
//!
//! | Kind | Outside the domain | Why |
//! |---|---|---|
//! | Storage form ([`store_form`]) | the raw input, untouched | a stored value must never lose its only handle |
//! | Match key ([`key`]) | `None` | a key that does not exist must not match anything |
//!
//! ## Three functions
//!
//! ```
//! use barkod::{parse, store_form, key, Reason};
//!
//! // The primary entry point: a rich result, not a string.
//! let p = parse("7350053850019");
//! assert_eq!(p.gtin().unwrap().as_str(), "07350053850019");
//!
//! // Sugar for the two things callers actually do.
//! assert_eq!(store_form("7350053850019"), "07350053850019");
//! assert_eq!(key("7350053850019").unwrap().as_key_str(), "07350053850010");
//!
//! // Out of domain: kept, classified, never blanked and never truncated.
//! assert_eq!(store_form("073905259077450"), "073905259077450");
//! assert_eq!(key("073905259077450"), None);
//! assert_eq!(parse("073905259077450").reason(), Some(Reason::TooLong { digits: 15 }));
//! ```
//!
//! ## Classify, never reject
//!
//! An invalid check digit does not mean "not a GTIN". Variable-weight in-store
//! codes and internal PLUs are live products that fail GS1 Mod-10 by design.
//! In one real dataset, 109 rows of a product register and 2,429 rows of a
//! product catalogue were in exactly that state. barkod therefore has no
//! validity gate anywhere — [`Gtin14`] exists regardless, and
//! [`Gtin14::check_digit_is_valid`] is a question you ask it.
//!
//! ## A GTIN is not a primary key
//!
//! It is neither unique nor stable. Product variants legitimately share one,
//! suppliers reuse them, and the same physical item can carry several. Any
//! `SELECT ... WHERE gtin = ?` that expects one row is a bug waiting for the
//! second one. See `docs/not-a-primary-key.md` — this is the single most
//! repeated mistake in the systems this crate was written against.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
// The documentation here is prose about production incidents and the systems
// this crate replaces. `doc_markdown` wants every proper noun in backticks,
// which makes those paragraphs harder to read than the lint is worth.
#![allow(clippy::doc_markdown)]

mod allocation;
mod check;
mod clean;
mod encoding;
mod gtin14;
mod heuristics;
mod key;
mod parse;

pub use allocation::{Allocation, Indicator};
pub use clean::Cleaning;
pub use encoding::Encoding;
pub use gtin14::Gtin14;
pub use key::GtinKey;
pub use parse::{parse, Parsed, Reason, DOMAIN_MAX_DIGITS, DOMAIN_MIN_DIGITS};

use std::borrow::Cow;

/// What to write when storing this value.
///
/// The canonical GTIN-14 when the input is in the domain; the input itself,
/// untouched, when it is not. Borrows rather than allocates whenever the
/// answer is already the input.
///
/// **Never returns an empty string for a non-empty input.** Blanking junk is
/// how it becomes indistinguishable from missing data, so a value barkod
/// cannot read comes back exactly as it arrived.
///
/// This function exists so that the "…or else keep the raw" half of the rule
/// lives in one place. Five separate implementations each spelled it for
/// themselves, and they drifted.
///
/// ```
/// # use barkod::store_form;
/// assert_eq!(store_form("7350053850019"), "07350053850019"); // padded
/// assert_eq!(store_form("lek"), "lek");                       // untouched
/// assert_eq!(store_form(""), "");                             // still empty
/// ```
#[must_use]
pub fn store_form(input: &str) -> Cow<'_, str> {
    match parse(input) {
        Parsed::Gtin { value, raw, .. } => {
            if raw == value.as_str() {
                Cow::Borrowed(raw)
            } else {
                Cow::Owned(value.as_str().to_owned())
            }
        }
        Parsed::NotGtin { raw, .. } => Cow::Borrowed(raw),
    }
}

/// The match key for this value, or `None` outside the domain.
///
/// `None` means "this row cannot match anything", which is the honest answer
/// for a blank cell, a non-numeric sentinel, or an 18-digit identifier that
/// is not a GTIN. Manufacturing a key for those is how unrelated rows join:
/// an over-length value truncated into a real product's key attaches that
/// product's brand, VAT and category to the wrong row, silently.
///
/// ```
/// # use barkod::key;
/// assert_eq!(key("7390525907745").unwrap().as_key_str(), "07390525907740");
/// assert_eq!(key("073905259077450"), None); // 15 digits — no key, not a truncated one
/// assert_eq!(key("MANUAL-a1b2c3d"), None);
/// assert_eq!(key(""), None);
/// ```
#[must_use]
pub fn key(input: &str) -> Option<GtinKey> {
    parse(input).key()
}
