# barkod

GTIN parsing, canonicalisation, match keys and GS1 classification. Written in
Rust; this is the Python binding.

```sh
pip install barkod
```

```python
import barkod

barkod.store_form("7350053850019")   # '07350053850019'  — padded, check digit kept
barkod.store_form("lek")             # 'lek'             — untouched, never blanked
barkod.key("7350053850019")          # GtinKey('07350053850010')
barkod.key("073905259077450")        # None              — 15 digits is not a GTIN

p = barkod.parse("370245100000000123")
p.is_gtin                            # False
p.reason                             # 'too_long'
p.message                            # 'More than 14 digits — not a GTIN'
p.raw                                # '370245100000000123' — always kept
```

## The domain

Every function agrees that a GTIN is **8 to 14 digits** and differs only on
what it does inside that range. Outside it, `store_form` hands the input back
untouched and `key` returns `None`. A lossy transformation is not a licence to
be more permissive about what counts as a GTIN.

## The key is not a GTIN

`key()` returns a `GtinKey`, not a `str`. It has no `__str__`, so printing one
gives `GtinKey('07390525907740')` rather than a bare number — a key that leaks
into an export is visibly wrong instead of a plausible fabricated identifier.
Roughly nine in ten keys end in a check digit no product has.

```python
key = barkod.key("7390525907745")
f"{key}"            # "GtinKey('07390525907740')"
key.as_key_str()    # '07390525907740'  — say it out loud
```

## Columns

`store_form_many`, `key_strings` and `parse_many` take a whole column and
release the GIL for the work. They accept whatever the column actually holds —
`str`, `int`, `float` or `None`.

```python
import polars as pl

df = df.with_columns(
    pl.Series("gtin_key", barkod.key_strings(df["gtin"].to_list()))
)
```

`key_strings` is named for what it does. It is the one place the type
distinction is deliberately dropped, because a dataframe column needs strings,
so the call site has to say so.

## Classify, never reject

An invalid check digit does not mean "not a GTIN". Variable-weight in-store
codes and internal PLUs are live products that fail GS1 Mod-10 by design.

```python
g = barkod.parse("02011030000000").gtin
g.check_digit_is_valid        # False
g.allocation                  # 'restricted_circulation_region'
g.measure_field_looks_zeroed  # True  — a named guess, not a verdict
```

Full documentation: <https://github.com/mommo-codes/barkod>

MIT licensed.
