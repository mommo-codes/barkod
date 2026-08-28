"""The binding's own tests.

The GTIN rules are tested once, in Rust. What is tested here is the boundary:
type coercion from what Python columns actually hold, and whether the GtinKey
distinction survived the crossing.
"""

import pytest

import barkod


class TestTheThreeVerbs:
    def test_store_form_pads_and_preserves_the_check_digit(self):
        assert barkod.store_form("7350053850019") == "07350053850019"
        assert barkod.store_form("10700021526") == "00010700021526"

    def test_store_form_hands_back_anything_out_of_domain(self):
        for value in ["lek", "MANUAL-a1b2c3d", "1234567", "073905259077450"]:
            assert barkod.store_form(value) == value

    def test_key_refuses_outside_the_domain(self):
        assert barkod.key("7390525907745").as_key_str() == "07390525907740"
        assert barkod.key("073905259077450") is None
        assert barkod.key("lek") is None
        assert barkod.key("") is None

    def test_parse_reports_why(self):
        assert barkod.parse("").reason == "empty"
        assert barkod.parse("lek").reason == "non_numeric"
        assert barkod.parse("1234567").reason == "too_short"
        assert barkod.parse("370245100000000123").reason == "too_long"
        assert barkod.parse("370245100000000123").digits == 18

    def test_raw_survives(self):
        assert barkod.parse("  lek  ").raw == "  lek  "


class TestCoercion:
    """What a GTIN column actually contains when you read it."""

    def test_none_is_missing_not_junk(self):
        assert barkod.parse(None).reason == "empty"
        assert barkod.store_form(None) == ""

    def test_int_is_exact_at_any_size(self):
        assert barkod.store_form(7350053850019) == "07350053850019"
        # Past ~15 digits a float would have corrupted this silently.
        assert barkod.parse(370245100000000123).reason == "too_long"
        assert barkod.parse(370245100000000123).digits == 18

    def test_float_debris_is_dropped_not_scraped(self):
        # The dangerous case: deleting the '.' gives "73186901234560",
        # fourteen digits and a different product.
        parsed = barkod.parse(7318690123456.0)
        assert parsed.gtin.value == "07318690123456"
        assert parsed.dropped_fraction is True

    def test_float_debris_as_a_string_behaves_identically(self):
        assert barkod.parse("7318690123456.0").gtin.value == "07318690123456"

    def test_a_float_too_large_to_write_out_is_refused_not_corrupted(self):
        # Precision is already lost by the time barkod sees it; refusing is
        # the honest answer. Read the column as text instead.
        assert barkod.parse(3.70245100000000012e17).reason == "non_numeric"

    def test_unsupported_types_raise(self):
        with pytest.raises(TypeError):
            barkod.parse(["7350053850019"])


class TestTheKeyDistinctionSurvives:
    def test_key_is_not_a_string(self):
        key = barkod.key("7390525907745")
        assert not isinstance(key, str)

    def test_formatting_a_key_shows_it_is_a_key(self):
        key = barkod.key("7390525907745")
        # This is the guard: a key that leaks into an export is visibly wrong
        # rather than a plausible fabricated GTIN.
        assert f"{key}" == "GtinKey('07390525907740')"
        assert str(key) == "GtinKey('07390525907740')"
        assert key.as_key_str() == "07390525907740"

    def test_keys_hash_and_compare(self):
        a = barkod.key("07390525907745")
        b = barkod.key("07390525907741")
        assert a == b
        assert len({a, b}) == 1


class TestClassification:
    def test_real_production_values(self):
        plu = barkod.parse("00000000014150").gtin
        assert plu.looks_like_internal_code is True

        weight = barkod.parse("02011030000000").gtin
        assert weight.measure_field_looks_zeroed is True
        assert weight.allocation == "restricted_circulation_region"
        assert weight.is_restricted_circulation is True
        assert weight.check_digit_is_valid is False

        book = barkod.parse("09789137163666").gtin
        assert book.allocation == "isbn"
        assert book.encoding == "GTIN-13"

        variable = barkod.parse("91599903387551").gtin
        assert variable.indicator == "variable_measure"
        assert variable.indicator_digit == 9

    def test_invalid_check_digit_is_still_a_gtin(self):
        parsed = barkod.parse("07390525907745")
        assert parsed.is_gtin
        assert parsed.gtin.check_digit_is_valid is False
        assert parsed.gtin.expected_check_digit == 2


class TestBatch:
    def test_store_form_many(self):
        assert barkod.store_form_many(["7350053850019", None, "lek"]) == [
            "07350053850019",
            "",
            "lek",
        ]

    def test_key_strings_has_holes_where_there_is_no_key(self):
        assert barkod.key_strings(["7390525907745", "lek", None]) == [
            "07390525907740",
            None,
            None,
        ]

    def test_parse_many(self):
        results = barkod.parse_many(["7350053850019", "lek"])
        assert results[0].is_gtin
        assert results[1].reason == "non_numeric"

    def test_batch_handles_mixed_column_types(self):
        assert barkod.store_form_many([7350053850019, 7318690123456.0, "lek", None]) == [
            "07350053850019",
            "07318690123456",
            "lek",
            "",
        ]
