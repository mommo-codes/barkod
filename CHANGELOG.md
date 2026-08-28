# Changelog

## 0.1.1 — the first coherent release

**Use this one. [0.1.0](#010--superseded) is superseded.**

No behaviour changed between 0.1.0 and 0.1.1. Every rule, type and answer is
identical. What changed is that all three published artefacts, and the git tag,
now come from a single tree.

- npm package renamed to **`@mommo-codes/barkod`**. The unscoped name is
  refused by npm as too similar to an existing package.
- The npm tarball now actually contains the WebAssembly. `wasm-pack` writes a
  `.gitignore` containing `*` into its output directory, and npm honours
  nested `.gitignore` files when packing — so the 0.1.0 tarball was four files
  with no wasm in it, and every import would have failed at runtime.
- The npm package ships a README, so its registry page is not blank.
- The release workflow no longer fails when one registry is already published,
  so a re-run to fix a single target is possible.

## 0.1.0 — superseded

Published, functional, and **not reproducible from any single commit.**

It went out across two workflow runs from three slightly different trees, and
the `v0.1.0` tag matches none of the first two:

| artefact | tree it came from |
|---|---|
| `barkod 0.1.0` on crates.io | the initial commit |
| `barkod 0.1.0` on PyPI | the initial commit |
| npm | never published under this version |

The artefacts themselves are correct. The problem is that "which commit is
this?" has no answer, and that question gets asked exactly when something has
gone wrong and nobody has time to reconstruct it.

That is the same class of drift this library exists to close — one rule with
several spellings, none authoritative. Leaving it in the release history
while arguing against it in the documentation would have been the wrong
trade, so 0.1.1 republishes all three from one tree and this note records why.

If you have pinned `barkod 0.1.0`, move to `0.1.1`. If you were waiting on the
npm package, it only exists from `@mommo-codes/barkod@0.1.1` onwards.
