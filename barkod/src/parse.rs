//! [`parse`] — the one door into the GTIN domain.

use crate::clean::{clean, Cleaning};
use crate::gtin14::Gtin14;
use crate::key::GtinKey;
use core::fmt;

/// The GTIN domain: 8 digits (GTIN-8, the shortest real encoding) through 14
/// (GTIN-14, the canonical form).
pub const DOMAIN_MIN_DIGITS: usize = 8;
/// See [`DOMAIN_MIN_DIGITS`].
pub const DOMAIN_MAX_DIGITS: usize = 14;

/// Why a value is not a GTIN.
///
/// Four outcomes, and the split between the first two is the point: a blank
/// cell is *missing*, `lek` is *junk*, and collapsing both to an empty string
/// is how one becomes indistinguishable from the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Reason {
    /// Nothing to read — empty, whitespace, or only dashes.
    Empty,
    /// Held a character that is neither a digit nor formatting.
    NonNumeric,
    /// Fewer than 8 digits, so not a truncated GTIN — malformed data.
    TooShort {
        /// How many digits were found.
        digits: usize,
    },
    /// More than 14 digits, so a different identifier, not a padded GTIN.
    ///
    /// Nothing in the value says which: `0000000012345670` is either a padded
    /// GTIN-8 or a 16-digit internal code, and an 18-digit SSCC's last 14
    /// digits form a well-formed GTIN-14 that identifies nothing. Truncating
    /// to fit manufactures a valid-looking wrong answer, so barkod reports
    /// the length and hands the value back untouched.
    TooLong {
        /// How many digits were found.
        digits: usize,
    },
}

impl Reason {
    /// A sentence fit to show a user next to their own spreadsheet cell.
    #[must_use]
    pub fn message(&self) -> &'static str {
        match self {
            Reason::Empty => "The cell is empty",
            Reason::NonNumeric => "Not a number — the cell holds more than digits",
            Reason::TooShort { .. } => "Fewer than 8 digits — too short to be a GTIN",
            Reason::TooLong { .. } => "More than 14 digits — not a GTIN",
        }
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

/// The result of reading one cell.
///
/// Both arms keep `raw` — the input, verbatim, always. A row that fails to
/// parse still has to be findable in the user's own file, and the only handle
/// it has is the text they typed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Parsed<'a> {
    /// Inside the domain.
    Gtin {
        /// The canonical form.
        value: Gtin14,
        /// The input, verbatim.
        raw: &'a str,
        /// What had to be removed to read it.
        cleaning: Cleaning,
    },
    /// Outside the domain, with the reason and the input kept intact.
    NotGtin {
        /// The input, verbatim.
        raw: &'a str,
        /// Why it is not a GTIN.
        reason: Reason,
    },
}

impl<'a> Parsed<'a> {
    /// The input, verbatim, whichever arm this is.
    #[must_use]
    pub fn raw(&self) -> &'a str {
        match self {
            Parsed::Gtin { raw, .. } | Parsed::NotGtin { raw, .. } => raw,
        }
    }

    /// The canonical form, if this is a GTIN.
    #[must_use]
    pub fn gtin(&self) -> Option<Gtin14> {
        match self {
            Parsed::Gtin { value, .. } => Some(*value),
            Parsed::NotGtin { .. } => None,
        }
    }

    /// Why this is not a GTIN, if it is not.
    #[must_use]
    pub fn reason(&self) -> Option<Reason> {
        match self {
            Parsed::NotGtin { reason, .. } => Some(*reason),
            Parsed::Gtin { .. } => None,
        }
    }

    /// What had to be removed, if this is a GTIN.
    #[must_use]
    pub fn cleaning(&self) -> Option<Cleaning> {
        match self {
            Parsed::Gtin { cleaning, .. } => Some(*cleaning),
            Parsed::NotGtin { .. } => None,
        }
    }

    /// Whether this is a GTIN.
    #[must_use]
    pub fn is_gtin(&self) -> bool {
        matches!(self, Parsed::Gtin { .. })
    }

    /// The match key, or `None` outside the domain.
    #[must_use]
    pub fn key(&self) -> Option<GtinKey> {
        self.gtin().map(|g| g.key())
    }

    /// What to write when storing this value: the canonical form for a GTIN,
    /// the raw input untouched for anything else.
    ///
    /// Never empty unless the input was. See [`crate::store_form`].
    #[must_use]
    pub fn store_form(&self) -> &str {
        match self {
            Parsed::Gtin { value, .. } => value.as_str(),
            Parsed::NotGtin { raw, .. } => raw,
        }
    }
}

/// Read one cell.
///
/// The primary entry point, and the only place the 8–14 domain is decided.
/// Allocates nothing.
///
/// ```
/// use barkod::{parse, Parsed, Reason};
///
/// // Padded to canonical form, check digit preserved.
/// assert_eq!(parse("7350053850019").store_form(), "07350053850019");
///
/// // Junk keeps its handle and gains a reason.
/// let p = parse("lek");
/// assert_eq!(p.raw(), "lek");
/// assert_eq!(p.reason(), Some(Reason::NonNumeric));
///
/// // Over-length is reported, never truncated.
/// assert_eq!(parse("370245100000000123").reason(), Some(Reason::TooLong { digits: 18 }));
/// assert_eq!(parse("370245100000000123").store_form(), "370245100000000123");
///
/// // Missing and junk are different answers.
/// assert_eq!(parse("").reason(), Some(Reason::Empty));
/// ```
#[must_use]
pub fn parse(input: &str) -> Parsed<'_> {
    let Ok(cleaned) = clean(input) else {
        return Parsed::NotGtin {
            raw: input,
            reason: Reason::NonNumeric,
        };
    };

    match cleaned.count {
        0 => Parsed::NotGtin {
            raw: input,
            reason: Reason::Empty,
        },
        n if n < DOMAIN_MIN_DIGITS => Parsed::NotGtin {
            raw: input,
            reason: Reason::TooShort { digits: n },
        },
        n if n > DOMAIN_MAX_DIGITS => Parsed::NotGtin {
            raw: input,
            reason: Reason::TooLong { digits: n },
        },
        n => Parsed::Gtin {
            value: Gtin14::from_digits(&cleaned.digits[..n]),
            raw: input,
            cleaning: cleaned.cleaning,
        },
    }
}
