/**
 * Compile-time assertions. This file emits nothing and runs nothing — if it
 * type-checks, the guarantees below hold; if a `@ts-expect-error` stops being
 * an error, `tsc` fails the build.
 *
 * These are the TypeScript half of "a type that makes misuse impossible".
 */

import { key, parse, type Gtin14String, type GtinKey } from "./index.js";

declare const anyKey: GtinKey;
declare const anyGtin: Gtin14String;

// A match key must never be storable where a GTIN belongs. This is the exact
// shape of the production bug: a key written into a column of GTINs.
// @ts-expect-error a GtinKey is not a Gtin14String
const wrong: Gtin14String = anyKey;

// Nor the reverse — a GTIN is not a key, and comparing one to a key column
// would silently never match.
// @ts-expect-error a Gtin14String is not a GtinKey
const alsoWrong: GtinKey = anyGtin;

// Each is still assignable to its own brand.
const rightKey: GtinKey = anyKey;
const rightGtin: Gtin14String = anyGtin;

// `key` returns `undefined` outside the domain, so callers must handle it.
const maybe = key("073905259077450");
// @ts-expect-error possibly undefined
const forced: GtinKey = maybe;

// `parse().gtin` is optional for the same reason.
const parsed = parse("lek");
// @ts-expect-error possibly undefined
const forcedGtin: string = parsed.gtin.value;

// Silence unused-variable diagnostics; the assertions above are the test.
export type _Assertions = [
  typeof wrong,
  typeof alsoWrong,
  typeof rightKey,
  typeof rightGtin,
  typeof forced,
  typeof forcedGtin,
];
