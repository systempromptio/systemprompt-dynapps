# Releasing

How a new `systemprompt` core version becomes a release of this repo: a
published image, signed bridge binaries for three platforms, and a GitHub
Release — produced by CI on every push to `main`.

**Nothing is built by hand any more.** `.github/workflows/release.yml` runs on
every push to `main` and publishes everything; `.github/workflows/gates.yml`
runs the quality gate on every push to `next` and every PR. The branch
contract that makes this safe — `next` on the vendored core `next`, `main` on
the published core — is in [BRANCHING.md](BRANCHING.md); read it first.

## Versioning policy

The fork tracks core in lockstep: core `X.Y.Z` on crates.io → workspace
`version = X.Y.Z` in both manifests → `bridge/CORE_REF = vX.Y.Z` → git tag
`vX.Y.Z` → image tag `X.Y.Z`. `scripts/sync-release-version.sh X.Y.Z --check`
is the drift gate. The bridge crate (`bridge/Cargo.toml`) keeps its own
version; its binaries ship attached to the `vX.Y.Z` release and the notes
state the bridge version.

Because the version is core's, most pushes to `main` will *not* carry a new
version. That is fine: the workflow is create-once for everything versioned
and only moves the floating tags on a repeat (see *Idempotency*).

## What a push to `main` produces

| artifact | where | tag / name | signed by |
|---|---|---|---|
| gateway image, amd64 + arm64 | `ghcr.io/astound-digital/systemprompt-astound` (private) | `X.Y.Z` (create-once), `latest`, `sha-<7>` | cosign keyless |
| server binaries tarball (`bin/systemprompt` + every `bin/systemprompt-mcp-*`, `services/`, MCP manifests) — consumed by `just fetch-release` / `setup-local` | GitHub Release `vX.Y.Z` | `systemprompt-astound-server-X.Y.Z-linux-{amd64,arm64}.tar.gz` | cosign keyless |
| bridge, Linux | Release **and** baked into the image at `/files/downloads/` | `astound-bridge-linux-{x86_64,aarch64}.tar.gz` + `.sha256` | cosign keyless |
| bridge, macOS universal | same | `astound-bridge-macos.dmg` + `.sha256` | cosign; Apple notarization when the `APPLE_*` secrets exist |
| bridge, Windows x64 | same | `astound-bridge-windows.exe` + `.sha256` | cosign; Authenticode when the `WINDOWS_CERT_*` secrets exist |
| installer | same | `install.sh` (release copy points at the release; image copy at the gateway) | cosign keyless |
| checksums | Release | `SHA256SUMS` + `.sig`/`.pem` | cosign keyless |

Job order, and why: `version` (asserts the branch contract and reads the
version) → `gates` (the full `just verify` on the merge commit) → `bridge`
matrix (four runners; **no build cache**, a warm `target/` once shipped a
stale binary) → `image` per arch (downloads the bridge artifacts into
`storage/files/downloads/` **before** building, because the image serves them;
pushes by digest) → `image-merge` (one manifest list, tags, cosign) →
`release` (checksums, signatures, `gh release create` which creates the tag)
→ `smoke` (pulls the image, checks `--version`, both arches, the five download
files, both MCP binaries, and the signature).

Expect roughly 45–70 minutes end to end; the macOS leg and the two image
builds dominate.

## Idempotency

| thing | on a repeat push at the same version |
|---|---|
| tag `vX.Y.Z`, the Release, its assets | untouched — never re-uploaded, never clobbered |
| image `:X.Y.Z` | untouched |
| image `:latest`, `:sha-<7>` | moved to the new build |
| bridge binaries inside the image | rebuilt (they live in the image, not only in the Release) |

A tag that exists at a **different** commit with no Release behind it fails the
run with instructions: nothing consumed it, so delete it
(`git push origin :refs/tags/vX.Y.Z`) and re-run, or bump the version.

## Step A0 — adopting an *unpublished* core (the patched path)

Most core versions are adopted here before they are on crates.io: the sibling
`../systemprompt-core` checkout is bumped, this repo is patched onto it via
`[patch.crates-io]`, and the two are proven together *before* core publishes.
This is the `next` branch's steady state, and `production` — a staging
environment — is deployed from it as a preview of the coming release.
`just core-bump` deliberately refuses to run in this state — it is the
published-crates path — so this step is by hand and nothing reminds you.

**The patch paths point at `.vendor/systemprompt-core`, not at the sibling.**
`just vendor-core` rsyncs the sibling checkout (working tree included) into
`.vendor/`, because the Docker build context is `COPY . /src` and cannot reach
`../systemprompt-core`. Re-vendor after every core edit; `just deploy-next` does
it for you, and a bare `just deploy` on `next` does not.

**Bump the pins first, and bump all of them.** A version requirement that no
longer matches the patched crate does not error: cargo silently drops the
patch and resolves the old version from crates.io, so the build "works" while
proving nothing about the new core. The pins live in **two** manifests, because
`tests/` is a separate workspace with its own copy:

```bash
grep -rn '^systemprompt[a-z-]* = { version = "' --include=Cargo.toml . | grep -v target
sed -i 's/OLD/NEW/g' Cargo.toml tests/Cargo.toml     # or let the script do it
scripts/sync-release-version.sh NEW --check          # core-pin lines must be silent
```

`sync-release-version.sh` covers every core pin in both manifests plus a
residual sweep that fails on any core pin it does not itself move — so a pin
added to a new crate cannot sit stale. Its remaining `DRIFT:` lines on this
path are the *product* version (workspace version, Chart.yaml, deploy files);
those belong to Step A, not here. Do **not** bump them for a core that has not
shipped.

Then prove it, in this order — each step catches a class the previous one
cannot:

```bash
just build                                    # patch resolved? log must read the new version
just clippy
grep -n 'Breaking' ../systemprompt-core/CHANGELOG.md   # then grep this repo for each item
./target/debug/systemprompt infra db migrate  # new core migrations, against the local DB
./target/debug/systemprompt --version         # must print the new core version
just start && curl -s localhost:8080/health   # must reach {"status":"healthy"}, not "starting"
```

Confirm the build log names `systemprompt-* vNEW (.../.vendor/systemprompt-core/...)`.
A build that compiles registry crates instead is a dropped patch, not a pass.

Then `just core-pin` so `bridge/CORE_REF` names the core commit you vendored
(push core first — CI checks that ref out of GitHub), and push `next`:
`gates.yml` runs the same tiers as `just verify` against that core.

Three things no gate catches on this path:

- **A tightened identifier validator is a runtime panic, not a compile error.**
  Core's `define_id!(…, validated, …)` types panic in `new()` on a value they
  used to accept, so a construction site that stops being legal still compiles
  and still passes clippy — it fails only when that code path executes. 0.29.0
  did exactly this to `ContextId` (now UUID-v4 only), and
  `hooks_track::build_request_context` had been passing `ContextId::new("")`,
  which would have panicked on every `/hooks/track` AI summary. Sweep for it
  whenever the core diff touches `crates/shared/identifiers`:
  `grep -rn '::new("' --include='*.rs' extensions/ src/` — and prefer
  `try_new` or `generate()` over a literal at any site that cannot prove the
  value's shape.

- **Migrations run silently and are not reversible.** Run them and then check
  the tables the core changelog describes actually exist, rather than trusting
  the success line.
- **A new core job is inert until this repo schedules it.** Core discovers jobs
  by inventory; whether one *runs* comes from `services/scheduler/config.yaml`.
  Boot warns `job is available in this build but has no scheduler.jobs entry`
  once per job and then carries on. Decide per job — scheduling it and
  deliberately leaving it off are both fine, silently missing it is not.

Only once core is published on crates.io do you comment the two
`[patch.crates-io]` blocks (root and `tests/`, in lockstep) and continue with
Step A.

## Step A — adopt the published core (on `next`)

```bash
# comment out BOTH [patch.crates-io] blocks first (Cargo.toml, tests/Cargo.toml)
just core-bump X.Y.Z
```

Refuses with an active patch, then: `sync-release-version.sh X.Y.Z` (every
pin in both workspaces), `bridge/CORE_REF = vX.Y.Z`, `cargo update -w`,
migrations against the local DB, `just build`, `just clippy`. Then `just
prepare` if any query changed, exercise what the core changelog touches, and
run the full gate:

```bash
just preflight
git commit -am "release: build against published core X.Y.Z"
git push origin next          # gates.yml must be green here
```

## Step B — land on `main`

```bash
just release X.Y.Z
```

Clean tree, on `next`, patch blocks commented, pins and `CORE_REF` agree,
`HEAD == origin/next`, `main` is an ancestor, `just preflight` green — then
`git push origin HEAD:main`. That push is the release; watch it with
`gh run watch` or in the Actions tab. **Do not tag by hand**: `release.yml`
creates `vX.Y.Z` from the merge commit.

## Signing

Everything is signed with cosign keyless (Sigstore, GitHub OIDC), verifiable
with:

```bash
cosign verify-blob \
  --certificate-identity-regexp='https://github.com/Astound-Digital/systemprompt-astound/' \
  --certificate-oidc-issuer='https://token.actions.githubusercontent.com' \
  --signature SHA256SUMS.sig --certificate SHA256SUMS.pem SHA256SUMS
```

OS-native signing is **off until the secrets exist**, and switching it on is
adding them — the workflow detects them and nothing else changes:

| platform | secrets | effect |
|---|---|---|
| macOS | `APPLE_CERT_P12_BASE64`, `APPLE_CERT_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_API_KEY_P8_BASE64`, `APPLE_API_KEY_ID`, `APPLE_API_ISSUER_ID`, `KEYCHAIN_PASSWORD` | Developer ID signature, notarized and stapled DMG (`scripts/package-bridge-macos.sh`, API-key mode) |
| Windows | `WINDOWS_CERT_PFX_BASE64`, `WINDOWS_CERT_PASSWORD` | Authenticode via `osslsigncode`, timestamped |

Absent, the run warns, the DMG is ad-hoc-signed (`UNSIGNED=1`) and the exe
unsigned, and the release notes say so. Nothing else is needed from GitHub:
`GITHUB_TOKEN` covers packages, releases and the OIDC token for cosign.

## First-run checklist (one-off, before the first push to `main`)

- [ ] Repository → Actions → General → *Workflow permissions: read and write* (or rely on the per-workflow `permissions:` block, which is set).
- [ ] After the first successful run: Packages → `systemprompt-astound` → *Manage access* → grant the team read.
- [ ] Branch protection on `main`: require the `Gates` checks; allow fast-forward only.
- [ ] Optionally add the signing secrets above.

## Fly (our own production) and other targets

`just deploy` from a `main` checkout still builds locally and pushes to the
Fly tenant — the registry and tag are minted by the systemprompt.io control
plane at deploy time, so it stays outside CI. `just downloads-fetch` backfills
the bridge assets from the live host as before; with a release out they can
equally come from `gh release download vX.Y.Z`. Colleagues on their own
infrastructure (Oracle VM, remote Postgres) use the published image:
[install/docker.md](install/docker.md).

## Rollback

1. Image users: `SYSTEMPROMPT_TAG=<previous> docker compose up -d`. Tags are
   never rewritten, so the previous build is exactly what it was.
2. Fly: `just deploy` from the previous release commit.
3. Never reuse or move a released tag — fix forward and cut the next version.
4. Core migrations are forward-only; roll back only to a version whose
   migrations match the database.
