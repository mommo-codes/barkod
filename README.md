# barkod

One implementation of what a GTIN is, callable from **Rust**, **Python** and
**TypeScript**.

Not a barcode library. barkod reads a string of digits and answers questions
about it: is this a GTIN, what is its canonical form, what key should it match
on, and what has GS1 allocated its range for. It never renders a barcode,
never reads a database and never looks anything up.

```rust
use barkod::{parse, store_form, key};

store_form("7350053850019");          // "07350053850019"  — padded, check digit kept
store_form("lek");                    // "lek"             — untouched, never blanked
key("7350053850019");                 // Some(GtinKey("07350053850010"))
key("073905259077450");               // None              — 15 digits is not a GTIN

parse("370245100000000123").reason(); // Some(TooLong { digits: 18 })
```

## Why it exists

It was written to end an argument between seven independent implementations of
these rules across two codebases — four in Python, two in TypeScript, one in
Apps Script. They disagreed, and the disagreements were never visible as
failures. They were visible as products enriched from the wrong row, as 169
rows that silently would not join, as a whole export column of fabricated
check digits, and as a 100% match rate on files that had not matched.

barkod is the result of auditing all seven against live data, compiled once
and shipped to each of them. Every rule below is forced by something that was
measured, not chosen for elegance.

## The law

**Every function agrees on the domain and differs only on the transformation
within it.**

The domain is **8 to 14 digits** — GTIN-8 is the shortest real encoding,
GTIN-14 the canonical form. Outside it, no GTIN operation is defined. Inside
it, transformations diverge freely: pad-only for the canonical form,
check-digit-zeroed for the match key, leading-zeros-dropped for the shortest
form.

A lossy transformation is **not** a licence to be more permissive about what
counts as a GTIN. The match key discards the check digit so two spellings of
one product match despite a wrong or missing one — which only means anything
for a value that *has* a check digit. Lossiness is a property of the
transformation, never of the domain.

The two refusals differ, and that difference is also the law:

| Kind | Outside the domain | Why |
|---|---|---|
| Storage form | the raw input, untouched | a stored value must never lose its only handle |
| Match key | nothing | a key that does not exist must not match anything |

### Four things that follow

**Over-length is a classification, not an error and not a padded GTIN.**
Nothing in a value distinguishes a padded GTIN-8 from a 16-digit internal
code, and an 18-digit SSCC's last 14 digits form a well-formed GTIN-14 that
identifies nothing. Truncating manufactures a valid-looking wrong answer, so
barkod reports the length and hands the value back.

**Classify, never reject.** An invalid check digit does not mean "not a
GTIN". In one live dataset, 109 rows of a product register and 2,429 rows of a
product catalogue were real products that fail GS1 Mod-10 by design —
variable-weight in-store codes with the measure field zeroed, and internal
PLUs. A validity gate would refuse real inventory.

**Never silently blank.** `store_form("lek")` is `"lek"`, not `""`. Blanking
junk is how it becomes indistinguishable from missing data — and `Empty` and
`NonNumeric` are separate answers for the same reason.

**The match key is a distinct type.** Not a `String`, not a `str`, not a
`string`. See below.

## What the match key is for

Not typo tolerance. One specific, measured situation: retailers send
weight-based items — deli, bakery, loose produce — with the final digit wrong,
usually forced to `0`, while the venue's own assortment carries the same
thirteen leading digits with the correct check digit. Raw equality never
matches the two, so a product the venue already stocks looks new.

Keying **both sides** to `first thirteen + '0'` makes them meet. Measured on
one real pair of retail assortment files — 11,584 rows in a venue's current
assortment against 12,317 in the incoming list:

| | |
|---|---|
| Incoming rows with an invalid check digit | 389 |
| …in the restricted-circulation range (variable weight) | **all 389** |
| …matched to a current product **only** via the key | **210** |
| Valid incoming rows whose key hit a *different* product | **0** |
| Distinct current GTINs sharing a key | **0** |

The zeroing is unconditional, and has to be: zeroing only when the check digit
fails would leave the venue side on `base + c` and the retailer side on
`base + 0`, and they would no longer meet.

Two *valid* GTINs can never collide, because the check digit is a function of
the first thirteen digits — so any collision needs a member with an invalid
check digit, which is the population the key is for. There is one known
exception, in-store PLU codes, documented on `GtinKey` along with why it is
deliberately not guarded against.

## The key is still not a GTIN

`GtinKey` identifies nothing. It is a bucket, and roughly nine in ten of them
end in a check digit no product has. Writing one into a file publishes a
plausible, fabricated identifier — which is exactly what one product export
did for its whole life, under a column heading that read as a GTIN.

So the type refuses to help you. In Rust it has no `Display`, no `Deref`, no
`Into<String>`, no way back to a `Gtin14` and no comparison with one; the only
route to a string is `as_key_str()`. In Python it has no `__str__`, so
`f"{key}"` prints `GtinKey('07390525907740')`. In TypeScript it is a branded
string that will not assign into a `Gtin14String`.

```python
key = barkod.key("7390525907745")
f"{key}"            # "GtinKey('07390525907740')"   ← visibly wrong in an export
key.as_key_str()    # '07390525907740'              ← you said it out loud
```

## What it can tell you

```rust
let g = parse("02011030000000").gtin().unwrap();

g.as_str();                      // "02011030000000"  canonical
g.shortest();                    // "2011030000000"   shortest standard form
g.encoding();                    // Gtin13
g.check_digit_is_valid();        // false
g.expected_check_digit();        // 5
g.allocation();                  // RestrictedCirculationRegion
g.allocation().description();    // "Restricted Circulation Numbers within a geographic region"
g.indicator();                   // None — a padded value has no indicator digit
g.measure_field_looks_zeroed();  // true
```

Classification is split in two, on purpose:

- **`Encoding` and `Allocation` are definitional.** Transcribed from the GS1
  General Specifications 23.0, Figures 1.4.2-1 and 1.4.3-1, one arm per row of
  the table. Both are *total* — every value maps to exactly one variant, so
  neither can return a non-answer. GTIN-8 gets its own table, because GS1
  publishes one.
- **`looks_like_internal_code` and `measure_field_looks_zeroed` are guesses.**
  Patterns observed in production, named `looks_like` rather than `is`. This
  is why there is no `VariableWeight` or `InternalPlu` classification: those
  names assert something the data cannot prove.

## Install

```sh
cargo add barkod        # Rust
pip install barkod      # Python — abi3 wheels, 3.8+
npm install barkod      # TypeScript — WebAssembly, ~40KB
```

The TypeScript build loads wasm, so it needs one `await init()` at start-up.
Every function throws until that resolves rather than returning a placeholder.

```ts
import * as barkod from "barkod";
await barkod.init();
barkod.storeForm("7350053850019"); // "07350053850019"
```

## Layout

| Crate | What it is |
|---|---|
| [`barkod/`](barkod) | The core. Zero dependencies, no `unsafe`, no data, no I/O. |
| [`barkod-py/`](barkod-py) | PyO3/maturin bindings. |
| [`barkod-wasm/`](barkod-wasm) | wasm-bindgen bindings plus the branded TypeScript wrapper. |
| [`conformance/`](conformance) | Exports a corpus so a port of these rules elsewhere can be pinned to this crate. |

The core does not depend on PyO3 or wasm-bindgen, and CI asserts that its
dependency tree stays empty.

## Testing

```sh
cargo test --workspace                 # unit, property, doc, production fixtures
cargo clippy --workspace --all-targets -- -D warnings
( cd barkod-py && maturin develop && pytest tests/ )
( cd barkod-wasm/npm && npm test )     # build + typecheck + node
cargo +nightly fuzz run parse fuzz/corpus/parse fuzz/seeds/parse  # arbitrary input
```

Property tests assert the invariants over generated input: that nothing
panics, that the raw input is never lost, that a non-empty value never stores
as blank, that storing is idempotent, that the shortest form round-trips back
to the same canonical value, and that two values share a key exactly when
their first thirteen digits agree.

The fuzzer runs `parse` over arbitrary bytes and **checks every one of those
invariants on every input**, rather than only looking for crashes — a target
that asked "did it crash?" would pass on a `parse` that refused everything.
Its seed corpus is 470 real values: register and catalogue rows, both sides of
live assortment files, and the encoding debris a spreadsheet export produces
(byte-order marks, zero-width spaces, Arabic-Indic digits, a stray NUL).

Fixtures are real production values. Every check digit in them was verified
against an independent implementation rather than assumed correct — the first
draft of that test asserted four "known-good" GTINs and two of them were not.

Each suite has been watched failing on purpose — the fuzzer was run against a
deliberately broken `shortest()` to confirm it catches it, and the conformance
corpus against a deliberately broken key. A green run means something only if
you have seen the red one.

## Further reading

- [docs/no-data.md](docs/no-data.md) — why there are no registries here, and
  where E-numbers, additives, country codes and GPC belong instead
- [docs/not-a-primary-key.md](docs/not-a-primary-key.md) — read this one
- [conformance/README.md](conformance/README.md) — pinning a port of these
  rules to this crate, and what that does and does not prove

MIT licensed.
