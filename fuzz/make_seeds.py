"""Write the seed corpus for the `parse` fuzz target.

A fuzzer starting from nothing spends its first million executions
rediscovering that GTINs are digits. Seeding it with real values puts it
straight onto the boundaries that matter — the domain edges, the
restricted-circulation ranges, the sentinels — and it mutates outward from
there.

Every value here came out of live retail data: a product register, a product
catalogue, or one real pair of assortment files. The unicode block at the end
did not; those are the shapes a spreadsheet export produces that nobody writes
a test for.

Run:  python fuzz/make_seeds.py
      python fuzz/make_seeds.py --from-csv ~/Downloads/some_assortment.csv gtin
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import pathlib
import sys

SEEDS: list[str] = [
    # ── domain edges ────────────────────────────────────────────────────
    "", " ", "-", "0", "1", "1234567", "12345678", "123456789012345",
    "7350053850019", "07350053850019", "073905259077", "12345670",
    # ── real register and catalogue values ──────────────────────────────
    "00010700021526", "10700021526",          # a real row, padded and unpadded
    "07390525907745", "073905259077450",      # genuine vs the 15-digit corruption
    "112345678912343",                        # a real over-length register row
    "370245100000000123", "0000000012345670", # an SSCC, and an ambiguous 16
    "lek", "MANUAL-a1b2c3d",                  # the two non-numeric register values
    "02112800000000",                         # restricted circulation, valid
    "02011030000000", "02115130000000",       # measure field zeroed
    "09789137163666", "09789113133492",       # ISBN
    "91599903387551", "92002511910077",       # indicator 9, variable measure
    "00000000014150", "00000000014151",       # internal PLU, discriminator digit
    "00000000081010", "00000000081014",       # five products, one key
    "00868302706290", "00868302706292",       # two products, one key
    "00000010066096", "00000020130435",       # GTIN-8, both prefix tables
    "00055653670209",                         # 11 significant digits
    # ── live assortment files: the variable-weight rescue pattern ───────
    # Each pair is one product: the final digit zeroed, and the real one.
    "2319228400000", "02319228400000", "02319228400003",
    "02320074500000", "02320074500007",
    "02092425100000", "02092425100005",
    "07316190205820", "07316190205829",
    # ── shapes a spreadsheet produces ───────────────────────────────────
    "7318690123456.0", "7318690123456,0", "7350053850019.",
    " 7350053850019 ", "7350053850019\r\n", "7-350-053-850-019",
    "1.5", "1.2.3", "abc12345678", "EAN 7350053850019",
    "7.35005E+12",                            # a number-formatted cell
    # ── encoding oddities: not production, but what a fuzzer needs ──────
    "\ufeff7350053850019",                      # byte-order mark
    "7350053850019\u200b",                      # zero-width space
    "735005385\u00a00019",                      # non-breaking space
    "\u0666\u0667\u0663\u0660\u0660\u0665",  # Arabic-Indic digits
    "735005385001\uff19",                       # fullwidth digit
    "7350053850019\u202e",                      # right-to-left override
    "7350053850019\x00",                        # embedded NUL
    "9" * 300,                                # absurd length
    "0" * 64,
]


def write(directory: pathlib.Path, values: list[str]) -> int:
    directory.mkdir(parents=True, exist_ok=True)
    written = 0
    for value in values:
        data = value.encode("utf-8")
        # Content-addressed, so re-running never duplicates a seed.
        name = hashlib.sha1(data).hexdigest()[:16]
        path = directory / name
        if not path.exists():
            path.write_bytes(data)
            written += 1
    return written


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--from-csv", nargs=2, metavar=("PATH", "COLUMN"),
                    help="also seed from a column of a real file")
    ap.add_argument("--limit", type=int, default=300,
                    help="how many values to take from --from-csv")
    args = ap.parse_args()

    values = list(SEEDS)

    if args.from_csv:
        path, column = args.from_csv
        with open(path, newline="", encoding="utf-8") as handle:
            rows = list(csv.DictReader(handle))
        # Spread across the file rather than taking the first N, so the sample
        # is not just whatever happens to sort first.
        step = max(1, len(rows) // args.limit)
        values += [r[column] for r in rows[::step] if r.get(column)]

    target = pathlib.Path(__file__).parent / "seeds" / "parse"
    written = write(target, values)
    print(f"{written} new seeds written to {target} "
          f"({len(list(target.iterdir()))} total)", file=sys.stderr)


if __name__ == "__main__":
    main()
