/**
 * barkod — GTIN parsing, canonicalisation, match keys and GS1 classification.
 *
 * The same compiled Rust the Python package runs. There is no TypeScript
 * reimplementation of any rule here, which is the entire point of shipping a
 * wasm target: the JavaScript side of this platform used to carry its own
 * port of these rules, and it drifted.
 *
 * What this file *does* add is the type distinction wasm cannot carry across
 * the boundary by itself.
 */

import initWasm, {
  parse as wasmParse,
  storeForm as wasmStoreForm,
  keyString as wasmKeyString,
  storeFormMany as wasmStoreFormMany,
  type Gtin14 as WasmGtin14,
  type Parsed as WasmParsed,
} from "../wasm/barkod_wasm.js";

/**
 * A canonical GTIN-14. A branded string: still a string at runtime, but the
 * compiler will not let a {@link GtinKey} be assigned into one.
 */
export type Gtin14String = string & { readonly __barkod: "gtin14" };

/**
 * A check-digit-agnostic match key. **Not a GTIN.**
 *
 * Branded separately from {@link Gtin14String} so the compiler refuses the
 * mistake that shipped in production: writing a match key into a column that
 * holds GTINs. Roughly nine in ten keys end in a check digit no product has.
 *
 * ```ts
 * const k = key("7390525907745");
 * const g: Gtin14String = k;  // ✗ does not compile
 * ```
 *
 * The brand is compile-time only. A `GtinKey` still satisfies a plain
 * `string` parameter, so it protects typed slots, not untyped ones — type
 * your GTIN columns and the guard works.
 */
export type GtinKey = string & { readonly __barkod: "key" };

export type Reason = "empty" | "non_numeric" | "too_short" | "too_long";
export type Encoding = "GTIN-8" | "GTIN-12" | "GTIN-13" | "GTIN-14";

export type Gtin14 = Omit<WasmGtin14, "value" | "keyString"> & {
  readonly value: Gtin14String;
  keyString(): GtinKey;
};

export type Parsed = Omit<WasmParsed, "gtin" | "reason" | "storeForm"> & {
  readonly gtin?: Gtin14;
  readonly reason?: Reason;
  readonly storeForm: string;
};

let ready = false;

/**
 * Load the WebAssembly module. Call once, at application start.
 *
 * Every other function throws until this resolves. That is deliberate: a
 * library that quietly returned `""` or `false` while warming up would be a
 * worse version of the bug barkod exists to remove.
 */
export async function init(module?: Parameters<typeof initWasm>[0]): Promise<void> {
  await initWasm(module);
  ready = true;
}

/** Whether {@link init} has completed. */
export function isReady(): boolean {
  return ready;
}

function assertReady(): void {
  if (!ready) {
    throw new Error(
      "barkod: call `await init()` once before using the library. " +
        "Refusing to answer rather than returning a wrong answer.",
    );
  }
}

/** Read one cell. The primary entry point. */
export function parse(input: string): Parsed {
  assertReady();
  return wasmParse(input) as unknown as Parsed;
}

/**
 * What to write when storing this value: the canonical GTIN-14 inside the
 * domain, the input untouched outside it. Never blank unless the input was.
 */
export function storeForm(input: string): string {
  assertReady();
  return wasmStoreForm(input);
}

/**
 * The match key, or `undefined` outside the domain.
 *
 * `undefined` means "this row cannot match anything", which is the honest
 * answer for a blank cell, a sentinel, or an 18-digit identifier that is not
 * a GTIN. Manufacturing a key for those is how unrelated rows join.
 */
export function key(input: string): GtinKey | undefined {
  assertReady();
  return wasmKeyString(input) as GtinKey | undefined;
}

/** Storage forms for a whole column. */
export function storeFormMany(inputs: string[]): string[] {
  assertReady();
  return wasmStoreFormMany(inputs);
}

/** Match keys for a whole column, with `undefined` where there is no key. */
export function keyStrings(inputs: string[]): (GtinKey | undefined)[] {
  assertReady();
  return inputs.map((input) => wasmKeyString(input) as GtinKey | undefined);
}

/** The GTIN domain: 8 digits through 14. */
export const DOMAIN_MIN_DIGITS = 8;
/** See {@link DOMAIN_MIN_DIGITS}. */
export const DOMAIN_MAX_DIGITS = 14;
