//! Fuzz `parse` over arbitrary bytes, and check the invariants on every one.
//!
//! `parse` is the crate's only entry point and it is fed spreadsheet cells,
//! which hold anything at all: a byte-order mark, a right-to-left override, a
//! lone surrogate that survived a bad export, ten thousand digits. The
//! property suite covers the space it was told to generate; this covers the
//! space nobody thought of.
//!
//! **It does not just look for panics.** A target that only checks "did not
//! crash" would pass on a `parse` that returned `Empty` for everything. Every
//! invariant the library actually promises is asserted here, so the fuzzer is
//! searching for a *wrong answer* as much as for a crash.
//!
//! The crate is `#![forbid(unsafe_code)]`, so memory-safety bugs are not the
//! target. What is: index arithmetic on a non-ASCII boundary, arithmetic
//! overflow on absurd lengths, and the `debug_assert!` in `allocation.rs`
//! that claims a GS1 prefix range is unreachable. Fuzz builds keep debug
//! assertions on, so that claim is under test rather than merely written down.
//!
//! Run:
//!   cargo +nightly fuzz run parse fuzz/corpus/parse fuzz/seeds/parse
//!
//! `fuzz/seeds/parse` is a curated, committed set of real values; the working
//! corpus goes to `fuzz/corpus/parse`, which is gitignored. libFuzzer writes
//! new units to the *first* directory listed, so the order matters — passing
//! only the seeds directory makes the fuzzer write its findings into it and
//! the curated set stops being curated.

#![no_main]

use barkod::{key, parse, store_form};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Cells are text. Non-UTF-8 bytes never reach `parse`, so feeding them
    // would only fuzz `from_utf8`.
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };

    let parsed = parse(input);

    // The raw input survives, always. It is the only handle a bad row has,
    // and every caller relies on being able to show the user their own cell.
    assert_eq!(parsed.raw(), input, "raw input was not preserved");

    // Never silently blank: a non-empty input never stores as empty.
    let stored = store_form(input);
    assert!(
        input.is_empty() || !stored.is_empty(),
        "non-empty input {input:?} stored as empty"
    );

    // Storing is idempotent, so a value cannot drift by being written twice.
    assert_eq!(
        store_form(&stored),
        stored,
        "store_form not idempotent for {input:?}"
    );

    // The two refusals agree on *whether* to refuse, differing only in how.
    assert_eq!(
        parsed.is_gtin(),
        key(input).is_some(),
        "storage and key disagree about the domain for {input:?}"
    );

    // `Parsed` is `#[non_exhaustive]`, so this goes through the accessors
    // rather than a match — a wildcard arm would silently swallow any variant
    // added later, which is exactly what a fuzz target must not do.
    match parsed.gtin() {
        None => {
            // A refusal always has something to show a user…
            let reason = parsed.reason().expect("a non-GTIN must carry a reason");
            assert!(!reason.message().is_empty());
            // …and hands the value back untouched.
            assert_eq!(stored, input);
        }
        Some(value) => {
            assert!(parsed.reason().is_none(), "a GTIN must not carry a reason");
            let canonical = value.as_str();
            assert_eq!(canonical.len(), 14, "canonical form must be 14 chars");
            assert!(canonical.bytes().all(|b| b.is_ascii_digit()));
            assert_eq!(stored, canonical);

            // The key is a pure function of the first thirteen digits, and
            // that is what the whole variable-weight matching technique
            // rests on. See the `GtinKey` docs.
            let k = value.key();
            let key_str = k.as_key_str();
            assert_eq!(key_str.len(), 14);
            assert!(key_str.ends_with('0'));
            assert_eq!(&key_str[..13], &canonical[..13]);

            // The shortest form is a suffix of the canonical form, always a
            // standard width, and always the width its encoding claims.
            let short = value.shortest();
            assert!(canonical.ends_with(short));
            assert!(matches!(short.len(), 8 | 12 | 13 | 14));
            assert_eq!(short.len(), value.encoding().digits());

            // …and it round-trips: shrinking then re-reading gives back the
            // same canonical value. This is what makes shrink safe to export.
            assert_eq!(
                parse(short).gtin(),
                Some(value),
                "shortest form did not round-trip for {input:?}"
            );

            // Check digits are digits, and a recomputed one always validates.
            assert!(value.check_digit() <= 9);
            assert!(value.expected_check_digit() <= 9);
            assert_eq!(
                value.check_digit_is_valid(),
                value.check_digit() == value.expected_check_digit()
            );
            assert!(value.with_recomputed_check_digit().check_digit_is_valid());

            // Classification is total: neither of these may fail to answer,
            // and `allocation` carries a debug_assert that a prefix range is
            // unreachable. Calling it on every input is what tests that.
            let _ = value.allocation().description();
            let _ = value.allocation().is_restricted_circulation();
            let _ = value.looks_like_internal_code();
            let _ = value.measure_field_looks_zeroed();

            // The indicator digit exists only when there is one to read.
            assert_eq!(
                value.indicator().is_some(),
                value.significant_digits() == 14
            );
        }
    }
});
