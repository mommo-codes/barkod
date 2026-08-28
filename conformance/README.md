# conformance/

Exports a corpus of inputs and their expected answers, so an implementation of
these rules that **cannot call barkod** can be pinned to it anyway.

```sh
python export_corpus.py > corpus.json
```

## When you would want this

Most consumers should just call the library — the Rust crate, the Python
wheel, or the WebAssembly build cover nearly everything. But some runtimes
load none of them. Google Apps Script is the case this was written for: it
runs inside a spreadsheet, and it can load neither a wasm module nor a native
extension, so a hand-written port is the only option.

A port is a drift risk from the day it is written. The corpus is what makes
the drift fail loudly: generate it here, check it into the port's repository,
and have that repository's test suite replay it. Change either side and the
test breaks.

## What it proves, and what it does not

**It proves agreement. It cannot prove correctness.**

The expected values are generated from barkod, so the corpus answers "do these
two do the same thing?" and nothing else. If barkod and the port are wrong in
the same way — and a port is the *most likely* place for that, since it is
usually a transliteration — the shared wrong answer is written into the file
as the expected value and every test passes forever.

This is not hypothetical. The predecessor of this file was a parity contract
pinning a Python implementation against its TypeScript port, and it was
structurally blind to a live bug for exactly this reason: the expected values
came from one implementation, so its bug became the specification.

So the correctness argument lives elsewhere, and the replaying repository
should carry its own hand-written cases alongside the replayed ones:

- **`barkod/tests/domain.rs`** — hand-written from the decided behaviour, not
  generated from anything.
- **`barkod/tests/production_fixtures.rs`** — real values with independently
  verified answers.
- **`barkod/tests/properties.rs`** — invariants over generated input.
- **`fuzz/`** — the same invariants over arbitrary bytes.

## What is pinned

Only the two functions barkod can speak for as a whole operation:

- **`getGtinKey`** — the join key, and the one that matters.
- **`computeCheckDigit`** — pure, and the same everywhere already.

A port's internal cleaning helper is deliberately **not** pinned. Ports tend
to have one that strips every non-digit character, which barkod does not do,
so pinning it would write a disagreement into the contract as though it were
the agreement. Likewise anything that truncates over-length input and
recomputes a check digit: that is a latent defect, not a behaviour to enshrine.

## Verify the harness can fail

Before trusting a green replay, break something on purpose and confirm the
replay goes red. A corpus test that has never been seen failing is not
evidence of anything — this one was checked by inverting the final digit of
the key and confirming the replay caught it.
