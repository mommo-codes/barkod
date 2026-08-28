//! Which standard GTIN encoding a value's shortest form occupies.

use core::fmt;

/// The four standard GTIN encodings.
///
/// This enum is **total**: every [`Gtin14`](crate::Gtin14) has exactly one,
/// because [`Gtin14::shortest`](crate::Gtin14::shortest) always lands on one
/// of these four widths. There is no `Unknown` variant and no escape hatch,
/// so [`Gtin14::encoding`](crate::Gtin14::encoding) cannot return a
/// non-answer.
///
/// A value with 9, 10 or 11 significant digits is not a counter-example —
/// those are not GTIN encodings, so such a value's shortest *standard* form
/// is `Gtin12`, reached by keeping the leading zeros that get it there. This
/// is not a corner case: 1,363 rows of one real product register are in that
/// shape.
///
/// The older names are aliases for the same numbers: GTIN-8 is EAN-8, GTIN-12
/// is UPC-A, GTIN-13 is EAN-13. barkod uses the GTIN names throughout because
/// EAN-13 and UPC-A are barcode *symbologies* — ways of printing bars — and
/// this crate never touches bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Encoding {
    /// 8 digits. Small items with no room for a full symbol.
    Gtin8,
    /// 12 digits. North America; formerly UPC-A.
    Gtin12,
    /// 13 digits. The global retail default; formerly EAN-13.
    Gtin13,
    /// 14 digits. Cases, pallets and other groupings of a retail item.
    Gtin14,
}

impl Encoding {
    /// How many digits this encoding is written with.
    #[must_use]
    pub fn digits(&self) -> usize {
        match self {
            Encoding::Gtin8 => 8,
            Encoding::Gtin12 => 12,
            Encoding::Gtin13 => 13,
            Encoding::Gtin14 => 14,
        }
    }
}

impl fmt::Display for Encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Encoding::Gtin8 => "GTIN-8",
            Encoding::Gtin12 => "GTIN-12",
            Encoding::Gtin13 => "GTIN-13",
            Encoding::Gtin14 => "GTIN-14",
        })
    }
}
