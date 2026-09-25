# Releasing

How a new `systemprompt` core version becomes a release of this repo: a
published image, signed bridge binaries for three platforms, and a GitHub
Release — produced by CI on every push to `main`.

**Nothing is built by hand any more.** `.github/workflows/release.yml` runs on
every push to `main` and publishes everything; `.github/workflows/gates.yml`
runs the quality gate on every push to `next` and ordinary PRs; frozen promotion
PRs verify the existing push proof, and
`.github/workflows/coverage.yml` measures coverage on `main` and nightly. The branch
contract that makes this safe — `next` on the vendored core `next`, `main` on
the published core — is in [BRANCHING.md](BRANCHING.md); read it first.

## Versioning policy

Astound and core carry **separate** versions. Astound's version (the workspace
`version`, `bridge/Cargo.toml`, the profile template, the operator docs) → git
tag `vX.Y.Z` → image tag `X.Y.Z`; `scripts/sync-release-version.sh X.Y.Z`
writes it and `--check` gates it. The core pins in both manifests and
`bridge/CORE_REF = vC.D.E` name the published core it builds on;
`scripts/sync-core-version.sh C.D.E` writes them and `--check` gates them. A
core release normally means an astound release, but an astound release (a
fix, or a re-release after a failed run) needs no core release: bump only
astound's version.

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
| bridge, macOS self-updater | Release only (the gateway's `/v1/bridge/latest` feed proxies it; `bridge_releases.assets.macos`) | `astound-bridge-macos.zip` + `.sha256` — the same stapled `.app` as the dmg, zipped with `ditto` | cosign; Apple notarization |
| bridge, Windows x64 | same | `astound-bridge-windows.exe` + `.sha256` | cosign; Authenticode when the `WINDOWS_CERT_*` secrets exist |
| installer | same | `install.sh` (release copy points at the release; image copy at the gateway) | cosign keyless |
| checksums | Release | `SHA256SUMS` + `.sig`/`.pem` | cosign keyless |

Job order, and why: `version` (asserts the branch contract and reads the
version) → `bridge`
matrix (four runners; dependency caches exclude workspace outputs) → `image`
per arch (downloads the bridge artifacts into
`storage/files/downloads/` **before** building, because the image serves them;
pushes by digest) → `image-merge` (one manifest list, tagged only
`:sha-<7>`, cosign) → `smoke` (pulls the image, checks `--version`, both arches,
the five download files, both MCP binaries, and the signature) and
`upgrade-boot` (boots it over every seeded release) → `promote-tags` (points
`:latest` and, when absent, `:X.Y.Z` at that proven digest) → `release`
(checksums, signatures, `gh release create` which creates the tag).

`:X.Y.Z` is create-once and an org package version cannot be deleted from this
account, so it is written only after the image has passed smoke and
upgrade-boot. A failed run leaves its `:sha-<7>` and nothing else.

The release does not repeat the Gates matrix. `just release` requires the latest
`gates.yml` push run on the exact `next` SHA to have completed successfully,
including `Gates passed`. It freezes that candidate and the current `main` base
in a promotion ref and opens a PR. The PR verifies the push proof, unchanged
head/base, ancestry and the GitHub merge tree. Missing, pending, cancelled and
failed proof all block promotion. Ordinary PRs still run the full Gates matrix.
Coverage measures `main`, nightly and on demand; it is not a release check.

A passing build or CI run does not establish browser acceptance. Validate the
exact candidate with the repository preflight and browser acceptance gates
before promotion.

## Idempotency

| thing | on a repeat push at the same version |
|---|---|
| tag `vX.Y.Z`, the Release, its assets | untouched — never re-uploaded, never clobbered |
| image `:X.Y.Z` | untouched |
| image `:latest`, `:sha-<7>` | moved to the new build once it passes smoke and upgrade-boot |
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

**The patch paths point at the sibling `../systemprompt-core`.** The image is
built from pre-built binaries, so no copy of core needs to live in this repo.
`just deploy` refuses (via `just core-guard`) unless that checkout is clean and
at `bridge/CORE_REF`, so what ships is always a committed, pinned core.

**Bump the pins first, and bump all of them.** A version requirement that no
longer matches the patched crate does not error: cargo silently drops the
patch and resolves the old version from crates.io, so the build "works" while
proving nothing about the new core. The pins live in **two** manifests, because
`tests/` is a separate workspace with its own copy:

```bash
grep -rn '^systemprompt[a-z-]* = { version = "' --include=Cargo.toml . | grep -v target
sed -i 's/OLD/NEW/g' Cargo.toml tests/Cargo.toml     # or let the script do it
scripts/sync-core-version.sh NEW --check            # must be silent
```

`sync-core-version.sh` covers every core pin in both manifests plus a
residual sweep that fails on any core pin it does not itself move — so a pin
added to a new crate cannot sit stale. Astound's own version is
`sync-release-version.sh` and belongs to Step A, not here.

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

Confirm the build log names `systemprompt-* vNEW (.../systemprompt-core/...)`.
A build that compiles registry crates instead is a dropped patch, not a pass.

Then `just core-pin` so `bridge/CORE_REF` names the core commit you built against
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
  the success line. What proves an upgrade from a *deployed* database is the
  ladder in `tests/integration/schema-upgrade`: every rung is restored and
  migrated forward by the current installer on each push, so a migration that
  only works on a database that already booted on some newer schema (the
  2026-09-21 self-host failure) fails here before it ships. A rung older than
  the floor is a customer database nothing tests — raise the floor only when
  no such instance exists.
- **A new core job is inert until this repo schedules it.** Core discovers jobs
  by inventory; whether one *runs* comes from `services/scheduler/config.yaml`.
  Boot warns `job is available in this build but has no scheduler.jobs entry`
  once per job and then carries on. Decide per job — scheduling it and
  deliberately leaving it off are both fine, silently missing it is not.

Only once core is published on crates.io do you comment the two
`[patch.crates-io]` blocks (root and `tests/`, in lockstep) and continue with
Step A.

## Step A — adopt the published core (on `next`)

Before publication, prepare the changelog, version pins, `bridge/CORE_REF`, and
removal of both `[patch.crates-io]` blocks. Once every required core crate exists
on the sparse index, run `cargo update -w` in root, `tests/` and `bridge/` and
commit all three lockfiles. Record the new rung of the schema-upgrade ladder
(`just schema-baseline` writes `tests/fixtures/schema/release-baseline-X.Y.Z.sql`
from a fresh local install; the ladder is append-only, one rung per release
since 0.52.0, and `scripts/check-schema-baseline.sh` refuses a missing rung);
regenerate SQLx caches when query/schema inputs changed.

Build or clippy only code broken by the core upgrade. If core's bridge GUI
changed, run `cargo check --manifest-path bridge/Cargo.toml --target
x86_64-pc-windows-gnu` before pushing. Do not run the local lint or application
test suites for release adoption: the exact `next` push runs the exhaustive
Gates matrix and is the release proof. A successful downstream CI run needs no
additional local build verification.

`just core-bump` remains a local development convenience that builds and lints;
the release lane uses its individual version and lockfile steps above.
Database migration commands must explicitly select `--profile local`; a bare
command can select the active session's cloud profile.

**Lockfile drift.** `scripts/check-core-crate-versions.sh` (run by
`preflight-static` and the `static` gate) fails when any `systemprompt-*`
crate resolves to more than one version across `Cargo.lock`,
`tests/Cargo.lock` and `bridge/Cargo.lock`. Two copies of one crate at
different versions surfaced during 0.51.0 as a compile error that named
neither lockfile; see [BRANCHING.md](BRANCHING.md#three-lockfiles-one-core).

**Secrets moved out of the environment (core 0.52.0).** Core no longer reads
`PGCA_CERT_PATH` (name a private CA with `?sslrootcert=` in the database URL),
`MCP_CREDENTIAL_BROKER_SECRET` (now `secrets.custom["MCP_CREDENTIAL_BROKER_SECRET"]`)
or `gateway.bridge_releases.token_env` (renamed `token_secret`, a key in the
secrets document). With the production profile's `secrets.source: env`, every
such key must be listed in the Fly app's `SYSTEMPROMPT_CUSTOM_SECRETS` as well
as set — check `flyctl secrets list` before the deploy that adopts 0.52.0.

**Docker.** `just deploy` puts `/usr/bin` ahead of `PATH` when
`/usr/bin/docker` exists and warns when `docker` still resolves elsewhere: a
wrapper shim ahead of the real binary failed `docker build` during 0.51.0
with an error unrelated to the shim.

## Step B — land on `main`

```bash
just release X.Y.Z
```

Run from clean `next` or a detached release worktree whose HEAD equals
`origin/next`. Both patch blocks must be absent, all version pins and `CORE_REF`
must agree, the schema baseline must be current, and `main` must be an ancestor.
The recipe verifies the latest successful next-push Gates result, freezes the
candidate and base in `promote/X.Y.Z/<base-sha>/<head-sha>`, and opens a PR.

Record the PR and workflow run IDs before waiting. Once the PR's `Gates passed`
proof completes, repeat `just release X.Y.Z`. The recipe revalidates the exact
candidate, base and merge tree, merges through GitHub with the expected head SHA,
and verifies the merged parents and tree. It never pushes directly to `main`.
The merge starts `release.yml`, whose version job independently verifies the
merged parents/tree and both successful proofs before publication. Watch that
run and verify its image and assets
by content. Do not tag by hand: the workflow creates `vX.Y.Z` at the merge commit.

## Signing

Everything is signed with cosign keyless (Sigstore, GitHub OIDC), verifiable
with:

```bash
cosign verify-blob \
  --certificate-identity-regexp='https://github.com/Astound-Digital/systemprompt-astound/' \
  --certificate-oidc-issuer='https://token.actions.githubusercontent.com' \
  --signature SHA256SUMS.sig --certificate SHA256SUMS.pem SHA256SUMS
```

Apple signing credentials are required for a release. Missing credentials stop
the macOS job before compilation. Windows signing remains optional:

| platform | secrets | effect |
|---|---|---|
| macOS | `APPLE_CERT_P12_BASE64`, `APPLE_CERT_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_API_KEY_P8_BASE64`, `APPLE_API_KEY_ID`, `APPLE_API_ISSUER_ID`, `KEYCHAIN_PASSWORD` | Developer ID signature, notarized and stapled DMG (`scripts/package-bridge-macos.sh`, API-key mode) |
| Windows | `WINDOWS_CERT_PFX_BASE64`, `WINDOWS_CERT_PASSWORD` | Authenticode via `osslsigncode`, timestamped |

Both the macOS app and DMG are notarized and stapled. Local `UNSIGNED=1` or
`SKIP_NOTARIZE=1` builds exit 2 and never stage downloads. The just recipe
revalidates the staged DMG even on a coordinator cache hit. Without Windows
credentials the exe remains unsigned and the release notes say so.
Nothing else is needed from GitHub:
`GITHUB_TOKEN` covers packages, releases and the OIDC token for cosign.

### Provision Apple credentials for CI

Run `bash scripts/provision-apple-signing.sh` on a Mac with the existing
Developer ID certificate/private key and App Store Connect API key. Set
`APPLE_CERT_PEM`, `APPLE_PRIVATE_KEY_PEM`, `APPLE_NOTARY_KEY` to their file paths,
and `APPLE_API_KEY_ID` and `APPLE_API_ISSUER_ID` to the App Store Connect metadata.
`APPLE_SIGNING_REPO` defaults to `Astound-Digital/systemprompt-astound` and
`APPLE_TEAM_ID` must match the signing certificate’s development team.

The script validates the matching key pair, certificate expiry, team and
notary API access before uploading. It generates a password-protected PKCS12
and a separate random runner-keychain password, pipes secrets into `gh secret
set` for local encryption, and removes the temporary PKCS12 on exit. No secret
belongs in git or workflow inputs. Both normal releases and repairs use
`.github/actions/apple-signing` and delete their ephemeral keychain afterward.

## Versions in documentation

`scripts/sync-release-version.sh X.Y.Z` (astound's version) also rewrites
the image pins in the operator docs (`deploy/self-host/*.md`,
`docs/install/*.md`), and its `--check` refuses any other `0.x.y` there. Hosted
pages under `services/content/` never carry a literal version: they write
`@RELEASE_VERSION@`, which ingestion replaces with the crate version at boot,
so the site can only ever describe the release it is running.
`scripts/check-docs-version.sh` (a lint gate) enforces both halves.

## First-run checklist (one-off, before the first push to `main`)

- [ ] Repository → Actions → General → *Workflow permissions: read and write* (or rely on the per-workflow `permissions:` block, which is set).
- [ ] After the first successful run: Packages → `systemprompt-astound` → *Manage access* → grant the team read.
- [ ] Branch protection on `main` requiring `Gates passed` — unavailable on the Free plan for a private repo; `just release` enforces it instead (`scripts/check-gates-green.sh`).
- [ ] Provision the required Apple signing secrets above; optionally add Windows signing.

## Fly (our own production) and other targets

`just deploy-release X.Y.Z` ships a published release to the Fly tenant — the
registry and tag are minted by the systemprompt.io control plane at deploy
time, so it stays outside CI. It works from a clean worktree of `origin/main`
(the tag must *be* `origin/main`), installs the release's server binaries and
bridge downloads and verifies them against the release `SHA256SUMS`, renders
`web/dist` there, deploys, requires both the Fly image digest and update timestamp to move, waits for a
healthy response at the target version, then watches
`/health` for five minutes and re-hashes the served downloads. `DEPLOY_DRY_RUN=1`
runs everything up to the deploy. `just deploy` remains the *preview* path:
it builds and ships whatever this working tree holds. Colleagues on their own
infrastructure (Oracle VM, remote Postgres) use the published image:
[install/docker.md](install/docker.md).

## Upgrading across several releases

A database is upgraded straight from any release on the ladder (0.56.0 and
later) to the current one in a single boot. Three guarantees make that safe;
each exists because a 0.58 → 0.60 client upgrade failed without it:

- **Migrations never run stale row triggers.** The runner suspends every
  enabled row trigger on the tables a migration writes, for that migration
  only, and restores exactly those. A migration that needs its triggers to
  fire declares `-- @cost: … triggers=live`. Never toggle a trigger by name in
  a migration (`ALTER TABLE … DISABLE TRIGGER <name>`): it fails on a
  database where the trigger is already retired, and boot refuses it.
- **Retirements run before any migration.** A trigger, function or view an
  extension retires goes in its `retirements()` (astound:
  `extensions/web/schema/retire/*.sql`, `DROP … IF EXISTS` only), not only in
  a migration: core migrates first, and a table core drops must not outlive
  the trigger that writes it.
- **Boot refuses a dangling trigger routine.** Once the schema is final, an
  enabled trigger whose function uses a relation that does not exist stops the
  boot with the trigger, function and relation named.

`docker/entrypoint.sh` runs `infra db migrate --repair-drift`: checksum drift
is repaired and retried once; any other failure stops with core's hint.
`/app/storage/data` must be a persistent volume writable by uid 1000 (see
`deploy/self-host/README.md`).

**A client stuck on 0.56–0.58 with an older image** (symptoms: ai 032
statement timeout, web 093 `relation "analytics_ingestion_producers" does not
exist`, web 102 `trigger "feedback_capture" … does not exist`, or `Cannot
create gateway journal`) upgrades cleanly to a release carrying these fixes.
Before that release, the manual route was: drop every `feedback_capture*`
trigger (`extensions/web/schema/retire/001_feedback_ingestion.sql` lists
them), create a no-op `feedback_capture` trigger on `ai_requests` so web 102
has something to toggle, and `chown -R 1000:1000` the `app_storage_data`
volume.

## Rollback

1. Image users: `SYSTEMPROMPT_TAG=<previous> docker compose up -d`. Tags are
   never rewritten, so the previous build is exactly what it was.
2. Fly tenant image tags are overwritten. Record the old digest before deploying;
   this release procedure provides no rollback. Fix a failed deployment forward.
3. Never reuse or move a released tag — fix forward and cut the next version.
4. Core migrations are forward-only; roll back only to a version whose
   migrations match the database.
