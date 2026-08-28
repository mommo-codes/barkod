//! Every value here was read out of a live retail database.
//!
//! Not invented, not adapted. The classifications barkod ships exist because
//! rows in these shapes exist, and the counts in the comments are what the
//! tables held when the library was designed. Two reference tables are cited
//! throughout: a **product register** of 187,132 numeric rows, and a
//! **product catalogue** of 14,803,000 rows spanning five markets.
//!
//! Synthetic fixtures would have been weaker. Several of these tests exist
//! because a real value did something the specification alone would not have
//! led anyone to write a case for.

use barkod::{parse, Allocation, Encoding, Indicator};

fn gtin(input: &str) -> barkod::Gtin14 {
    parse(input)
        .gtin()
        .expect("fixture should be in the domain")
}

#[test]
fn internal_plu_codes() {
    // 101 of the register's 187,132 numeric rows have fewer significant
    // digits than GTIN-8 can express, so no GS1 member organisation issued
    // them. They are in-store PLU codes for counter items.
    for raw in [
        "00000000000420",
        "00000000000450",
        "00000000000672",
        "00000000014150",
    ] {
        let g = gtin(raw);
        assert!(g.looks_like_internal_code(), "{raw} should look internal");
        assert!(g.significant_digits() < 8);
        // Still a GTIN-14 as far as storage is concerned — never blanked.
        assert_eq!(g.as_str(), raw);
    }
}

#[test]
fn variable_weight_codes_with_a_zeroed_measure_field() {
    // Weight-sold bakery and deli goods. One market in the catalogue held
    // 2,414 rows with a wrong check digit against roughly 5 in each of the
    // others: the measure field was zeroed after the check digit was
    // computed, so the digit is stale and the product is real.
    for (raw, expected_check) in [
        ("02011030000000", 5),
        ("02115130000000", 9),
        ("02115790000000", 5),
        ("02115830000000", 2),
    ] {
        let g = gtin(raw);
        assert!(
            !g.check_digit_is_valid(),
            "{raw} check digit should be stale"
        );
        assert_eq!(g.expected_check_digit(), expected_check);
        assert_eq!(g.allocation(), Allocation::RestrictedCirculationRegion);
        assert!(g.allocation().is_restricted_circulation());
        assert!(g.measure_field_looks_zeroed());
    }
}

#[test]
fn a_valid_check_digit_in_the_same_range_is_not_flagged() {
    // Another market holds 31,461 rows in the same range with their measure
    // fields intact. The range alone means nothing.
    let g = gtin("02112800000000");
    assert!(g.check_digit_is_valid());
    assert_eq!(g.allocation(), Allocation::RestrictedCirculationRegion);
    assert!(!g.measure_field_looks_zeroed());
}

#[test]
fn books() {
    // 1,568 register rows are in the ISBN/ISSN ranges.
    for raw in ["09789137163666", "09789113133492", "09789180663069"] {
        let g = gtin(raw);
        assert_eq!(g.allocation(), Allocation::Isbn);
        assert!(g.allocation().is_publication_or_coupon());
        assert_eq!(g.encoding(), Encoding::Gtin13);
    }
}

#[test]
fn variable_measure_trade_items() {
    // 5 register rows carry indicator digit 9, which GS1 reserves for trade
    // items whose measure varies per item.
    for raw in [
        "91599903387551",
        "92002511910077",
        "94260052151764",
        "95998200567189",
    ] {
        let g = gtin(raw);
        assert_eq!(g.significant_digits(), 14);
        assert_eq!(g.indicator(), Some(Indicator::VariableMeasure));
        assert_eq!(g.encoding(), Encoding::Gtin14);
    }
}

#[test]
fn padded_values_have_no_indicator() {
    // The leading digit of a padded GTIN-13 is padding. Reading it as an
    // indicator would report "indicator 0" for four million catalogue rows.
    assert_eq!(gtin("09789137163666").indicator(), None);
    assert_eq!(gtin("00010700021526").indicator(), None);
}

#[test]
fn nine_ten_and_eleven_significant_digits_shrink_to_gtin12() {
    // 1,363 register rows. Those widths are not GTIN encodings, so the
    // shortest standard form keeps the zeros that reach twelve.
    for raw in ["00055653670209", "00041143025826", "00021723339994"] {
        let g = gtin(raw);
        assert_eq!(g.significant_digits(), 11);
        assert_eq!(g.encoding(), Encoding::Gtin12);
        assert_eq!(g.shortest().len(), 12);
        assert_eq!(g.shortest(), &raw[2..]);
    }
}

#[test]
fn gtin8_uses_its_own_prefix_table() {
    // GS1 publishes a separate allocation table for GTIN-8. Classifying an
    // 8-digit value with the 13-digit table gives a confidently wrong answer.
    let issued = gtin("00000010066096");
    assert_eq!(issued.encoding(), Encoding::Gtin8);
    assert_eq!(issued.shortest(), "10066096");
    assert_eq!(issued.allocation(), Allocation::Gtin8Issuance);

    let in_store = gtin("00000020130435");
    assert_eq!(in_store.encoding(), Encoding::Gtin8);
    assert_eq!(
        in_store.allocation(),
        Allocation::RestrictedCirculationCompany
    );
    assert!(in_store.allocation().is_restricted_circulation());
}

#[test]
fn no_upce_present_in_the_measured_data() {
    // An 8-digit value is ambiguous: EAN-8 and the zero-suppressed UPC-E are
    // both 8 digits, and they expand to different 12-digit numbers. Padding a
    // UPC-E as though it were an EAN-8 would be a silent wrong answer.
    //
    // Measured before the rule was settled: of 3,640 eight-significant
    // register rows only 2 carry a UPC-E number system (0 or 1), and both
    // validate as GTIN-8. So padding an 8-digit value to GTIN-14 is safe for
    // this data.
    //
    // This test is a tripwire, not a proof. A UPC-E would arrive as an
    // 8-digit value starting 0 or 1 whose GTIN-8 check digit fails, and these
    // two rows are the ones to re-measure against.
    for raw in ["00000010066096", "00000010066140"] {
        let g = gtin(raw);
        assert!(g.check_digit_is_valid(), "{raw} validates as a GTIN-8");
    }
}

#[test]
fn the_variable_weight_rescue_pattern() {
    // What the match key is for. A retailer sends the item with its final
    // digit forced to 0; the venue's assortment carries the same thirteen
    // leading digits with the correct check digit. Raw equality never matches
    // them; the key does.
    //
    // Measured on one real pair of assortment files: 389 of 12,317 incoming
    // rows had an invalid check digit, every one of them in the
    // restricted-circulation range, and 210 matched a current product only
    // through this key — each against a partner whose own check digit was
    // valid. Both sides of this pair are real values from those files.
    let from_retailer = gtin("02319228400000"); // final digit zeroed
    let in_assortment = gtin("02319228400003"); // correct check digit

    assert!(!from_retailer.check_digit_is_valid());
    assert!(in_assortment.check_digit_is_valid());
    assert_eq!(from_retailer.expected_check_digit(), 3);

    // Different values, so raw equality fails. Same key, so the match works.
    assert_ne!(from_retailer, in_assortment);
    assert_eq!(from_retailer.key(), in_assortment.key());
    assert_eq!(
        from_retailer.allocation(),
        Allocation::RestrictedCirculationRegion
    );
}

#[test]
fn internal_plu_codes_share_a_key_and_that_is_known() {
    // The documented limitation, pinned so that changing it has to be a
    // decision rather than an accident. These five are five different
    // varieties of loose fruit whose final digit is a product discriminator
    // rather than a check digit, so the key merges them.
    //
    // Deliberately not guarded: this population lives in the register, not in
    // the assortment files the key actually processes, where the measured
    // collision count is zero. See the `GtinKey` documentation.
    let discriminated = [
        "00000000081010",
        "00000000081011",
        "00000000081012",
        "00000000081013",
        "00000000081014",
    ];
    let keys: Vec<_> = discriminated.iter().map(|a| gtin(a).key()).collect();
    assert!(
        keys.windows(2).all(|w| w[0] == w[1]),
        "all five share one key — this is the known limitation"
    );

    // And they are all identifiable as internal codes, which is what a caller
    // joining against a broad reference table would filter on.
    assert!(discriminated
        .iter()
        .all(|a| gtin(a).looks_like_internal_code()));

    // The case no predicate catches: twelve significant digits, two unrelated
    // grocery items, both check digits invalid.
    let first = gtin("00868302706290");
    let second = gtin("00868302706292");
    assert_eq!(first.key(), second.key());
    assert!(!first.looks_like_internal_code());
    assert!(!first.check_digit_is_valid() && !second.check_digit_is_valid());
}

#[test]
fn a_fifteen_digit_row_that_really_exists() {
    // A real over-length value found living in a GTIN column. Before the
    // domain rule, its key collided with whatever product shares its first
    // thirteen digits.
    let raw = "112345678912343";
    assert!(!parse(raw).is_gtin());
    assert_eq!(parse(raw).store_form(), raw);
    assert_eq!(parse(raw).key(), None);
}
