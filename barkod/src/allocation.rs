//! What GS1 has allocated a number range for.
//!
//! Transcribed from the **GS1 General Specifications, Release 23.0 (ratified
//! January 2023)**, Figure 1.4.2-1 *Synopsis of GS1 Prefix ranges* and Figure
//! 1.4.3-1 *Synopsis of GS1-8 Prefixes*. Every arm below corresponds to one
//! line of one of those two figures, and nothing here was inferred.
//!
//! Two tables, because GS1 defines two. A GTIN-8's prefix is read from its own
//! allocation table, not from the one that governs GTIN-12/13/14 — the ranges
//! genuinely differ, so classifying an 8-digit value with the 13-digit table
//! would give a confidently wrong answer.
//!
//! **This is structure, not a registry.** These ranges say what a number range
//! is *for*. They do not say which company or country owns a prefix — that is
//! an allocation registry that changes, and it is deliberately not here. See
//! `docs/no-data.md`.

use core::fmt;

/// A GTIN-14 indicator digit.
///
/// Only meaningful on a value with 14 significant digits; see
/// [`Gtin14::indicator`](crate::Gtin14::indicator).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Indicator {
    /// 1–8: a standard grouping of a retail item — a case, a pallet.
    ///
    /// Never `0`: a leading zero makes the value 13 significant digits or
    /// fewer, which has no indicator at all.
    Grouping(u8),
    /// 9: reserved for variable measure trade items, whose weight, dimension
    /// or volume varies per item and travels in a separate element string.
    VariableMeasure,
}

impl Indicator {
    pub(crate) fn from_digit(digit: u8) -> Self {
        if digit == 9 {
            Indicator::VariableMeasure
        } else {
            Indicator::Grouping(digit)
        }
    }
}

/// What a GS1 prefix range is allocated for.
///
/// Total over both tables: every possible value maps to exactly one variant,
/// so there is no `Unknown` and no way for this to return a non-answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Allocation {
    /// Restricted Circulation Numbers within a company — in-store codes that
    /// mean nothing outside the shop that issued them.
    RestrictedCirculationCompany,
    /// Restricted Circulation Numbers within a geographic region. The `2x`
    /// range: in-store variable-weight items live here.
    RestrictedCirculationRegion,
    /// An ordinary GS1 Company Prefix.
    CompanyPrefix,
    /// A GS1 Company Prefix from which a U.P.C. Company Prefix can be derived.
    UpcCompanyPrefix,
    /// Reserved by GS1 US for future use (`05`).
    ReservedGs1Us,
    /// Unused, to avoid collision with GTIN-8 (`0000001`–`0000099`).
    UnusedForGtin8,
    /// Used to issue GTIN-8s.
    Gtin8Issuance,
    /// General Manager Numbers for the EPC General Identifier scheme (`951`).
    EpcGeneralManager,
    /// Demonstrations and examples of the GS1 system (`952`).
    Demonstration,
    /// ISSN International Centre, for serial publications (`977`).
    Issn,
    /// International ISBN Agency, for books (`978`–`979`).
    Isbn,
    /// International ISMN Agency, for printed music (`9790`).
    Ismn,
    /// GS1 identification of refund receipts (`980`).
    RefundReceipt,
    /// GS1 coupon identification (`981`–`983`, `99`).
    Coupon,
    /// Reserved for future GS1 coupon identification (`984`–`989`).
    ReservedCoupon,
    /// Reserved for future use in the GTIN-8 table (`977`–`999`).
    ReservedFutureUse,
}

impl Allocation {
    /// True for both restricted-circulation ranges.
    ///
    /// A number in these ranges is meaningful only inside the company or
    /// region that issued it. It is still a real, scannable product — 4,414
    /// rows of one real product register are in the `2x` range — so this is a
    /// fact about the number, never a reason to refuse it.
    #[must_use]
    pub fn is_restricted_circulation(&self) -> bool {
        matches!(
            self,
            Allocation::RestrictedCirculationCompany | Allocation::RestrictedCirculationRegion
        )
    }

    /// True when the range identifies something other than a trade item:
    /// books, serials, printed music, coupons, refund receipts.
    #[must_use]
    pub fn is_publication_or_coupon(&self) -> bool {
        matches!(
            self,
            Allocation::Issn
                | Allocation::Isbn
                | Allocation::Ismn
                | Allocation::Coupon
                | Allocation::ReservedCoupon
                | Allocation::RefundReceipt
        )
    }

    /// GS1's own wording for this range.
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Allocation::RestrictedCirculationCompany => {
                "Restricted Circulation Numbers within a company"
            }
            Allocation::RestrictedCirculationRegion => {
                "Restricted Circulation Numbers within a geographic region"
            }
            Allocation::CompanyPrefix => "GS1 Company Prefixes",
            Allocation::UpcCompanyPrefix => {
                "GS1 Company Prefixes from which U.P.C. Company Prefixes can be derived"
            }
            Allocation::ReservedGs1Us => "GS1 US reserved for future use",
            Allocation::UnusedForGtin8 => "Unused to avoid collision with GTIN-8",
            Allocation::Gtin8Issuance => "Used to issue GTIN-8s",
            Allocation::EpcGeneralManager => {
                "General Manager Numbers for the EPC General Identifier (GID) scheme"
            }
            Allocation::Demonstration => "Demonstrations and examples of the GS1 system",
            Allocation::Issn => "ISSN International Centre, for serial publications",
            Allocation::Isbn => "International ISBN Agency, for books",
            Allocation::Ismn => "International ISMN Agency, for printed music",
            Allocation::RefundReceipt => "GS1 identification of refund receipts",
            Allocation::Coupon => "GS1 coupon identification",
            Allocation::ReservedCoupon => "Reserved for future GS1 coupon identification",
            Allocation::ReservedFutureUse => "Reserved for future use",
        }
    }

    /// GS1 General Specifications 23.0, Figure 1.4.2-1.
    ///
    /// `body` is the 13-digit form: the canonical GTIN-14 without its leading
    /// character. That single rule reads correctly for every encoding — a
    /// padded GTIN-13 exposes its own 13 digits, a padded GTIN-12 exposes the
    /// leading zero that the U.P.C.-derivable ranges are written around, and a
    /// true GTIN-14 exposes the digits following its indicator. Reading the
    /// prefix off the 14-digit string instead would report `0` for every
    /// padded value in the database.
    // Arms are one-to-one with the rows of Figure 1.4.2-1 and stay that way
    // even where two rows share an answer. Merging `300..=950` with
    // `953..=976` would save a line and lose the property that makes this
    // function checkable against the specification.
    #[allow(clippy::match_same_arms)]
    pub(crate) fn from_gs1_prefix(body: &str) -> Allocation {
        let bytes = body.as_bytes();
        let digit = |i: usize| {
            bytes
                .get(i)
                .map_or(0u32, |b| u32::from(b.wrapping_sub(b'0')))
        };

        let p2 = digit(0) * 10 + digit(1);
        let p3 = p2 * 10 + digit(2);

        if digit(0) == 0 {
            let p4 = p3 * 10 + digit(3);
            let p5 = p4 * 10 + digit(4);
            if p3 == 0 {
                if p4 == 0 {
                    if p5 == 0 {
                        let p7 = digit(5) * 10 + digit(6);
                        return if p7 == 0 {
                            Allocation::RestrictedCirculationCompany // 0000000
                        } else {
                            Allocation::UnusedForGtin8 // 0000001–0000099
                        };
                    }
                    return Allocation::UpcCompanyPrefix; // 00001–00009
                }
                return Allocation::UpcCompanyPrefix; // 0001–0009
            }
            if p3 <= 19 {
                return Allocation::UpcCompanyPrefix; // 001–019
            }
            return match p2 {
                2 => Allocation::RestrictedCirculationRegion,  // 02
                4 => Allocation::RestrictedCirculationCompany, // 04
                5 => Allocation::ReservedGs1Us,                // 05
                _ => Allocation::UpcCompanyPrefix,             // 03, 06–09
            };
        }

        match p2 {
            10..=19 => Allocation::CompanyPrefix,
            20..=29 => Allocation::RestrictedCirculationRegion,
            99 => Allocation::Coupon,
            // p2 is 30..=98 here, so p3 is 300..=989.
            _ => match p3 {
                300..=950 => Allocation::CompanyPrefix,
                951 => Allocation::EpcGeneralManager,
                952 => Allocation::Demonstration,
                953..=976 => Allocation::CompanyPrefix,
                977 => Allocation::Issn,
                978 => Allocation::Isbn,
                979 => {
                    if digit(3) == 0 {
                        Allocation::Ismn // 9790, sub-allocated out of 979
                    } else {
                        Allocation::Isbn
                    }
                }
                980 => Allocation::RefundReceipt,
                981..=983 => Allocation::Coupon,
                _ => {
                    debug_assert!((984..=989).contains(&p3), "unreachable GS1 prefix {p3}");
                    Allocation::ReservedCoupon // 984–989
                }
            },
        }
    }

    /// GS1 General Specifications 23.0, Figure 1.4.3-1 — the GTIN-8 table.
    // One arm per row of the figure; see `from_gs1_prefix`.
    #[allow(clippy::match_same_arms)]
    pub(crate) fn from_gs1_8_prefix(eight: &str) -> Allocation {
        let bytes = eight.as_bytes();
        let digit = |i: usize| {
            bytes
                .get(i)
                .map_or(0u32, |b| u32::from(b.wrapping_sub(b'0')))
        };
        let p3 = digit(0) * 100 + digit(1) * 10 + digit(2);

        match p3 {
            0..=99 => Allocation::RestrictedCirculationCompany,
            100..=199 => Allocation::Gtin8Issuance,
            200..=299 => Allocation::RestrictedCirculationCompany,
            300..=951 => Allocation::Gtin8Issuance,
            952 => Allocation::Demonstration,
            953..=976 => Allocation::Gtin8Issuance,
            _ => Allocation::ReservedFutureUse, // 977–999
        }
    }
}

impl fmt::Display for Allocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.description())
    }
}
