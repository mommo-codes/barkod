# barkod has no data, and never will

barkod is pure computation over a string of digits. Zero dependencies, no
data files, no I/O, no clock, no database. `cargo tree -p barkod` is one
line, and CI asserts that.

This is a design constraint, not a stage the project will grow out of.

## What is in, and what is out

**In: structure.** The GS1 prefix ranges in `allocation.rs` say what a number
range is *for* — restricted circulation, ISBN, coupons. They are defined by
the GS1 General Specifications, they change on the timescale of the standard
itself, and they are what makes a number classifiable without asking anyone
anything.

**Out: registries.** Which company owns a prefix, which country a prefix was
allocated to, what a given E-number is, which GPC brick a product belongs in.
These look like the same kind of knowledge and are not. They are *lookups*:
rows that someone maintains, with a publication date, that are wrong the
moment they go stale.

## Why registries belong in a database, not in this crate

E-numbers, additives, country codes and GPC codes are all useful. None of them
belong here, and none of them belong in a sibling crate either.

**Because the questions are joins.** "Every product containing E202" is a
query against a product table. A crate cannot answer it without loading that
whole table into memory, which is the wrong shape for the question and the
wrong place for the data. Anywhere this data is actually wanted, there is
already a database holding the products it would be joined against.

**Because updating should not be a release.** A registry in a database is
updated by a migration or an admin edit. A registry in a crate is updated by:
cut a release, publish to crates.io, publish to PyPI, publish to npm, then
bump the version in every consumer. That is four artefacts and a coordination
problem to correct one row, and it means the correction lands in different
services at different times — which is its own class of bug.

**Because the same reasoning already excluded GEPIR.** The GS1 company prefix
registry — which company owns which prefix — was considered and left out for
exactly this reason. Letting a different registry in through a side door would
be the same mistake with a different table.

**Because "zero dependencies, correct forever" is only true without data.**
Every function in barkod is a function of its input alone. That is what makes
the property tests meaningful and what makes the same answer come out of Rust,
Python and TypeScript. A lookup table breaks it: the answer starts depending
on which version of the table you happen to have.

## The one case this does not settle

Anything a frontend needs **offline** cannot come from a server. A scanner
running in a shop with no connection cannot join against a database, and if it
has to name a country from a prefix while offline, that data has to be in the
bundle.

Country codes are the likely candidate. Nothing else obviously is.

This is recorded, not designed for. If a real case appears, decide it then —
and if the answer turns out to be "ship a table", it should be a separate
package with a dated data file that barkod does not depend on, so the core
keeps its guarantee either way.

## For anyone about to add a lookup table here

Don't. Ask instead:

1. Is the question a join over rows stored somewhere already? Then it is a
   query.
2. Does the answer have a publication date? Then it is a registry.
3. Would a correction to it require a release of this crate? Then it does not
   belong in this crate.

If all three are no, it is probably structure, and it probably belongs in
`allocation.rs` with a citation to the specification that defines it.
