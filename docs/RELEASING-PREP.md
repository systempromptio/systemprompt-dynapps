# systemprompt-astound — release readiness prompt

Committed counterpart of the gitignored working doc at
`.systemprompt/release-prep-prompt.md`. Procedure lives in `docs/RELEASING.md`
and `docs/BRANCHING.md`; anything here that proves durable belongs there.

## Role

This prompt prepares **one consumer repo** for a core release. It is not the
release. The core prompt owns the version bump, the publish, and the fan-out;
this repo's job is to be in a state where its lane of core's Phase 4 is a
mechanical pin-flip and a `just release X.Y.Z`.

The `systemprompt-internal` repo carries the same prompt against its own facts.
Read that one for internal and this one for astound — **they diverge on the
single most load-bearing point below**, and applying internal's conclusions here
will make you skip the local pass that actually protects this repo.

## The one rule about checks, applied to this repo

Every check runs in exactly one place. Read this before running anything.

`.github/workflows/gates.yml` triggers on `workflow_dispatch`, `workflow_call`,
`pull_request: [main, next]` **and `push: [next]`.** That last trigger is the
difference from `systemprompt-internal`, and it inverts internal's advice:

- **A push to `next` runs the full gate here.** `gates.yml` is `just verify`
  (preflight-static → preflight-lint → test) plus a separate coverage job
  enforcing the floor and ratchet. So pushing `next` *is* remote proof, and
  `next` does **not** accumulate gate debt silently the way internal's does.
- **There is no promotion PR in this repo's flow.** `main` moves only by
  fast-forward from `next`, driven by `just release X.Y.Z`; every push to `main`
  IS the release (`release.yml` builds the GHCR image, the bridge binaries for
  three platforms, the server tarballs and the GitHub Release, and creates the
  `vX.Y.Z` tag). There is no PR whose checks someone reads instead.
- **`gates.yml` is called by `release.yml`** via `workflow_call`, so the release
  is proven on the exact commit it ships from.

Because `main` is reached by fast-forward and not by a reviewable PR, this repo's
local pass is **wider** than internal's, not narrower. `just preflight` is the
mandatory pre-merge gate and `just release` runs it for you:

```bash
just verify      # mid-iteration: static → lint → test  (== what CI runs on push)
just preflight   # before landing next -> main: verify + coverage floor/ratchet
```

Run `just preflight` before handing off. Do not hand-run the tiers piecemeal:
`just build`, `just clippy`, `just test-*` and `just lint-gates` are single-flight
through `scripts/build-coordinator.sh` and several agents share this clone. Check
`just build-status` first and attach to an in-flight run rather than starting a
second.

The two checks that CI genuinely cannot surface, and that must be run locally:

```bash
scripts/sync-release-version.sh <version> --check
bash scripts/check-core-ref.sh
```

`check-fork-drift.sh` **does not exist in this repo** — the fork-drift gate was
removed on 2026-08-26 when the template stopped being tracked. Do not go looking
for it. `check-release-version.sh` does not exist either; this repo's equivalent
is `sync-release-version.sh --check`.

## What this repo must be in, before core publishes

### 1. The pins must satisfy the patch, or the build proves nothing

`[patch.crates-io]` is ignored **silently** when its version does not satisfy the
requirement, and cargo then resolves the published crate instead. For a 0.x crate
`version = "0.45.0"` means `>=0.45.0, <0.46.0`, so a patch pointing at a 0.46.0
core does not satisfy it and is dropped — the build goes green having compiled
none of the code being released.

`scripts/sync-release-version.sh <version> --check` catches exactly this. It is
the one gate whose failure is invisible in a build log.

The **only** positive proof the patch is live is the build log naming vendored
paths:

```bash
grep -c '\.vendor/systemprompt-core' .build/logs/build-*.log | tail -1
```

Zero matches means the whole pass proved nothing. Treat it as a hard failure.

### 2. Three manifests, two patch blocks, one path dependency

"Both manifests" is three files here, and they do not behave alike:

| Manifest | Core arrives via | On `main` |
|---|---|---|
| `Cargo.toml` | `[patch.crates-io]` (12 crates, line 271) + `systemprompt*` pins (lines 43-45) | patch block commented out, pins at the release version |
| `tests/Cargo.toml` | its own `[patch.crates-io]` (line 113) + its own pins (lines 55-56) | same |
| `bridge/Cargo.toml` | **a bare path dep** — `systemprompt-bridge = { path = "../../systemprompt-core/bin/bridge" }` (line 25), no version, no patch block | unchanged: sibling-coupled in every branch |

`[patch]` is per-workspace, which is why the first two each carry their own block
and their own pins. The bridge is not on crates.io and never resolves from the
registry; removing the patch blocks does not change it. Do not "remove the
bridge's patch block" — there isn't one, and the comment at `bridge/Cargo.toml:23`
saying so is easy to grep past.

There are no skip-worktree'd manifests here (`git ls-files -v | grep ^S` is
empty). Every crate under `extensions/**` and `tests/**` takes core through
`workspace = true`, so there is no per-crate pin sweep to do. Confirm rather than
assume:

```bash
grep -rn '^systemprompt[a-z-]* = ' --include='Cargo.toml' . | grep -v target | grep -v '\.vendor'
```

### 3. `bridge/CORE_REF` gates the whole remote proof, not just the bridge

`gates.yml` materialises one core checkout at the ref in `bridge/CORE_REF` and
serves both consumers from it: the `[patch]` paths point at
`.vendor/systemprompt-core`, and `bridge/` depends on the sibling path, so the
checkout lands in `.vendor/` and a symlink stands in for the sibling. While the
patch blocks are active, **that file decides which core CI compiles against.**

- On `next`: a 40-char SHA on core's `next`. Keep it moving with core; a stale
  SHA silently gates against an older core. `just core-pin` with no argument
  writes the sibling's current `HEAD`.
- On `main`: `v<version>`. `scripts/check-core-ref.sh` **enforces this** — when
  `[patch.crates-io]` is commented it requires `CORE_REF == v<pin>` and fails
  otherwise. This is stronger than internal's, where both forms are accepted and
  nothing catches the switch being missed. `just core-bump X.Y.Z` writes it.

`check-core-ref.sh` is one of the 25 gates in `just lint-gates`, so it runs on
every push. While the patch is active only the *shape* of the SHA is checked —
nothing verifies it still names core `next` HEAD. That check is yours.

### 4. `.vendor/` is a build input, and it is a copy of a working tree

This is the sharpest edge in this repo and it has no analogue in internal.

`just vendor-core` rsyncs `../systemprompt-core` into `.vendor/` — **working tree
included, uncommitted changes and all.** The Docker build context is `COPY . /src`
and cannot see the sibling, so the core source has to live inside this repo. The
consequences:

- `.vendor/` is a **stale rsync artifact**, not a source of truth. Never read it
  to learn what core does; read `../systemprompt-core`.
- A **dirty** `../systemprompt-core` means every local result here describes
  another session's uncommitted work — work that will not be in the published
  core, and that CI (which checks out `CORE_REF`) will not have.
- `bridge/CORE_REF` names a *commit*, but `.vendor/` holds a *working tree*.
  These two disagree the moment the sibling is dirty, and nothing warns you.

Check all three before believing any local result:

```bash
git -C ../systemprompt-core status --porcelain      # must be empty
git -C ../systemprompt-core rev-parse next          # must equal bridge/CORE_REF
diff -rq --exclude=target --exclude=.git .vendor/systemprompt-core ../systemprompt-core
```

Core is **write-only from here**: never checkout, pull, fetch, test, clippy or fmt
in that checkout. If it is dirty, the owner of that work commits and pushes it to
core `next`; then re-pin and re-vendor here.

### 5. Three lockfiles, and `cargo update -w` re-resolves one

`Cargo.lock`, `tests/Cargo.lock`, `bridge/Cargo.lock`. `cargo update -w`
re-resolves the **root workspace only**; a manifest `--check` reads manifests and
cannot see a lockfile still pointing at a path. Re-resolve each explicitly:

```bash
cargo update -w
cargo update --manifest-path tests/Cargo.toml -w
cargo update --manifest-path bridge/Cargo.toml -w
```

Assert by **core crate sourcing**, not by counting registry lines — a raw
`grep -c 'registry+...'` counts every third-party crate (636/643/629 here) and
tells you nothing about core:

```bash
for f in Cargo.lock tests/Cargo.lock bridge/Cargo.lock; do
  echo "$f: $(awk '/^name = "systemprompt/{getline; getline; if ($1=="source") c++} END{print c+0}' "$f") of $(grep -c '^name = "systemprompt' "$f") core crates registry-sourced"
done
```

The correct `next` shape is **0 registry-sourced** in all three (45 / 43 / 6 core
crates, all patched or path). After the pin-flip, root and `tests/` must carry
them registry-sourced and `bridge/` must still carry none.

### 6. The two version gates must agree

`scripts/check-docs-version.sh` (a lint gate) and
`scripts/sync-release-version.sh --check` (run by `just release`) both sweep
operator docs for stale versions, and both must honour the
`<!-- pinned-release -->` end-of-line exemption for sentences stating a fact about
a specific past release. They drifted apart once — the lint gate learned the
exemption in `3320065b` and the sweep did not, so `just lint-gates` passed and
`just release` then failed on the same line, with no local signal until the
release itself. If you touch either, touch both.

## Publication surface this repo owns

`release.yml` triggers on `push: main`. There is no separate tag step — the
workflow creates the `vX.Y.Z` tag. It publishes:

- `ghcr.io/astound-digital/systemprompt-astound` — `:X.Y.Z` (create-once),
  `:latest`, `:sha-<short>`, amd64 + arm64, cosign-signed;
- a GitHub Release `vX.Y.Z` (create-once) carrying the three-platform bridge
  binaries, the server tarballs, `install.sh`, `SHA256SUMS` and a cosign
  `.sig`/`.pem` per asset.

The `vX.Y.Z` tag, the Release and its assets, and the `:X.Y.Z` image tag are
**created once and never overwritten**; a second push to `main` at the same
version rebuilds only `:latest` and `:sha-<short>`. So confirm by content — pull
the tag, list the assets — never by a green tick.

Deployment is separate and manual: `just deploy-next` (re-vendors first) to the
`production` profile, which for this repo is a **staging** environment at
`https://astound.systemprompt.io`. A bare `just deploy` on `next` would ship
whatever stale copy is in `.vendor/`. Require the new version back:

```bash
curl -fsS https://astound.systemprompt.io/health
```

A 200 carrying the old build is a failed deploy.

## Ready when

- `scripts/sync-release-version.sh <version> --check` and
  `bash scripts/check-core-ref.sh` both green locally.
- `../systemprompt-core` **clean**, its `next` HEAD equal to `bridge/CORE_REF`,
  and `.vendor/` re-rsynced from it after the last core commit.
- The build log names `.vendor/systemprompt-core` paths — the patch is provably live.
- `just preflight` green, including the coverage floor and ratchet
  (`coverage/baseline.json`: floor 83.3, total 83.86).
- `next` pushed and its `gates.yml` run green — unlike internal, that run is real
  proof here, and it is the last remote signal before `main`.
- The pin-flip is understood and left to the promoting agent: two patch blocks
  commented out, pins in both workspaces to the release version, `CORE_REF` to
  `v<version>`, three lockfiles re-resolved. `just core-bump X.Y.Z` does the first
  three; `just release X.Y.Z` re-checks them and refuses if the patch is still
  active, the tree is dirty, or `HEAD != origin/next`.

Then hand off. The promoting agent runs `just core-bump` and `just release`; this
repo's preparer does not.

## Where this repo differs from `systemprompt-internal`

Fold these into `internal/release.md` §1d as *repo-specific*, not general:

1. **"A consumer has no push CI trigger" is not universal.** Astound's
   `gates.yml` runs on `push: next`, so a push there is real proof and `next` does
   not silently rot. Internal's "the promotion PR is the only proof" is a fact
   about internal.
2. **Astound has no promotion PR at all.** `main` moves by fast-forward via
   `just release`, and every push to `main` is the release. The local `preflight`
   is therefore the review, which is why it is mandatory here and skipped there.
3. **`check-core-ref.sh` does catch the missed `main` switch here.** Internal's
   `--check` accepts both forms; astound's gate hard-requires `v<pin>` whenever
   the patch is commented. Internal should adopt it.
4. **Astound vendors core into the repo (`.vendor/`), from a working tree.** The
   `[patch]` paths are not the sibling; they are an rsync of it that includes
   uncommitted changes. `CORE_REF` (a commit) and `.vendor/` (a tree) can disagree
   silently. Internal patches the sibling directly and has no such gap.
5. **Neither repo's `bridge/Cargo.toml` has a patch block** — that one is common,
   and worth stating once in the shared doc rather than twice.
