# @mommo-codes/barkod

GTIN parsing, canonicalisation, match keys and GS1 classification. Written in
Rust, compiled to WebAssembly — the same implementation the Rust crate and the
Python package run, so all three give the same answers.

Scoped because npm rejects the bare name `barkod` as too similar to `marked`.
The [crate](https://crates.io/crates/barkod) and the
[Python package](https://pypi.org/project/barkod/) are unscoped.

```sh
npm install @mommo-codes/barkod
```

The scoped package starts at 0.1.1; there is no 0.1.0 on npm.

```ts
import * as barkod from "@mommo-codes/barkod";

await barkod.init(); // loads the wasm once, at start-up

barkod.storeForm("7350053850019"); // "07350053850019" — padded, check digit kept
barkod.storeForm("lek");           // "lek"            — untouched, never blanked
barkod.key("7350053850019");       // "07350053850010"
barkod.key("073905259077450");     // undefined        — 15 digits is not a GTIN

const p = barkod.parse("370245100000000123");
p.isGtin;  // false
p.reason;  // "too_long"
p.message; // "More than 14 digits — not a GTIN"
p.raw;     // "370245100000000123" — always kept
```

Every function throws until `init()` resolves, rather than returning a
placeholder. A library that quietly answered wrongly while warming up would be
a worse version of the bug this one exists to remove.

## The domain

Every function agrees that a GTIN is **8 to 14 digits** and differs only on
what it does inside that range. Outside it, `storeForm` hands the input back
untouched and `key` returns `undefined`. A lossy transformation is not a
licence to be more permissive about what counts as a GTIN.

## The key is not a GTIN

`key()` returns a branded `GtinKey`, not a plain `string`. It will not assign
into a `Gtin14String`, so the compiler refuses the mistake this library was
written to end: writing a match key into a column that holds GTINs. Roughly
nine in ten keys end in a check digit no product has.

```ts
const k = barkod.key("7390525907745");
const g: barkod.Gtin14String = k; // ✗ does not compile
```

The brand is compile-time only, so it protects typed slots rather than untyped
ones — type your GTIN columns and the guard works.

Full documentation: <https://github.com/mommo-codes/barkod>

MIT licensed.
