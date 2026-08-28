/**
 * Runtime tests for the TypeScript/wasm build.
 *
 * The GTIN rules are tested once, in Rust. What is tested here is that the
 * same answers survive the wasm boundary, and that the wrapper refuses to
 * work before it is ready rather than answering wrongly.
 */

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import * as barkod from "../dist/index.js";

// `--target web` expects to fetch the .wasm; under Node we hand it the bytes.
const wasmBytes = await readFile(new URL("../wasm/barkod_wasm_bg.wasm", import.meta.url));

test("refuses to answer before init", () => {
  assert.equal(barkod.isReady(), false);
  assert.throws(() => barkod.storeForm("7350053850019"), /call `await init\(\)`/);
});

test("init makes it ready", async () => {
  await barkod.init({ module_or_path: wasmBytes });
  assert.equal(barkod.isReady(), true);
});

test("storage pads and preserves the check digit", () => {
  assert.equal(barkod.storeForm("7350053850019"), "07350053850019");
  assert.equal(barkod.storeForm("10700021526"), "00010700021526");
});

test("storage hands back anything out of domain, untouched", () => {
  for (const value of ["lek", "MANUAL-a1b2c3d", "1234567", "073905259077450", ""]) {
    assert.equal(barkod.storeForm(value), value);
  }
});

test("keys refuse outside the domain", () => {
  assert.equal(barkod.key("7390525907745"), "07390525907740");
  assert.equal(barkod.key("073905259077450"), undefined);
  assert.equal(barkod.key("lek"), undefined);
  assert.equal(barkod.key(""), undefined);
});

test("parse reports why, and keeps the raw input", () => {
  assert.equal(barkod.parse("").reason, "empty");
  assert.equal(barkod.parse("lek").reason, "non_numeric");
  assert.equal(barkod.parse("1234567").reason, "too_short");

  const long = barkod.parse("370245100000000123");
  assert.equal(long.reason, "too_long");
  assert.equal(long.digits, 18);
  assert.equal(long.message, "More than 14 digits — not a GTIN");
  assert.equal(long.raw, "370245100000000123");
  assert.equal(long.storeForm, "370245100000000123");
});

test("missing and junk are different answers", () => {
  assert.notEqual(barkod.parse("").reason, barkod.parse("lek").reason);
});

test("cleaning is recorded", () => {
  const p = barkod.parse(" 7350053850019\r");
  assert.equal(p.gtin?.value, "07350053850019");
  assert.equal(p.wasCleaned, true);
  assert.equal(p.removedWhitespace, true);

  const f = barkod.parse("7318690123456.0");
  assert.equal(f.gtin?.value, "07318690123456");
  assert.equal(f.droppedFraction, true);
});

test("classification matches the Rust and Python answers", () => {
  const weight = barkod.parse("02011030000000").gtin;
  assert.equal(weight.allocation, "restricted_circulation_region");
  assert.equal(weight.checkDigitIsValid, false);
  assert.equal(weight.expectedCheckDigit, 5);
  assert.equal(weight.measureFieldLooksZeroed, true);

  const book = barkod.parse("09789137163666").gtin;
  assert.equal(book.allocation, "isbn");
  assert.equal(book.encoding, "GTIN-13");

  const variable = barkod.parse("91599903387551").gtin;
  assert.equal(variable.indicator, "variable_measure");

  const plu = barkod.parse("00000000014150").gtin;
  assert.equal(plu.looksLikeInternalCode, true);
  assert.equal(plu.shortest, "00014150");
});

test("shrink leaves over-length values alone because it never sees them", () => {
  assert.equal(barkod.parse("00000007350001").gtin.shortest, "07350001");
  assert.equal(barkod.parse("073905259077450").isGtin, false);
});

test("batch forms", () => {
  assert.deepEqual(barkod.storeFormMany(["7350053850019", "lek", ""]), [
    "07350053850019",
    "lek",
    "",
  ]);
  assert.deepEqual(barkod.keyStrings(["7390525907745", "lek"]), [
    "07390525907740",
    undefined,
  ]);
});
