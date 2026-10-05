# Branching: `next` and `main`

Two branches, two cores. Which one you are on decides where every
`systemprompt-*` crate resolves from, and nothing else in the repo changes.

| | `next` | `main` |
|---|---|---|
| Purpose | development; the coming release, shipped before it is perfect | stable; what colleagues install |
| Core | **unreleased core `next`**, the sibling checkout `../systemprompt-core` | **the last core published on crates.io** |
| `[patch.crates-io]` in `Cargo.toml` **and** `tests/Cargo.toml` | active (uncommented) | commented out |
| Version pins (`systemprompt = "X.Y.Z"`) | the sibling tree's workspace version | the published release |
| `bridge/CORE_REF` | a 40-char SHA on core `next` — the sibling's HEAD, which `just deploy` enforces | `vX.Y.Z`, the tag of the pinned release |
| CI on push | **gates only** (`.github/workflows/gates.yml`) — no images | **release** (`.github/workflows/release.yml`): image, bridge binaries, GitHub Release — the gates ran on the push to `next`, which `just release` verifies before PR promotion; coverage (`coverage.yml`) measures the merge commit without blocking |
| Deploys | `just deploy` → the `production` Fly profile, which is a **staging** environment (refuses unless the sibling core is clean at `CORE_REF`) | `just deploy` (Fly), or colleagues pull the published image |
| Allowed to break? | yes — a red gate on `next` is information, not an incident | no — every push is a release |

`main` only moves through a frozen promotion PR (`just release X.Y.Z`), after the
core it was proven against has been published. Nothing is committed directly
to `main`.

## One core checkout, one place

`bridge/` depends on `systemprompt-bridge` by **path** — it is not published —
so a core checkout must exist at `../systemprompt-core` on both branches. The
server workspaces, when patched, resolve through the same sibling path. The
production image is built from **pre-built binaries** (the profile Dockerfile
copies `target/release/*`), so nothing in the Docker context needs the core
source, and there is no vendored copy to go stale. `just deploy` runs
`just core-guard` first: with the patch active, the sibling must be clean and
at `bridge/CORE_REF`, so the binary, CI and the bridge all come from one core
commit. In CI the composite action `.github/actions/core-checkout` checks core
out at `bridge/CORE_REF` inside the workspace and symlinks the sibling path
onto it.

## Three lockfiles, one core

`bridge/Cargo.toml` depends on `systemprompt-bridge` by path
(`../../systemprompt-core/bin/bridge`), and `tests/` is a separate workspace
with its own `Cargo.lock`, so there are three lockfiles and `cargo update -w`
in the root refreshes only one of them. The design choice is that all three
resolve every `systemprompt-*` crate at **the same version** — a path copy and
a registry copy may coexist, but not two versions. Otherwise a test or bridge
build carries a second copy of a shared crate and fails with a type-mismatch
error that names neither lockfile. `scripts/check-core-crate-versions.sh`
(`preflight-static`, the `static` gate) enforces it; `just core-bump` refreshes
all three.

## The silent-drop trap

A `[patch]` whose version does not satisfy the pin is **dropped without an
error**: cargo resolves the published crate instead, the build passes, and it
has proved nothing about the core you meant to test. For a 0.x crate
`version = "0.41.0"` means `>=0.41.0, <0.42.0`, so a patch pointing at a
0.42.0 tree is ignored by a 0.41.0 pin.

Defences, in the order they catch it:

1. `scripts/sync-core-version.sh <core-version> --check` — every core pin in
   both workspaces must agree (a residual sweep fails on any pin it does not
   move).
2. `scripts/check-core-ref.sh` (a `lint-gates` gate) — with the patch
   commented, `bridge/CORE_REF` must be `v<pin>`.
3. The build log. On `next` it must name `../systemprompt-core/...`
   paths for the `systemprompt-*` crates; a bare `v0.42.0` with no path is a
   dropped patch. `release.yml` additionally refuses a `main` whose
   `Cargo.lock` does not resolve `systemprompt` from the registry.

## Working on `next`

```bash
just core-pin                    # bridge/CORE_REF = the sibling's HEAD — commit it so CI builds the same core
just verify                      # what gates.yml runs
git push origin next             # gates.yml runs; nothing is published
just deploy                      # staging preview on Fly; core-guard checks the sibling first
```

Core changes are **write-only from here**: commit them on core's `next`
branch and push — core's own CI is their validation surface. `bridge/CORE_REF`
must name a commit that exists on GitHub, so push core before pushing here.

## Landing `next` on `main` (once core X.Y.Z is on crates.io)

```bash
# Remove BOTH [patch.crates-io] blocks, then prepare every version pin and CORE_REF.
scripts/sync-core-version.sh C.D.E           # core pins + bridge/CORE_REF
scripts/sync-release-version.sh X.Y.Z        # astound's own version
cargo update -w
cargo update -w --manifest-path tests/Cargo.toml
cargo update -w --manifest-path bridge/Cargo.toml
# Record the release's schema-upgrade rung (just schema-baseline) and refresh SQLx caches if their inputs changed.
git add -A
git commit -m "release: build against published core X.Y.Z"
git push origin HEAD:next        # the full Gates matrix runs here
just release X.Y.Z               # open frozen PR; repeat after its proof passes
```

`just release` refuses when the tree is dirty, when run off `next` (a clean
detached worktree at `origin/next` also qualifies), when a patch block is active, when a pin or `CORE_REF` disagrees, when the schema
fixture records a different version, when `origin/next` is behind, or when
`main` is not an ancestor of `HEAD`. The push to `main`
triggers `release.yml`; what it produces is in [RELEASING.md](RELEASING.md).
Once `release.yml` is green, `just deploy-release X.Y.Z` ships that release to
production from a clean worktree of `origin/main` — not `just deploy`, which
builds and ships this working tree.

Afterwards, to resume development, re-activate the patch blocks on `next`,
`just core-pin`, and bump the pins to the sibling's
version when core moves past X.Y.Z.

## Emergency patch releases

Create `hotfix/<astound-version>/<main-base-sha>` in a separate worktree from
`origin/main`. Keep the published core pins and `bridge/CORE_REF` unchanged;
bump only Astound's release version. Emergency PR gates run static checks, the complete lint tier and focused
Salesforce/OAuth unit regressions; the full test matrix continues in a separate
nonblocking job. They verify the frozen base, candidate tree and published core. Before merging,
run `scripts/check-hotfix-green.sh <head-sha> <branch> <main-base-sha>` and verify
the PR merge tree equals the candidate. Use a two-parent merge through GitHub;
`release.yml` verifies that exact PR proof before publishing all artifacts.
Merge the release back into `next`, preserving its active core patches and pins.

For an unchanged schema on the same released core, `just schema-baseline` can
use `SCHEMA_BASELINE_CLI` (a checksum-verified released binary or wrapper) and
`SCHEMA_BASELINE_CONTAINER` (the local Postgres container). The recipe checks
the binary's core version and rejects schema differences from the release tag,
then performs a fresh install and records the schema through the usual path.

During an Actions billing outage the owner may merge an isolated hotfix PR
before PR gates run. The release workflow then runs mandatory static checks,
root clippy and focused OAuth regressions on the exact merged tree before any
artifact publication. `HOTFIX_VALIDATE_IN_RELEASE=1` is accepted only inside
Actions for that merge; ordinary promotion proof remains unchanged. After
billing recovery, rerun the failed main Release workflow or dispatch it on
main with `dry_run=false`. Publication still requires signed image smoke and
upgrade checks.
