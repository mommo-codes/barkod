# A GTIN is not a primary key

barkod will canonicalise a GTIN, classify it, and give you a stable match key
for it. None of that makes it a unique identifier for a row in your database,
and treating it as one is the most repeated mistake in the systems this
library was built for.

Four open bugs in one codebase shared this single root cause. It is written
here, loudly, because a library that stayed silent about it would end up
blamed for it.

## Why it is not unique

**Variants share one, by design.** A product register's `gtin` column is
routinely non-unique on purpose: the same trade item appears more than once
with different attributes. Such a column has no unique constraint because a
unique constraint would be wrong.

**The same product can carry several.** A GTIN-8, a GTIN-13 and a GTIN-14
grouping of the same item are three different numbers for one thing.
Canonicalising to GTIN-14 collapses the *spellings* of one number; it does not
merge a retail unit with the case it ships in.

**Suppliers reuse them.** GS1 permits reallocation after a product is
withdrawn. A GTIN identifies a trade item at a point in time.

**Some of them are not GTINs at all.** In-store PLU codes, variable-weight
codes with a zeroed measure field, and manual-entry sentinels all live in GTIN
columns and are all legitimately there. barkod classifies them rather than
rejecting them precisely because they are real.

## What goes wrong

```sql
SELECT * FROM products WHERE gtin = $1
```

Written against a non-unique column and consumed with
`scalar_one_or_none()`, this raises the moment a second row exists — which it
already does. Written as an upsert key, it creates a duplicate row for the
same product instead of updating one. Written as a join key, it multiplies
rows: two register rows against three catalog rows is six, silently, and the
count looks like a successful match.

None of these fail loudly. They produce answers.

## What to do instead

**Use your own primary key.** A surrogate `id` you control. The GTIN is an
attribute of the row, not its identity.

**Expect multiple rows.** Every lookup by GTIN returns a set. If your code
path needs exactly one, it needs a rule for choosing — and that rule is
business logic that belongs in the open, not in the shape of a query.

**Match on `GtinKey`, store `Gtin14`.** The key is for finding candidates.
What you do with more than one candidate is your decision, and barkod
deliberately does not make it for you.

**Do not blank what you cannot read.** `store_form` hands back the raw value
for anything outside the domain, so a row with an unreadable GTIN keeps the
only handle it has. It is still a row. It still needs to be findable.
