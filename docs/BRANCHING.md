# Branching: `next` and `main`

Two branches, two cores. Which one you are on decides where every
`systemprompt-*` crate resolves from, and nothing else in the repo changes.

| | `next` | `main` |
|---|---|---|
| Purpose | development; the coming release, shipped before it is perfect | stable; what colleagues install |
| Core | **unreleased core `next`**, vendored into `.vendor/systemprompt-core` | **the last core published on crates.io** |
| `[patch.crates-io]` in `Cargo.toml` **and** `tests/Cargo.toml` | active (uncommented) | commented out |
| Version pins (`systemprompt = "X.Y.Z"`) | the vendored tree's workspace version | the published release |
| `bridge/CORE_REF` | a 40-char SHA on core `next` — what was vendored | `vX.Y.Z`, the tag of the pinned release |
| CI on push | **gates only** (`.github/workflows/gates.yml`) — no images | gates **+ release** (`.github/workflows/release.yml`): image, bridge binaries, GitHub Release |
| Deploys | `just deploy-next` → the `production` Fly profile, which is a **staging** environment | `just deploy` (Fly), or colleagues pull the published image |
| Allowed to break? | yes — a red gate on `next` is information, not an incident | no — every push is a release |

`main` only moves by fast-forward from `next` (`just release X.Y.Z`), after the
core it was proven against has been published. Nothing is committed directly
to `main`.

## Why the core lives in two places

`bridge/` depends on `systemprompt-bridge` by **path** — it is not published —
so a core checkout must exist at `../systemprompt-core` on both branches. The
server workspaces, when patched, resolve through `.vendor/systemprompt-core`,
because the Docker build context is `COPY . /src` and cannot see a sibling
directory. Locally `just vendor-core` rsyncs the sibling (working tree
included) into `.vendor/`; `just deploy-next` does it for you. In CI the
composite action `.github/actions/core-checkout` checks core out at
`bridge/CORE_REF` straight into `.vendor/systemprompt-core` and symlinks the
sibling path onto it, so one checkout serves both consumers.

## The silent-drop trap

A `[patch]` whose version does not satisfy the pin is **dropped without an
error**: cargo resolves the published crate instead, the build passes, and it
has proved nothing about the core you meant to test. For a 0.x crate
`version = "0.41.0"` means `>=0.41.0, <0.42.0`, so a patch pointing at a
0.42.0 tree is ignored by a 0.41.0 pin.

Defences, in the order they catch it:

1. `scripts/sync-release-version.sh <version> --check` — every pin in both
   workspaces must agree (a residual sweep fails on any pin it does not move).
2. `scripts/check-core-ref.sh` (a `lint-gates` gate) — with the patch
   commented, `bridge/CORE_REF` must be `v<pin>`.
3. The build log. On `next` it must name `.vendor/systemprompt-core/...`
   paths for the `systemprompt-*` crates; a bare `v0.42.0` with no path is a
   dropped patch. `release.yml` additionally refuses a `main` whose
   `Cargo.lock` does not resolve `systemprompt` from the registry.

## Working on `next`

```bash
just vendor-core                 # after every core edit (rsyncs ../systemprompt-core)
just core-pin                    # bridge/CORE_REF = the sibling's HEAD — commit it so CI builds the same core
just verify                      # what gates.yml runs
git push origin next             # gates.yml runs; nothing is published
just deploy-next                 # staging preview on Fly
```

Core changes are **write-only from here**: commit them on core's `next`
branch and push — core's own CI is their validation surface. `bridge/CORE_REF`
must name a commit that exists on GitHub, so push core before pushing here.

## Landing `next` on `main` (once core X.Y.Z is on crates.io)

```bash
# on next
#   1. comment out BOTH [patch.crates-io] blocks (Cargo.toml, tests/Cargo.toml)
just core-bump X.Y.Z             # pins, bridge/CORE_REF=vX.Y.Z, cargo update, migrate, build, clippy
just prepare                     # refresh .sqlx/ against the new core if queries changed
just preflight                   # static → lint → tests → coverage ratchet
git commit -am "release: build against published core X.Y.Z" && git push origin next
just release X.Y.Z               # re-checks everything, then fast-forwards main
```

`just release` refuses when the tree is dirty, when run off `next`, when a
patch block is active, when a pin or `CORE_REF` disagrees, when `origin/next`
is behind, or when `main` is not an ancestor of `HEAD`. The push to `main`
triggers `release.yml`; what it produces is in [RELEASING.md](RELEASING.md).

Afterwards, to resume development, re-activate the patch blocks on `next`,
`just vendor-core`, `just core-pin`, and bump the pins to the vendored
version when core moves past X.Y.Z.
