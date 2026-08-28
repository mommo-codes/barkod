//! The match key — a different type from a GTIN, on purpose.

use crate::gtin14::Gtin14;
use core::fmt;

/// A check-digit-agnostic key for matching two spellings of one product.
///
/// Fourteen digits, the last always `'0'`. Two values whose first thirteen
/// digits agree share a key, whatever check digit each carries.
///
/// # Why this exists: matching variable-weight items
///
/// This is not a generic "be lenient about typos" key. It exists for one
/// measured, long-standing situation.
///
/// Retailers send weight-based items — deli, bakery, loose produce — with the
/// final digit wrong, usually forced to `0`. The venue's own assortment
/// carries the same thirteen leading digits with the *correct* check digit.
/// Raw equality never matches the two, so the product looks new when it is
/// not. Keying **both sides** to `first thirteen + '0'` makes them meet.
///
/// Measured on one real pair of retail assortment files — 11,584 rows in the
/// venue's current assortment against 12,317 in the incoming list:
///
/// | | |
/// |---|---|
/// | New rows with an invalid check digit | 389 |
/// | …in the restricted-circulation range (variable weight) | **389 — all of them** |
/// | …matched to a current product **only** via this key | **210** |
/// | …of those, current partner carries a valid check digit | **210 — all of them** |
/// | Valid new rows whose key hit a *different* current product | **0** |
/// | Distinct current GTINs sharing a key | **0** |
///
/// The technique predates this crate by years of production use. The numbers
/// above are why it keeps being right, not an argument that it might be.
///
/// ## The zeroing has to be unconditional
///
/// It is tempting to zero only when the check digit fails validation. That
/// breaks the match: the venue side keeps `base + c` while the retailer side
/// becomes `base + 0`, and the two no longer meet. A key that depends on a
/// property of the value it is keying is not a key. So every `Gtin14` gets
/// the same treatment, and `GtinKey` is a pure function of the first thirteen
/// digits — asserted as a property test, not a convention.
///
/// ## Two valid GTINs can never collide here
///
/// The check digit is a function of the first thirteen digits, so two *valid*
/// GTINs sharing a base are the same string. Every possible key collision
/// therefore requires at least one member with an invalid check digit — which
/// is exactly the population this key is for. Measured across a 14.8M-row
/// product catalogue and a 187k-row product register: zero groups containing
/// two valid GTINs.
///
/// ## Known limitation: internal PLU codes
///
/// There is one population where zeroing the last digit destroys real
/// information. In-store PLU codes use the final digit as a **product
/// discriminator**, not as a check digit, so collapsing it merges genuinely
/// different products:
///
/// ```text
/// 00000000081010..4   five varieties of loose fruit      5 products, 1 key
/// 00000000014150..4   four sizes of coffee               4 products, 1 key
/// ```
///
/// **This is deliberately not guarded against.** Eight of the eleven known
/// cases have fewer than 8 significant digits, so
/// [`Gtin14::looks_like_internal_code`] would catch them — but all eleven
/// occur in a product register, not in the assortment files this key actually
/// processes, where the measured collision count is zero. Making the key
/// conditional to defend against a population it does not meet would cost the
/// property the whole technique rests on.
///
/// A guard would not be sufficient anyway. `00868302706290` and `...92` are
/// two unrelated grocery items at twelve significant digits, both with invalid
/// check digits, and no structural rule separates them from a genuine
/// variable-weight pair.
///
/// **So: if you join on keys against a broad reference table rather than
/// against a single venue's assortment, check for multi-product keys
/// yourself.** Grouping by key and looking for more than one distinct
/// `Gtin14` finds every one of them. That the whole set is enumerable in a
/// single query is what makes the limitation safe to live with rather than
/// merely known — worth running periodically, with the count recorded so a
/// change is what gets noticed.
///
/// Note that several GTINs under one key is usually *correct*, not a
/// collision: ten sequential GTINs that turn out to be one product, or the
/// same item sent once with its check digit zeroed. The number to watch is
/// distinct **product names** per key, not GTINs per key.
///
/// # This is not a GTIN, and the type is what says so
///
/// A `GtinKey` identifies nothing. It is a bucket that a product falls into,
/// and roughly nine in ten of them end in a check digit no product has. Write
/// one into a file and you have published a plausible, wrong identifier.
///
/// That is not hypothetical. It is what one product export did for its whole
/// life, publishing keys under a column heading that read as a GTIN, until the
/// audit that produced this crate found it — roughly nine in ten of the values
/// in that column carried a fabricated check digit. So `GtinKey`:
///
/// - has **no** `Display`, so `format!("{key}")` does not compile;
/// - has **no** `Deref<Target = str>`, `AsRef<str>` or `Into<String>`;
/// - cannot be converted back into a [`Gtin14`] by any route;
/// - cannot be compared to a [`Gtin14`];
/// - reaches a string only through [`as_key_str`](GtinKey::as_key_str), which
///   forces the call site to say the word *key* out loud.
///
/// Its `Debug` deliberately keeps the wrapper visible, so a key that leaks
/// into a log or an error message reads as `GtinKey("07390525907740")` rather
/// than as a bare number someone might paste into a search box.
///
/// ```
/// # use barkod::parse;
/// let key = parse("7390525907745").gtin().unwrap().key();
/// assert_eq!(key.as_key_str(), "07390525907740");
///
/// // Same product, wrong check digit — same key.
/// let typo = parse("7390525907741").gtin().unwrap().key();
/// assert_eq!(key, typo);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GtinKey(pub(crate) [u8; 14]);

impl GtinKey {
    /// The key as fourteen characters.
    ///
    /// Named for what it returns. Anything that stores, exports or displays
    /// the result is storing, exporting or displaying a match key, and the
    /// name at the call site is the last chance to notice that.
    #[must_use]
    pub fn as_key_str(&self) -> &str {
        core::str::from_utf8(&self.0).unwrap_or("<non-utf8 key>")
    }
}

impl From<Gtin14> for GtinKey {
    fn from(gtin: Gtin14) -> Self {
        gtin.key()
    }
}

impl From<&Gtin14> for GtinKey {
    fn from(gtin: &Gtin14) -> Self {
        gtin.key()
    }
}

impl fmt::Debug for GtinKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GtinKey({:?})", self.as_key_str())
    }
}
