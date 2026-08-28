//! [`Gtin14`] — the canonical form, and everything you can ask a value that
//! reached it.

use crate::allocation::{Allocation, Indicator};
use crate::check::check_digit;
use crate::encoding::Encoding;
use crate::key::GtinKey;
use core::fmt;

/// A value that is inside the GTIN domain, in canonical GTIN-14 form.
///
/// Fourteen ASCII digits, zero-padded on the left, **check digit preserved**.
/// `Copy`, `Ord` and `Hash`, and it allocates nothing.
///
/// # What this type does and does not promise
///
/// It promises the value had 8 to 14 digits and is now written as 14. It does
/// **not** promise the check digit is correct, and that is deliberate: in one
/// real dataset 109 register rows and 2,429 catalogue rows carried a wrong one
/// and were live products a merchant could scan. A type that refused them
/// would refuse real inventory. Ask
/// [`check_digit_is_valid`](Gtin14::check_digit_is_valid) if you need to know;
/// do not assume.
///
/// # It cannot be built any other way
///
/// There is no constructor. The only route in is [`parse`](crate::parse),
/// which is where the domain rule lives, so every `Gtin14` in existence went
/// through it exactly once.
///
/// That has a consequence worth stating: [`shortest`](Gtin14::shortest)
/// cannot receive an over-length string, because an over-length string never
/// becomes a `Gtin14`. The bug where a shrink routine computed its offsets
/// from a hard-coded width of 14 and returned a plausible wrong answer for
/// anything longer is not fixed here — it is unrepresentable.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Gtin14(pub(crate) [u8; 14]);

impl Gtin14 {
    /// Build from 8–14 digits. Private: the domain check lives in `parse`.
    pub(crate) fn from_digits(digits: &[u8]) -> Self {
        let mut out = [b'0'; 14];
        let start = 14 - digits.len();
        out[start..].copy_from_slice(digits);
        Self(out)
    }

    /// The canonical form: always exactly fourteen characters.
    #[must_use]
    pub fn as_str(&self) -> &str {
        // Infallible: the array holds ASCII digits by construction.
        core::str::from_utf8(&self.0).unwrap_or("00000000000000")
    }

    /// Digits that are not leading zeros. `0` for an all-zero value.
    #[must_use]
    pub fn significant_digits(&self) -> usize {
        14 - self.0.iter().take_while(|&&b| b == b'0').count()
    }

    /// The shortest standard GTIN form, as a borrowed suffix of the canonical
    /// form — no allocation, and no way for it to disagree with `as_str`.
    ///
    /// Drops leading zeros as far as a standard width allows: to 8 if six can
    /// go, else 12, else 13, else the 14 stay. Widths 9, 10 and 11 are not
    /// GTIN encodings and are never produced.
    ///
    /// ```
    /// # use barkod::parse;
    /// assert_eq!(parse("00000007350001").gtin().unwrap().shortest(), "07350001");
    /// assert_eq!(parse("07390525907745").gtin().unwrap().shortest(), "7390525907745");
    /// assert_eq!(parse("073905259077").gtin().unwrap().shortest(), "073905259077");
    /// ```
    #[must_use]
    pub fn shortest(&self) -> &str {
        let start = match self.significant_digits() {
            0..=8 => 6,
            9..=12 => 2,
            13 => 1,
            _ => 0,
        };
        &self.as_str()[start..]
    }

    /// Which standard encoding [`shortest`](Gtin14::shortest) occupies.
    #[must_use]
    pub fn encoding(&self) -> Encoding {
        match self.significant_digits() {
            0..=8 => Encoding::Gtin8,
            9..=12 => Encoding::Gtin12,
            13 => Encoding::Gtin13,
            _ => Encoding::Gtin14,
        }
    }

    /// The check-digit-agnostic match key. See [`GtinKey`] before using it.
    #[must_use]
    pub fn key(&self) -> GtinKey {
        let mut out = self.0;
        out[13] = b'0';
        GtinKey(out)
    }

    fn body(&self) -> [u8; 13] {
        let mut body = [b'0'; 13];
        body.copy_from_slice(&self.0[..13]);
        body
    }

    /// The check digit the value actually carries, as `0..=9`.
    #[must_use]
    pub fn check_digit(&self) -> u8 {
        self.0[13] - b'0'
    }

    /// The check digit GS1 Mod-10 says it should carry, as `0..=9`.
    #[must_use]
    pub fn expected_check_digit(&self) -> u8 {
        check_digit(&self.body()) - b'0'
    }

    /// Whether the check digit is correct.
    ///
    /// A `false` here is a classification, not an error — see the type-level
    /// documentation. Variable-weight in-store codes fail this on purpose.
    #[must_use]
    pub fn check_digit_is_valid(&self) -> bool {
        self.0[13] == check_digit(&self.body())
    }

    /// The same first thirteen digits with a correct check digit appended.
    ///
    /// **Export use only, and think first.** This changes which product the
    /// code identifies whenever the original check digit was not a mistake.
    /// In the 2x restricted-circulation range a "wrong" check digit is
    /// usually correct data — the trailing measure field was zeroed after the
    /// digit was computed — so repairing it invents an identifier for a
    /// product that has one.
    ///
    /// Legitimate uses look like showing a user what the digit *would* be, or
    /// filling one in on a row being created for the first time. Rewriting
    /// stored data with it is how a fifth GTIN implementation turned
    /// `073905259077450` into `73905259077451`.
    #[must_use]
    pub fn with_recomputed_check_digit(&self) -> Gtin14 {
        let mut out = self.0;
        out[13] = check_digit(&self.body());
        Gtin14(out)
    }

    /// The GTIN-14 indicator digit, when the value actually has one.
    ///
    /// `None` for anything with fewer than 14 significant digits, where the
    /// leading digit is padding rather than an indicator. Reading it as an
    /// indicator regardless would report "indicator 0" for every padded row
    /// in a catalogue — which, since GTIN-13 is the retail default, is most
    /// of them.
    #[must_use]
    pub fn indicator(&self) -> Option<Indicator> {
        if self.significant_digits() == 14 {
            Some(Indicator::from_digit(self.0[0] - b'0'))
        } else {
            None
        }
    }

    /// What GS1 has allocated this number range for.
    #[must_use]
    pub fn allocation(&self) -> Allocation {
        if self.encoding() == Encoding::Gtin8 {
            Allocation::from_gs1_8_prefix(self.shortest())
        } else {
            Allocation::from_gs1_prefix(&self.as_str()[1..])
        }
    }
}

impl fmt::Display for Gtin14 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Gtin14 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gtin14({:?})", self.as_str())
    }
}
