//! Reading digits out of a raw cell, and recording what had to be removed.
//!
//! This is the only place in the crate that decides what counts as
//! *formatting* and what counts as *not a GTIN*. That distinction is the
//! whole reason the module exists, so it is stated once, here:
//!
//! - Whitespace and dashes are formatting. `" 7350053850019\r"` and
//!   `"7-350-053-850-019"` are a GTIN that someone typed or exported with
//!   decoration around it.
//! - One decimal separator with digits on both sides is float debris. A
//!   spreadsheet stored the number, not the string, and it came back as
//!   `"7318690123456.0"`.
//! - **Anything else means the value is not a GTIN.** Scraping the digits out
//!   of `"abc12345678"` would manufacture a well-formed-looking GTIN-8 from
//!   something that was never an identifier.
//!
//! That last rule is stricter than the implementations this crate replaced,
//! which deleted every non-digit character unconditionally. The measured cost
//! of the change was zero: across 188,699 rows of a real product register the
//! only non-numeric values were a stray word and a synthetic manual-entry id,
//! and both are refused either way.

use core::fmt;

/// What [`parse`](crate::parse) had to remove before it could read the value.
///
/// Recorded rather than hidden. A caller that wants to be strict — an
/// importer deciding whether a source file is clean enough to trust — can ask;
/// a caller that just wants the number can ignore it. What neither can do is
/// silently accept a value that needed rescuing.
///
/// ```
/// # use barkod::{parse, Parsed};
/// let p = parse(" 7350053850019 ");
/// assert!(matches!(p, Parsed::Gtin { .. }));
/// assert!(!p.cleaning().unwrap().is_pristine());
/// assert!(p.cleaning().unwrap().removed_whitespace());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Cleaning {
    pub(crate) whitespace: bool,
    pub(crate) separators: bool,
    pub(crate) fraction: bool,
}

impl Cleaning {
    /// True when the input was already nothing but digits.
    #[must_use]
    pub fn is_pristine(&self) -> bool {
        !self.whitespace && !self.separators && !self.fraction
    }

    /// Whitespace was removed — spaces, tabs, newlines, non-breaking spaces,
    /// zero-width spaces, or a byte-order mark.
    #[must_use]
    pub fn removed_whitespace(&self) -> bool {
        self.whitespace
    }

    /// Dashes were removed — ASCII hyphen-minus, the Unicode hyphens and
    /// dashes, or a soft hyphen.
    #[must_use]
    pub fn removed_separators(&self) -> bool {
        self.separators
    }

    /// A fractional part was dropped: the value arrived as a number, not a
    /// string. `"7318690123456.0"` is a GTIN-13 that went through a float.
    ///
    /// Dropping the fraction is not the same as deleting the `.`, and the
    /// difference is not cosmetic: deleting it turns `"7318690123456.0"` into
    /// the 14-digit `"73186901234560"`, which looks exactly like a real
    /// GTIN-14 and is not one.
    #[must_use]
    pub fn dropped_fraction(&self) -> bool {
        self.fraction
    }
}

impl fmt::Display for Cleaning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_pristine() {
            return f.write_str("nothing removed");
        }
        let mut first = true;
        for (flag, label) in [
            (self.whitespace, "whitespace removed"),
            (self.separators, "dashes removed"),
            (self.fraction, "fractional part dropped"),
        ] {
            if flag {
                if !first {
                    f.write_str(", ")?;
                }
                f.write_str(label)?;
                first = false;
            }
        }
        Ok(())
    }
}

/// Digits recovered from a raw cell, plus the record of what was removed.
pub(crate) struct Cleaned {
    /// The first 14 digits found. Meaningful only when `count <= 14`.
    pub(crate) digits: [u8; 14],
    /// Every digit found, including ones past the fourteenth.
    pub(crate) count: usize,
    pub(crate) cleaning: Cleaning,
}

/// The input held a character that is neither a digit nor formatting.
pub(crate) struct NotNumeric;

/// True for characters that carry no meaning inside a number.
///
/// `char::is_whitespace` covers the non-breaking space (U+00A0) that Excel
/// exports produce. The three explicit additions are invisible characters that
/// `is_whitespace` reports as `false` but that appear in real copy-pasted data.
fn is_blank(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, '\u{200B}' | '\u{FEFF}' | '\u{2060}')
}

/// True for every character that is unambiguously a dash.
///
/// A closed set, listed rather than inferred: guessing at "punctuation" would
/// eventually strip something that changes the value.
fn is_dash(ch: char) -> bool {
    matches!(
        ch,
        '-' | '\u{00AD}'
            | '\u{2010}'
            | '\u{2011}'
            | '\u{2012}'
            | '\u{2013}'
            | '\u{2014}'
            | '\u{2015}'
            | '\u{2212}'
    )
}

/// Read the digits out of `input`, or refuse.
///
/// Allocates nothing. Digits past the fourteenth are counted but not stored —
/// an over-length value is never a GTIN, so only its length is ever needed.
pub(crate) fn clean(input: &str) -> Result<Cleaned, NotNumeric> {
    let mut digits = [b'0'; 14];
    let mut count = 0usize;
    let mut cleaning = Cleaning::default();
    let mut in_fraction = false;
    let mut fraction_digits = 0usize;

    for ch in input.chars() {
        if ch.is_ascii_digit() {
            if in_fraction {
                fraction_digits += 1;
            } else {
                if count < 14 {
                    digits[count] = ch as u8;
                }
                count += 1;
            }
        } else if is_blank(ch) {
            cleaning.whitespace = true;
        } else if is_dash(ch) {
            cleaning.separators = true;
        } else if ch == '.' || ch == ',' {
            // A second separator, or one with no digits before it, is not a
            // number — `"1.2.3"` and `".5"` are values of some other kind.
            if in_fraction || count == 0 {
                return Err(NotNumeric);
            }
            in_fraction = true;
        } else {
            return Err(NotNumeric);
        }
    }

    if in_fraction {
        // A trailing separator with nothing after it is malformed, not a
        // number with an empty fraction.
        if fraction_digits == 0 {
            return Err(NotNumeric);
        }
        cleaning.fraction = true;
    }

    Ok(Cleaned {
        digits,
        count,
        cleaning,
    })
}
