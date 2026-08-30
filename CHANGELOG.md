# Changelog

## 0.1.2 — a Linux wheel for arm64

**Use this one.** No behaviour changed. Every rule, type and answer is
identical to 0.1.1.

`pip install barkod` failed inside any Linux container on an Apple Silicon
Mac. The release matrix built wheels on `ubuntu-latest` with `manylinux:
auto`, which produces x86_64 only, and there is no sdist to fall back on — so
an arm64 Linux interpreter had nothing it could install. Every developer on a
modern Mac hits this the moment they try to adopt the library, which is
exactly what happened the first time anything did.

- **`manylinux_2_17_aarch64` wheels**, cross-compiled by maturin from the same
  x86_64 runner. One more matrix leg, not a new runner.
- Every leg now names its target explicitly instead of relying on the
  runner's default, and the uploaded artefact is keyed on that name — two
  legs share `ubuntu-latest` and would otherwise overwrite each other.
- **npm and crates.io now wait for the wheels.** They need nothing from them,
  but a release is one tree across three registries, and without the gate a
  single failing wheel target publishes two registries and leaves PyPI a
  version behind. That is the split-tree release 0.1.1 exists to have ended,
  and the workflow was still shaped to permit it.
- The release workflow accepts `workflow_dispatch`, so the build matrix can
  be exercised without cutting a tag.

### 0.1.1 has a stray aarch64 wheel

Proving the new target built was done by dispatching the workflow while the
tree still said 0.1.1, on the assumption that every publish step would no-op.
Three of the four did. PyPI's `skip-existing` skips **files** that already
exist, not versions — and the aarch64 wheel was a new filename, so it
uploaded.

`barkod-0.1.1-cp38-abi3-manylinux_2_17_aarch64.manylinux2014_aarch64.whl` is
therefore real, installable, and built from a commit the `v0.1.1` tag does not
point at. Its library source is byte-identical to the tag — the commit
changed CI configuration and nothing under `barkod/`, `barkod-py/` or
`barkod-wasm/` — so it is correct, and it is left in place rather than
deleted, because deleting a file from PyPI burns the filename forever.

It is still a provenance wart of exactly the kind
[0.1.0](#010--superseded) was superseded for, and recording it here is
cheaper than someone later finding a wheel whose commit they cannot locate.
0.1.2 restores the property that a tag accounts for every artefact under it.

## 0.1.1 — the first coherent release

**Superseded by [0.1.2](#012--a-linux-wheel-for-arm64), which adds the arm64
Linux wheel this release is missing.**

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
