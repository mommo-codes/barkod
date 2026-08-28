//! The domain law, and the two refusal spellings.
//!
//! These are hand-written from the decided behaviour, not generated from the
//! implementation. A generated corpus can only prove that two things agree;
//! it cannot prove either is right. Everything asserted here was decided
//! first and typed in by hand.

use barkod::{key, parse, store_form, Reason};

#[test]
fn domain_is_eight_through_fourteen_digits() {
    for digits in 8..=14 {
        let input = "7".repeat(digits);
        assert!(parse(&input).is_gtin(), "{digits} digits should be a GTIN");
    }
    for digits in 1..8 {
        let input = "7".repeat(digits);
        assert!(!parse(&input).is_gtin(), "{digits} digits is too short");
    }
    for digits in 15..40 {
        let input = "7".repeat(digits);
        assert!(!parse(&input).is_gtin(), "{digits} digits is too long");
    }
}

#[test]
fn storage_preserves_and_keys_refuse() {
    // The asymmetry is the law: a stored value keeps its handle, a key that
    // does not exist must not match anything.
    for outside in [
        "",
        "   ",
        "lek",
        "MANUAL-a1b2c3d",
        "1234567",
        "073905259077450",
        "370245100000000123",
    ] {
        assert_eq!(
            store_form(outside),
            outside,
            "storage must not alter {outside:?}"
        );
        assert_eq!(key(outside), None, "there is no key for {outside:?}");
    }
}

#[test]
fn over_length_is_reported_never_truncated() {
    // Truncating manufactures a valid-looking wrong answer: the first 13
    // digits of this 15-digit value are a real product's.
    let corrupt = "073905259077450";
    let genuine = "07390525907745";

    assert_eq!(store_form(corrupt), corrupt);
    assert_eq!(
        parse(corrupt).reason(),
        Some(Reason::TooLong { digits: 15 })
    );

    // The trap this closes: the two must not collide.
    assert_eq!(key(corrupt), None);
    assert!(key(genuine).is_some());
}

#[test]
fn missing_and_junk_are_different_answers() {
    // Collapsing both to an empty string is how junk becomes
    // indistinguishable from missing data.
    assert_eq!(parse("").reason(), Some(Reason::Empty));
    assert_eq!(parse("   ").reason(), Some(Reason::Empty));
    assert_eq!(parse("lek").reason(), Some(Reason::NonNumeric));
    assert_ne!(parse("").reason(), parse("lek").reason());
}

#[test]
fn a_manual_entry_sentinel_needs_no_special_case() {
    // A real synthetic id, used by one system to mark a manually added item.
    // It is exactly 14 characters, so every `len == 14` check waves it
    // through. The domain rule catches it with no sentinel list to keep in
    // sync — it is non-numeric, and that is enough.
    let sentinel = "MANUAL-a1b2c3d";
    assert_eq!(sentinel.len(), 14);
    assert_eq!(parse(sentinel).reason(), Some(Reason::NonNumeric));
    assert_eq!(key(sentinel), None);
    assert_eq!(store_form(sentinel), sentinel);
}

#[test]
fn padding_preserves_the_check_digit() {
    // Zeroing it would rescue one scan across 8,976 and merge 134 genuinely
    // distinct products. Pad only.
    assert_eq!(store_form("10700021526"), "00010700021526");
    assert_eq!(store_form("7350053850019"), "07350053850019");
    assert_eq!(store_form("12345670"), "00000012345670");
}

#[test]
fn cleaning_is_recorded_not_hidden() {
    let p = parse(" 7350053850019\r");
    assert_eq!(p.gtin().unwrap().as_str(), "07350053850019");
    let cleaning = p.cleaning().unwrap();
    assert!(cleaning.removed_whitespace());
    assert!(!cleaning.is_pristine());

    let dashed = parse("7-350-053-850-019");
    assert_eq!(dashed.gtin().unwrap().as_str(), "07350053850019");
    assert!(dashed.cleaning().unwrap().removed_separators());

    let pristine = parse("07350053850019");
    assert!(pristine.cleaning().unwrap().is_pristine());
}

#[test]
fn float_debris_drops_the_fraction_rather_than_scraping_digits() {
    // Deleting the '.' instead would give "73186901234560" — fourteen digits,
    // perfectly well-formed, and a different product.
    let p = parse("7318690123456.0");
    assert_eq!(p.gtin().unwrap().as_str(), "07318690123456");
    assert!(p.cleaning().unwrap().dropped_fraction());

    assert_eq!(
        parse("7318690123456,0").gtin().unwrap().as_str(),
        "07318690123456"
    );

    // A real fraction is still debris — a GTIN has no fractional part — but
    // it is never scraped into the number.
    assert_eq!(parse("1.5").reason(), Some(Reason::TooShort { digits: 1 }));
}

#[test]
fn digits_are_not_scraped_out_of_junk() {
    // Stricter than the implementations barkod replaces, which deleted every
    // non-digit unconditionally and would have produced a GTIN-8 here.
    assert_eq!(parse("abc12345678").reason(), Some(Reason::NonNumeric));
    assert_eq!(
        parse("EAN 7350053850019").reason(),
        Some(Reason::NonNumeric)
    );
    assert_eq!(parse("1.2.3").reason(), Some(Reason::NonNumeric));
    assert_eq!(parse("7350053850019.").reason(), Some(Reason::NonNumeric));
    assert_eq!(parse(".5").reason(), Some(Reason::NonNumeric));
}

#[test]
fn a_dash_only_cell_is_missing_not_junk() {
    // "-" is a placeholder for "no value" in several of the source exports.
    assert_eq!(parse("-").reason(), Some(Reason::Empty));
}

#[test]
fn key_ignores_the_check_digit_and_nothing_else() {
    let good = key("07390525907745").unwrap();
    let typo = key("07390525907741").unwrap();
    assert_eq!(good, typo);
    assert_eq!(good.as_key_str(), "07390525907740");

    // A different thirteenth digit is a different product.
    let other = key("07390525907845").unwrap();
    assert_ne!(good, other);
}
