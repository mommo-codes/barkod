//! Invariants that must hold for every input, checked against generated ones.
//!
//! Hand-written cases prove the decided answers; these prove the shape of the
//! whole space. Both are needed — a property suite alone would happily pass
//! on a consistent wrong rule.

use barkod::{key, parse, store_form, Encoding};
use proptest::prelude::*;

/// Any string at all, including ones no spreadsheet would ever produce.
fn any_input() -> impl Strategy<Value = String> {
    prop_oneof![
        3 => "[0-9]{0,20}",
        2 => "[0-9 .,\\-]{0,20}",
        1 => ".{0,20}",
        1 => "\\PC{0,12}",
    ]
}

/// Strings that are inside the domain by construction.
fn in_domain() -> impl Strategy<Value = String> {
    "[0-9]{8,14}"
}

proptest! {
    /// Nothing panics, whatever arrives. Cells hold anything.
    #[test]
    fn parse_never_panics(input in any_input()) {
        let parsed = parse(&input);
        let _ = parsed.is_gtin();
        if let Some(g) = parsed.gtin() {
            // Touch every derived answer, including the two table lookups
            // whose totality is the thing being tested.
            let _ = (g.encoding(), g.allocation(), g.indicator(), g.shortest());
            let _ = (g.check_digit_is_valid(), g.expected_check_digit());
            let _ = (g.looks_like_internal_code(), g.measure_field_looks_zeroed());
            let _ = g.with_recomputed_check_digit();
        }
    }

    /// The raw input survives, always. It is the only handle a bad row has.
    #[test]
    fn raw_is_never_lost(input in any_input()) {
        prop_assert_eq!(parse(&input).raw(), &input);
    }

    /// Never silently blank: a non-empty input never stores as empty.
    #[test]
    fn store_form_never_blanks(input in any_input()) {
        prop_assume!(!input.is_empty());
        prop_assert!(!store_form(&input).is_empty());
    }

    /// Storing is idempotent — canonicalising a canonical value changes
    /// nothing, so a value cannot drift by being written twice.
    #[test]
    fn store_form_is_idempotent(input in any_input()) {
        let once = store_form(&input).into_owned();
        let twice = store_form(&once).into_owned();
        prop_assert_eq!(once, twice);
    }

    /// Inside the domain, storage is exactly zero-padding to fourteen.
    #[test]
    fn in_domain_storage_is_pad_only(digits in in_domain()) {
        let stored = store_form(&digits);
        prop_assert_eq!(stored.len(), 14);
        prop_assert!(stored.ends_with(&digits));
        prop_assert!(stored[..14 - digits.len()].bytes().all(|b| b == b'0'));
    }

    /// The shortest form round-trips: shrinking then re-reading gives back
    /// the same canonical value. This is what makes shrink safe to export.
    #[test]
    fn shortest_round_trips(digits in in_domain()) {
        let g = parse(&digits).gtin().unwrap();
        let back = parse(g.shortest()).gtin().unwrap();
        prop_assert_eq!(g, back);
    }

    /// The shortest form is always a standard width, and always the width its
    /// encoding claims.
    #[test]
    fn shortest_is_always_a_standard_width(digits in in_domain()) {
        let g = parse(&digits).gtin().unwrap();
        prop_assert!(matches!(g.shortest().len(), 8 | 12 | 13 | 14));
        prop_assert_eq!(g.shortest().len(), g.encoding().digits());
        prop_assert!(matches!(
            g.encoding(),
            Encoding::Gtin8 | Encoding::Gtin12 | Encoding::Gtin13 | Encoding::Gtin14
        ));
    }

    /// A recomputed check digit always validates.
    #[test]
    fn recomputed_check_digit_validates(digits in in_domain()) {
        let g = parse(&digits).gtin().unwrap();
        prop_assert!(g.with_recomputed_check_digit().check_digit_is_valid());
    }

    /// Two values agree on their key exactly when their first thirteen
    /// canonical digits agree — no more, no less.
    #[test]
    fn key_ignores_only_the_check_digit(a in in_domain(), b in in_domain()) {
        let (ga, gb) = (parse(&a).gtin().unwrap(), parse(&b).gtin().unwrap());
        let same_body = ga.as_str()[..13] == gb.as_str()[..13];
        prop_assert_eq!(ga.key() == gb.key(), same_body);
    }

    /// Every key ends in '0' and is fourteen characters.
    #[test]
    fn key_shape(digits in in_domain()) {
        let k = key(&digits).unwrap();
        prop_assert_eq!(k.as_key_str().len(), 14);
        prop_assert!(k.as_key_str().ends_with('0'));
    }

    /// The domain rule, stated independently of the implementation: a purely
    /// numeric input is a GTIN precisely when it has 8 to 14 digits.
    #[test]
    fn domain_boundary_is_exact(digits in "[0-9]{0,25}") {
        let expected = (8..=14).contains(&digits.len());
        prop_assert_eq!(parse(&digits).is_gtin(), expected);
        prop_assert_eq!(key(&digits).is_some(), expected);
    }

    /// The two refusals never disagree about *whether* to refuse, only about
    /// how: a key exists exactly when the value is in the domain, and outside
    /// it storage hands back the input untouched.
    #[test]
    fn refusals_agree_on_the_domain(input in any_input()) {
        let parsed = parse(&input);
        prop_assert_eq!(parsed.is_gtin(), key(&input).is_some());
        if parsed.is_gtin() {
            prop_assert_eq!(store_form(&input).len(), 14);
        } else {
            let stored = store_form(&input).into_owned();
            prop_assert_eq!(stored, input.clone());
        }
    }
}
