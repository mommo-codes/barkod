"""Generate a conformance corpus for a port of these rules.

Some runtimes can load neither a wasm module nor a native Python extension —
Google Apps Script, running inside a spreadsheet, is the case this was written
for. There, a hand-written port is the only option, and it is a drift risk
from the day it is written.

This corpus is what keeps such a port honest: generated here, checked into the
port's repository, replayed by a test on that side. Change either
implementation and that test fails.

**What it proves is agreement, not correctness.** barkod is the source of
truth for the expected values, so the corpus cannot tell you barkod is right;
it can only tell you the two have not drifted apart. The correctness argument
lives in `barkod/tests/`, and the replaying test should carry its own
hand-written cases alongside the replayed ones for the same reason.

Only the two functions barkod can speak for as a whole operation are pinned:

- `getGtinKey` — the join key, and the one that matters.
- `computeCheckDigit` — pure, and identical on both sides already.

A port's internal cleaning helper is deliberately **not** pinned. Ports tend
to have one that scrapes every non-digit out of its input, which barkod does
not do, so pinning it would write a disagreement into the contract as though
it were the agreement. Nor is anything that truncates over-length input and
recomputes the digit: that is a latent defect, not a behaviour to enshrine.

Run: python conformance/export_corpus.py > corpus.json
"""

from __future__ import annotations

import json
import sys

import barkod


def key_of(value: str) -> str | None:
    key = barkod.key(value)
    return key.as_key_str() if key else None


def check_digit_for(base13: str) -> str:
    """The GS1 check digit that should terminate a 13-digit base."""
    gtin = barkod.parse(base13 + "0").gtin
    return str(gtin.expected_check_digit)


KEY_INPUTS: list[str] = [
    # ── the domain boundary, every length ───────────────────────────────
    *["7" * n for n in range(0, 21)],
    *["0" * n for n in range(1, 18)],
    # ── the variable-weight rescue pattern, both sides of real pairs ────
    # The retailer sends the final digit zeroed; the venue's assortment holds
    # the real one. Both must produce the same key or the match stops working.
    "02319228400000", "02319228400003",
    "02320074500000", "02320074500007",
    "02319203800000", "02319203800002",
    "02319100900000", "02319100900003",
    "2319228400000", "2319203800000",    # as they arrive: 13 digits, unpadded
    # ── junk and sentinels: the bug a domain check closes ───────────────
    "", "   ", "-", "lek", "MANUAL-a1b2c3d", "MANUAL-a1", "abc12345678",
    "0", "00", "1", "1234567", "EAN 7350053850019",
    # ── over-length: must not truncate into a real product's key ────────
    "07390525907745", "073905259077450", "370245100000000123",
    "0000000012345670",
    # ── cleaning: decoration is stripped, everything else is not a GTIN ─
    " 7350053850019 ", "7350053850019\r", "7-350-053-850-019",
    "7318690123456.0", "7318690123456,0", "1.5", "1.2.3", "7350053850019.",
    # ── ordinary values ─────────────────────────────────────────────────
    "7350053850019", "07350053850019", "10700021526", "00010700021526",
    "12345670", "00000012345670", "073905259077",
    # ── internal PLU codes: these share a key, and that is known ────────
    # The final digit discriminates products rather than checking them.
    "00000000081010", "00000000081011", "00000000014150", "00000000014151",
]

CHECK_DIGIT_BASES: list[str] = [
    "0735005385001",  # 07350053850019, valid
    "0231922840000",  # a variable-weight base
    "0232007450000",
    "0211280000000",
    "0739052590774",
    "0000000000000",
    "0001070002152",
    "9789137163666"[:13],
]


def main() -> None:
    corpus = {
        "_comment": (
            "Generated from barkod by conformance/export_corpus.py in "
            "github.com/mommo-codes/barkod. Do not hand-edit. See the "
            "header of that file for what this proves and what it does not."
        ),
        "_source": f"barkod {getattr(barkod, '__version__', '0.1.0')}",
        "getGtinKey": [{"in": v, "out": key_of(v)} for v in KEY_INPUTS],
        "computeCheckDigit": [
            {"in": b, "out": check_digit_for(b)} for b in CHECK_DIGIT_BASES
        ],
    }
    json.dump(corpus, sys.stdout, indent=2, ensure_ascii=False)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
