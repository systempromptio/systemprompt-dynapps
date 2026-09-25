# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

# Astound Digital

**Nothing in this file is a startup routine.** It describes how to do work in this
repo *once you have been given a task*. On arrival, run no commands and read no
state — wait for the task, then come back for the part that applies.

**When a task needs a command you don't know, discover it from the CLI.**
`systemprompt --help` is your starting point.

---

## Branching (this repo)

Two branches, two different cores. Which one you are on decides where
`systemprompt-*` resolves from, and nothing else in the repo changes.

**`main` builds against the published core crates.** The `[patch.crates-io]`
blocks in `Cargo.toml` and `tests/Cargo.toml` are commented out and the pins sit
at the released version.

**`next` builds against core `next`** — a preview of the next release, shipped
before it is perfect. The patch blocks are **active** and point straight at the
sibling checkout, `../systemprompt-core`; there is no vendored copy. The
production image copies **pre-built** binaries (the profile Dockerfile), so the
core source never has to be inside this repo. `just deploy` runs `core-guard`
first: with the patch active it refuses unless `../systemprompt-core` is clean
and its HEAD equals `bridge/CORE_REF` — that is what "build `next` and `next`
together" means, and it is the only way the image, CI and the bridge all see
the same core commit. Commit core on its `next`, push, `just core-pin`, then
deploy.

**The version pins must match the sibling core's workspace version.** For a 0.x
crate, `version = "0.40.0"` means `>=0.40.0, <0.41.0`, so a patch pointing at a
0.41.0 core **does not satisfy it and is silently dropped** — cargo falls back to
the published crate and the build passes having proved nothing. Bump the pins in
**both** workspaces (`Cargo.toml` and `tests/Cargo.toml`; `[patch]` is
per-workspace) and check with `scripts/sync-core-version.sh <core-version> --check`.
Astound's own version is separate (`scripts/sync-release-version.sh`): an
astound release does not need a core release.
The only proof the patch is live is the build log: it must name
`../systemprompt-core` paths. A bare `v0.40.0` with no path means the pins
are still wrong.

****`production` in this repo is a staging environment**, so `next` deploys to it
as a preview. For a published release, remove both patch blocks, prepare the
version pins and `bridge/CORE_REF`, refresh all three lockfiles, and push `next`.
The exact next-push Gates run is the proof; do not repeat application lint or
tests locally for release adoption. `just release X.Y.Z` opens a frozen PR onto
`main`; repeat the command after its lightweight proof succeeds to merge it.
A clean detached worktree at `origin/next` is supported. See `docs/RELEASING.md`.
`bridge/CORE_REF` names a core SHA while patches are active, and `vX.Y.Z` when
using published crates (`scripts/check-core-ref.sh` enforces the latter).

**Core (`../systemprompt-core`) is write-only from here.** When patch-active
work needs a core change: write the functionality, commit on core's `next`
branch, push. Run NO validation in the core checkout — no tests, no clippy, no
fmt, no cargo check loops; core's CI on `next` is its validation surface. All
building and testing happens in this repo, which compiles the patched core
anyway.

## Quick Start

```bash
# First-time setup: writes .systemprompt/profiles/local/, starts Docker Postgres,
# runs publish_pipeline. With no key arg, the CLI prompts for which provider to
# use; the chosen provider becomes ai.default_provider (others disabled) and the
# gateway default. Passing keys is non-interactive — the first becomes default.
just setup-local                                                          # interactive provider pick
just setup-local <anthropic_key> [openai_key] [gemini_key] [http_port=8080] [pg_port=5432]
# setup-local first tries `just fetch-release` (server + MCP binaries from the
# GitHub Release tarball into target/release/) and only compiles if that fails.
just fetch-release [X.Y.Z]                                                # no toolchain needed

# Build (auto-uses live DB if reachable, else SQLX_OFFLINE=true)
just build            # debug
just build --release  # release

# Lint (workspace, -D warnings, same offline fallback as build)
just clippy

# Regenerate .sqlx/ offline query cache (needs live DB)
just prepare

# Start services
just start

# Discover CLI commands
systemprompt --help

# List skills
systemprompt core skills list
```

---

## Shared Build State (read this when you are about to compile)

Several agents work this clone at once. Builds, clippy, and tests are expensive
and take a shared cargo lock, so a build started mid-iteration stalls everyone.

**Do all the work first, validate once at the end.** Never run `just build` or
`just clippy` between edits to see how you're doing; finish the change set, then
run the gate a single time.

**Once a task has you compiling, check the shared state before spending anything**
(these are pre-compile checks, not things to run on arrival):

```bash
just build-status     # in-flight run + last result per recipe (a record, never a reason to skip)
just server-status    # running server, its binary, and whether that binary is stale
```

`just build`, `just clippy`, `just test-*`, and `just lint-gates` are
single-flight (`scripts/build-coordinator.sh`). One rule: **one build in
flight at a time, always of the latest source.** Every call compiles the tree
as it is (cargo is incremental, so an unchanged tree costs seconds); there is
no "already built, skip" cache — one existed until 2026-09-18 and returned
"already green" while a bare `cargo build` from another tree had overwritten
`target/debug/systemprompt` with stale code.

| situation | what happens |
|-----------|--------------|
| a run of this exact source is in flight | you are told so, attach to its log, exit with its status |
| a run of different source is in flight | you are told so, wait for it, then run over the latest tree |
| nothing in flight | you lead |

Results land in `.build/` (gitignored): `runs.jsonl`, `latest/<recipe>.json`,
`logs/`, `binaries.jsonl`. Read them instead of re-running.

`just start` reports the running server first, then starts. It does not restart
a server another agent is already running (say so and stop), and it warns when
the binary predates the current source, but it only refuses outright when there
is no binary at all. Staleness is reported from the ledger when the binary came
from a coordinated build, and from file mtimes otherwise.

**Orphaned MCP children on macOS.** Core's two halves of child supervision have
different reach. Parent-death prevention is `prctl(PR_SET_PDEATHSIG)` and stays
Linux-only, so a `SIGKILL`ed or crashed server on macOS leaves its MCP servers
reparented to `launchd`, still holding their ports. Identity verification now
works on both platforms (`/proc/<pid>/environ` on Linux,
`sysctl(KERN_PROCARGS2)` on macOS), so the next `just start` recognises those
orphans as its own and reclaims the ports — no manual cleanup, no
`PortOwnedByForeignProcess`. If startup instead reports `PortHolderUnverifiable`,
the platform genuinely cannot check identity and the process must be stopped by
hand. `just stop` shuts services down cleanly and avoids the orphaning entirely.

Always go through the justfile. A bare `cargo build` bypasses coordination and
re-creates the contention. Escape hatches when you truly need them:
`START_FORCE=1`, `BUILD_NO_COORD=1`.

---

## Preflight (the gates; CI re-runs them on every push)

```bash
just preflight          # optional development confidence: static → lint → tests → coverage
just preflight-static   # seconds: fmt, sqlx cache, 25 source gates
just preflight-lint     # clippy (both workspaces), doc-check, msrv-check
just preflight-full     # weekly: preflight + deny + audit + machete + hack
just init-hooks         # once per clone: tracked .githooks/ (pre-commit only)
```

`verify` = preflight minus coverage, for development iteration.
`.github/workflows/gates.yml` runs the full matrix on each `next` push and
ordinary PRs. Frozen release PRs verify that exact successful push run, its
aggregate job, the frozen head/base and the merge tree without repeating tests.
Coverage is a measurement on `main` and nightly, not a release check.
`just release X.Y.Z` promotes through a PR; direct pushes to `main` are forbidden.
The merged commit starts `.github/workflows/release.yml`, which validates the
promotion proof before publishing the image, bridge binaries and GitHub Release.
Contract and procedure: `docs/BRANCHING.md`, `docs/RELEASING.md`.

**Coverage floor + ratchet.** `just coverage` runs an instrumented llvm-cov
pass over all three workspaces (root, `tests/`, `bridge/`) into
`coverage-report/` (gitignored); `just coverage-check` enforces the tracked
`coverage/baseline.json` — a global floor, a 0.5pt total ratchet, and per-crate
ratchets. If you raised coverage, re-record with `just coverage-baseline` and
commit the file; lowering it is a review-visible act. `coverage-check` refuses
to record a baseline less than half the previous total — a run that lost its
instrumented binaries reports 0.00%, and that is a broken run rather than a
regression. The README badge is rendered from the same file by
`just coverage-badge`, and the `coverage-badge.sh` gate fails if the two
disagree, so a re-recorded baseline cannot leave a stale number on the front
page. Never use cargo-llvm-cov
here (it re-injects the mold linker flags and silently produces zero profraws
— see `scripts/coverage.sh`).

**The schema-upgrade ladder.** `tests/fixtures/schema/release-baseline-<X.Y.Z>.sql`
holds one recorded schema per release since 0.56.0 (the floor; see `scripts/check-schema-baseline.sh`); `tests/integration/schema-upgrade`
restores each rung and runs the current installer over it, then diffs against a fresh
install. `scripts/check-schema-baseline.sh` (a static gate) requires a rung for every
release tag from the floor and one named for the workspace version, so `just
core-bump` turns it red until `just schema-baseline` records the new rung. A rung for
a release that predates the ladder is recorded from its published binary with
`just schema-baseline X.Y.Z`. Never edit a rung by hand.

**Every test lives in the `tests/` workspace** (`unit/`, `integration/`,
`contract/`) — there is exactly one home. Inline `#[cfg(test)]` modules are
banned, and so are in-crate `extensions/**/tests/` dirs: a test that compiles
as part of the crate it tests is a test the tests workspace cannot see. The
`tests/unit/*` crates are libraries, so a test is a module in `src/` declared
in `lib.rs` as `#[cfg(test)] mod <name>;`, not a file in a `tests/` dir.
DB-backed suites need Docker Postgres up (`just db-up`).

```bash
just test-unit          # tests/unit/* (mcp-unit-tests + web-unit-tests)
just test-integration   # DB-backed; throwaway mcp_ext_test_* databases
just test-contract      # every admin route x three principals, diffed vs baseline
just test               # all three tiers, in that order
```

**Running one test.** The single-flight recipes always run the whole tier, so
drop to `cargo nextest` directly with a filter — this bypasses the coordinator,
so keep it to iteration and finish with the `just` recipe:

```bash
cargo nextest run -p systemprompt-web-admin --tests -E 'test(list_top_users)'
cargo nextest run --manifest-path tests/Cargo.toml -p web-unit-tests some_name
# DB-backed suites need the maintenance DB, normally derived from the local
# profile; override explicitly with:
SYSTEMPROMPT_TEST_DATABASE_URL=postgres://…/postgres \
  cargo nextest run --manifest-path tests/Cargo.toml -p admin-db-core-tests
```

The contract suite pins HTTP status per route in
`tests/contract/admin/baseline.txt`. A status change fails the run; if it is
intended, re-record with `UPDATE_CONTRACT_BASELINE=1` and call it out in the PR.

---

## CLI Structure

```
systemprompt <domain> <subcommand> [args]
```

| Domain | Purpose |
|--------|---------|
| `core` | Skills, content, files, contexts, plugins, hooks, artifacts |
| `infra` | Services, database, jobs, logs |
| `admin` | Users, agents, config, setup, session |
| `cloud` | Auth, deploy, sync, secrets, tenant, domain |
| `analytics` | Overview, conversations, agents, tools, requests, sessions, content, traffic, costs |
| `web` | Content-types, templates, assets, sitemap, validate |
| `plugins` | Extensions, MCP servers, capabilities |
| `build` | Build core workspace and MCP extensions |

**Use `systemprompt <domain> --help` to explore any domain.**

---

## CLI Discovery Workflow

When you need to perform a task, use the CLI help to find the right command:

```bash
# Top-level help
systemprompt --help

# Domain help
systemprompt core --help
systemprompt infra --help

# Subcommand help
systemprompt core skills --help
systemprompt core skills show --help
```

---

## Architecture (big picture)

- `src/main.rs` is a thin entry point that delegates to the published `systemprompt` core crates (sibling checkout at `../systemprompt-core`, patched in via `[patch.crates-io]` for cross-repo work). All customization is **compile-time** via the [`inventory`](https://docs.rs/inventory) crate — there is no dynamic plugin loader.
- Rust code lives in `extensions/`: `extensions/mcp/*` for MCP server extensions, `extensions/web` for page data and template rendering. Each MCP extension has its own crate with `Cargo.toml` + `.sqlx/` offline cache.
- Configuration is YAML under `services/`, loaded through `services/config/config.yaml`'s explicit `includes:` list. Unknown keys error loudly (`#[serde(deny_unknown_fields)]`).
- Governance is a four-stage synchronous pipeline on every tool call: **scope check → configured secret scan → blocklist → rate limit**. Every decision is audited to Postgres with a trace_id linking identity → agent → tool → result → cost. **All four stages are enabled in this installation** (`services/governance/config.yaml`), as are the gateway safety scanners (`services/gateway/policies.yaml`) — both were switched on 2026-08-25 for the enterprise rollout, and both files carry the tunables and the reasoning. A missing governance file falls back to core's vendor-neutral warn-only chain; this installation declares every stage and signature explicitly. Authentication is separate and is never disabled — an invalid or expired token is denied with `policy=authentication`.

  The safety scanners are a **second, independent plane** from the governance chain: the chain is request-only and runs first; the scanners run both directions. Which categories *block* is deliberately narrower than which are *scanned* — every finding is audited regardless. `jailbreak` and `pii_credit_card` are audit-only after both denied ordinary developer traffic in production on 2026-08-31; see the header of `services/gateway/policies.yaml`.
- Per-clone Docker Postgres: `just db-up / db-down / db-logs [tenant=local]`. Project name is derived from a hash of the repo path, so multiple clones on one host get isolated containers and volumes. There is no destructive reset recipe — recover migration checksum drift in place with `just repair-migrations`.
- Deploy flow: `just build-all` (release binary + MCP servers + web assets) then `just deploy`. The `publish_pipeline` job also runs automatically at server startup.

### Three cargo workspaces

`--workspace` covers only the root one. The other two need an explicit
`--manifest-path`, which is why `clippy`, `doc-check`, and `coverage` each run
three times:

| Workspace | Members | Note |
|-----------|---------|------|
| root (`Cargo.toml`) | `extensions/**` | `exclude = ["tests"]` |
| `tests/Cargo.toml` | `unit/`, `integration/`, `contract/` crates | separate so it never compiles when extensions are consumed as deps |
| `bridge/Cargo.toml` | the Claude Code client | depends on unpublished `systemprompt-bridge` **by path** — needs `../systemprompt-core` checked out (`just bridge-build` clones it) |

### Crate layering (enforced by `scripts/lint-layers.sh`)

```
facade (src/) -> aggregator (extensions/web) -> app (site, jobs)
  -> domain (admin, content) -> shared (web/shared, mcp/shared)
```

Membership is derived from a crate's directory, so moving a crate re-classifies
it. Three properties are checked and must hold exactly: no dependency points
upward, no cycles, and **no domain -> domain edges** — `admin` and `content` are
peers, so anything they both need belongs in `shared`. `extensions/cli/*` and
`extensions/mcp/*` sit at the entry tier beside the aggregator. Dev-dependencies
are exempt.

### Source-shape gates worth knowing before you write code

`just lint-gates` runs 25 independent read-only checks concurrently (the
`gates=()` array in the justfile is the list — trust the array, not this count;
every failure is reported, so one red gate never hides the rest). Two frontend
gates that used to live there — admin CSS classes and frontend standards — are
now cargo tests in `tests/unit/web/src/`, so they surface from `just test-unit`
rather than `lint-gates`. They locate the repo root by climbing until it is
found (`tests/unit/web/src/support.rs`), never by a fixed depth — a stale depth
makes every one of them walk an empty tree and pass. The ones that most often surprise:

- **No SQL in `extension.rs`** (`lint-extensions.sh`). Schema DDL lives in
  `<crate>/schema/**.sql` and is embedded with `include_str!`; migrations are
  discovered from `schema/migrations/NNN_<name>.sql` and surfaced by
  `extension_migrations!()`. A raw string literal (`r"…"`) or a hand-built
  `Migration::new` in an `extension.rs` fails the gate.
- **Every production source opens with a `//!` module head**
  (`check-file-headers.sh`). Line 1 must be the head itself or an inner
  attribute (`#![…]`) — nothing else may precede it.
- **A Dockerfile may not name what the repo no longer has**
  (`check-dockerfile-paths.sh`). Nothing in `gates.yml` builds the image, so a
  `COPY` of a deleted crate's binary passes all four CI jobs and fails only in
  `release.yml`, after `main` has moved. The gate checks every tracked Dockerfile
  **and workflow** for `systemprompt-mcp-*` binaries no manifest declares —
  release.yml's smoke step asserts them with `docker run --entrypoint ls
  /app/bin/<binary>`, so a deleted one fails the release there too — and checks
  the root Dockerfile's COPY sources against **git**, not the disk, because an
  untracked leftover surviving a `git rm` makes a broken COPY look fine locally
  while the release builds from a clean checkout. Rule 2 is root-only: that
  image alone is built with the repo root as its context (`COPY . /src`).
  `.systemprompt/profiles/production/docker/` is gitignored by design and sits
  outside this gate; `cloud deploy` refuses a stale COPY there itself.
- Comment density, inline comments, file size, duplicate types, dead repository
  code, admin template links/assets, and `services/` validity are all gates too
  — read the failing script's header comment, which states the rule and why.

`just clippy` additionally depends on two identity gates that are not in that
array, so they fail the lint tier rather than `lint-gates`:

- **`lint-no-synthesis`** — no `UserId::new("…")` with a string literal anywhere
  under `extensions/`. Every legitimate `UserId::new` takes an identifier
  validated from a cookie, query, JWT claim, or DB row. Genuine provisioning
  code belongs in `extensions/**/bootstrap/`, which is exempt.
- **`lint-no-untyped-admin`** — no `UserId::admin()` outside its sanctioned call
  sites (tests are exempt).

### Client / bridge

`bridge/` is the Claude Code client, built separately from the server.
`just bridge-build` builds it (and clones `../systemprompt-core`, which it needs
by path). `just claude <code>` runs it in a container scoped to this clone and
gateway — the host's `~/.claude` and `~/.config` are untouched; `just connect
<code>` is the host-configuring variant instead. Connect codes are single-use
with a 10-minute TTL, so build the bridge before issuing one. Headless code
issue: `systemprompt admin bridge issue-code --user-id <email>`.
`just claude-reset` signs this clone out (`ALL=1` for every clone on the host).

---

## Operating on production from this machine

Two separate auth planes; nothing tells you which one a command wants:

| plane | command | what it unlocks |
|-------|---------|-----------------|
| Cloud (Fly/registry) | `just login` (= `cloud auth login production`) | `just deploy` / `cloud deploy`, image push |
| Instance admin session | `systemprompt admin session login` (production profile active) | remote-routed CLI commands (`infra db`, `infra logs`, `analytics`, …) |

Remote routing needs BOTH of these true, or it fails with circular-looking
errors:

1. **`database.external_db_access: true`** in
   `.systemprompt/profiles/production/profile.yaml`. Without it the CLI
   resolves the Fly-internal DB hostname (`iad.…internal` — unresolvable off
   the Fly network) and bootstrap dies before anything runs. With it, the
   `external_database_url` secret (public `db.systemprompt.io`) is used and
   local `/app` path validation is skipped.
2. **An admin session for the production profile**:
   `systemprompt admin session switch production`, then
   `systemprompt admin session login`. The session is minted directly against
   the external DB and stored in `.systemprompt/sessions/`; remote-routed
   commands then execute on the tenant (`Remote` target) with that token.
   (Core older than the 2026-08-26 `next` fix refused `session login` on
   external-DB profiles — the guard exempted only `admin users`.)

A CLI **session token is not a gateway credential** — `/v1/messages` wants a
PAT or bridge JWT plus an `x-session-id` header; a session token fails with
`kid does not match any known signing key`. To exercise inference, use a PAT
(see Debugging below), never the session token.

## Debugging & Troubleshooting

```bash
# Quick error check
systemprompt infra logs view --level error --since 1h

# Debug AI request failures
systemprompt infra logs request list --limit 10
systemprompt infra logs audit <request-id>

# Debug MCP tool failures
systemprompt plugins mcp logs <server-name>

# Debug agent issues
systemprompt infra logs trace list --agent <agent-name> --status failed
```

**Key debugging workflow:**
1. `infra logs view --level error` — Find the error
2. `infra logs request list` — Find failed AI requests
3. `infra logs audit <id>` — Get full conversation context
4. `plugins mcp logs <server>` or `logs/mcp-*.log` — Get MCP tool errors

---

## Viewing Governance

Every inference call (`/v1/messages`) and every MCP tool call lands a row in the governance spine. Same CLI surface for both — no separate "gateway logs" vs "tool logs":

```bash
# Every AI request — user, model, token counts, cost, latency, status
systemprompt infra logs request list --limit 20
systemprompt infra logs request list --since 1h --provider anthropic   # request list filters: --since / --until / --user / --model / --provider / --before <cursor> (no --status)
systemprompt infra logs trace list --status failed          # only failed runs — --status lives on trace list, not request list

# Audit for one request — identity, policy evals, tokens, cost, message/tool counts;
# the transcript is opt-in and paged so a long session never comes back as one blob
systemprompt infra logs audit <request-id>
systemprompt infra logs audit <request-id> --messages --limit 20 --max-content 400
# Spend per user (id + display name, requests, tokens, distinct conversations)
systemprompt analytics costs breakdown --by user --since 30d

# Tool-call traces (PreToolUse → decision → spawn → result)
systemprompt infra logs trace list --limit 20
systemprompt infra logs trace list --agent <name> --status failed
systemprompt infra logs trace show <trace-id>

# Cost + usage rollups (hits the same audit table)
systemprompt analytics costs summary
systemprompt analytics requests stats
systemprompt analytics agents list
systemprompt analytics tools list
```

`logs request list` shows one row per `/v1/messages` hit — the gateway path Pi / any Anthropic-SDK client uses. `logs trace list` shows MCP tool calls. Both are backed by the same 18-column `ai_requests` / trace tables with `user_id`, `tenant_id`, `session_id`, `trace_id` — so `audit <id>` reconstructs the chain from identity to cost.

**`infra logs` vs `analytics` — operational vs dashboard.** The `infra logs request {list,stats}` commands are quick operational views (recent rows, by-provider / by-model aggregate). Their `analytics requests {list,stats}` counterparts are dashboard metrics over a time range with model filtering, cache-hit rate, and CSV export. Same `ai_requests` table underneath — reach for `infra logs` when triaging a live issue, `analytics` when reporting. The `--help` on each cross-references the other.

For live tailing while reproducing an issue: `infra logs stream` (`infra logs view` has no follow mode).

---

## Services Configuration

All runtime configuration lives as YAML under `services/`. The root `services/config/config.yaml` is a thin aggregator with an explicit `includes:` list — every **flat** resource file must be listed there. Skills and plugins are the exception: they are auto-discovered from their nested directories and must *not* be added to `includes:`.

```
services/
  config/config.yaml        Root aggregator (includes the flat resource files)
  mcp/<name>.yaml           Flat MCP server definitions
  skills/<id>/config.yaml   Skill definitions (nested dir, auto-discovered)
  plugins/<id>/config.yaml  Plugin binding descriptors (nested dir, auto-discovered)
  marketplaces/<id>/config.yaml  Marketplaces (nested dir, auto-discovered)
  artifacts/<id>/config.yaml     Dashboards (nested dir, auto-discovered)
  access-control/rules.yaml The one declarative source of entitlement (all bands, marketplaces included)
  governance/config.yaml    Policy chain (all four stages on — see Architecture)
  gateway/policies.yaml     Gateway quotas + safety scanners (both on here)
  ai/config.yaml            AI provider config
  scheduler/config.yaml     Job scheduler
  web/config.yaml           Web frontend config (full WebConfig)
  content/config.yaml       Content source config
```

Unknown YAML keys cause loud errors at load time (`#[serde(deny_unknown_fields)]`). Nested `includes:` resolve recursively. Plugin YAMLs are binding descriptors that reference top-level agents, skills, mcp servers, and content sources by id — never inline copies.

**This instance ships no A2A agents.** The one it used to ship, `admin_console`,
existed only because the inbound Slack pipeline could not dispatch to anything else;
Slack is gone and so is the agent. Everything on this instance is skills, MCP servers,
and artifacts — `admin agents list` returning zero rows is the correct answer.

---

## Critical Rules

1. **Core is a crate dependency** — consumed from crates.io; the sibling `../systemprompt-core` checkout IS editable for cross-repo work via the `[patch.crates-io]` toggle (publish + bump + re-comment before landing).
   **Adopting a new core version:** bump the pins in **both** `Cargo.toml` and `tests/Cargo.toml` — a stale pin silently drops the patch and resolves the old crate from crates.io, so the build passes having proved nothing. Verify with `scripts/sync-core-version.sh <version> --check` and confirm the build log names the sibling path. Then run migrations with the **new** binary, `just prepare` to refresh the offline cache, and read core's changelog for tightened identifier validators and new `NOT NULL` columns — both are runtime failures that `cargo build` cannot catch (a validated `ContextId` panics in `new()`; a `NOT NULL` column breaks seed migrations that never named it). Full procedure: `docs/RELEASING.md` Step A0.
2. **Rust code -> `extensions/`** — All `.rs` files live here.
3. **Config only -> `services/`** — YAML/Markdown only. No Rust code.
4. **CSS files -> `storage/files/css/`** — NEVER put CSS in `extensions/*/assets/css/`.
5. **Brand name is `Astound Digital`** — Use "Astound Digital" for display, "astounddigital.com" for URLs.
6. **It's a library, not a framework** — Embedded code you own and extend. NEVER call it a "framework".
7. **Demo scripts must work on macOS and Linux** — BSD vs GNU differ on `grep -oP`, `head -n -1`, `sha256sum`, `sed -i`, and binary downloads (pick `hey_darwin_amd64` vs `hey_linux_amd64`). `demo/_common.sh` provides `install_hey()` for the last case; prefer `grep -oE` + `sed -n 's/.../\1/p'` over `grep -oP … \K …`.
8. **No Co-Authored-By in commits** — `coauthorAttribution: false` is set in `.claude/settings.json`. Never add `Co-Authored-By:` trailers to commit messages.

---

## Repository Naming Convention

Every function under `extensions/web/admin/src/repositories/` is named for what
it returns, so a call site reads the same as its signature:

| Returns | Prefix | Example |
|---------|--------|---------|
| `Vec<T>` — zero or more rows | `list_` | `list_top_users` |
| `Option<T>` — a row that may be absent | `find_` | `find_session_header` |
| `T` — exactly one value, or an error | `get_` | `get_request_stats` |
| a page plus its total, `(Vec<T>, i64)` | `list_` | `list_requests_paged` |

Mutations keep the verb that describes them: `insert_`, `update_`, `delete_`,
`set_`, `count_`.

`scripts/check-repository-naming.sh` enforces this: it rejects `fetch_`
outright, and checks every other prefix against the function's actual return
type, so the table above cannot quietly stop being true.

`fetch_` is banned because it is not a synonym for the three above —
it was doing all three jobs at once, which is how the convention drifted: a
reader could not tell from `fetch_summary` whether an absent row was `None` or
an error, and had to open the file to find out.

---

## CSS Files (IMPORTANT)

**All CSS files go in `storage/files/css/`** and must be registered in `extensions/web/src/extension.rs`.

```
storage/files/css/          <- CSS SOURCE (put files here)
extensions/web/src/extension.rs  <- REGISTER here in required_assets()
web/dist/css/               <- OUTPUT (generated, never edit)
```

**To add CSS:**
1. Create file in `storage/files/css/`
2. Register in `extension.rs` `required_assets()`
3. `just publish` to compile templates, bundle CSS/JS, and copy all assets to `web/dist/`

---

## Publishing Assets

After changing templates, CSS, JS, or static files, run:

```bash
just publish
```

This runs (in order): `bundle_admin_css` -> `copy_extension_assets` -> `site_content_prerender`. Order matters — bundles must be built before `copy_extension_assets` copies them to `web/dist/`. Admin pages are SSR'd at runtime from `.hbs` templates in `storage/files/admin/templates/`, not precompiled.

**Exception: the public-site partials are compiled into the binary.** `services/web/templates/partials/{head-assets,header,footer,scripts}.html` are `include_str!`-embedded by `extensions/web/site/src/partials.rs`. Editing them requires a rebuild (`just build`) and a server restart before `just publish` — running publish alone keeps serving the markup baked into the old binary.

---

## Plugins

Each plugin is a directory holding one `config.yaml` — `services/plugins/<id>/config.yaml` — auto-discovered, so it is **not** listed in the root aggregator's `includes:`. The file's root key is `plugin:` (singular; one plugin per file). It aggregates agents, skills, mcp servers, and content sources by reference:

```yaml
plugin:
  id: astound-commons
  name: "Astound Commons"
  version: "1.0.0"
  enabled: true
  skills:
    source: explicit
    include:
      - who_am_i
  agents:
    source: explicit
    include: []
  mcp_servers:
    source: explicit
    include: []
  hooks:
    governance: true
```

`source:` selects where members come from — keep it `explicit` and list ids under `include:`. Leaving it to default to `Instance` makes the plugin claim every skill and agent on the instance, which is how the marketplace once showed every plugin with all 230 skills.

Every id listed must resolve to a real top-level resource in `services/`. `ServicesConfig::validate()` enforces this at load time.

Skills follow the same nested shape: `services/skills/<id>/config.yaml`, with the instruction body beside it. So do marketplaces (`services/marketplaces/<id>/config.yaml`) and artifacts (`services/artifacts/<id>/config.yaml`).

**Skill ids are `snake_case` in `services/` and dashed everywhere else.** Core renders a skill for every host as `<id>` with `_` → `-` (`/astound-commons:who-am-i` for `services/skills/who_am_i/`), the hooks report the dashed name, and every analytics key is the dashed `plugin:skill`. Code that starts from a services id and needs the host or analytics form goes through `extensions/web/admin/src/util/skill_ref.rs`; never compare the two spellings directly.

**Every entitlement is declared in `services/access-control/rules.yaml`** — one block per
entity (`marketplace/<id>`, `plugin/<id>`, `skill/<id>`, `mcp_server/<id>`, `gateway_route/*`),
each with a required `why`. Marketplace configs carry **no** `access:` block; the services gate
refuses one. The database is what is enforced and what the console edits: boot seeds an empty
table from the file and otherwise only reports drift; `/admin/sync` offers
insert-only, overwrite-from-code, or export-to-code. Console edits on a shared band require a
reason. Model and procedure: `/documentation/access-control`.

**Sources, planes, sync.** `/admin/sync` ("Code ↔ Instance") is the one sync system. A
*source* is where declarations come from, each with a content hash: `base` is this tree and
`bundle:<name>` is each remote kit the profile pins (`services.sources[]`, by digest). A *plane*
is what those declarations project into the database — `access_control`, `groups` and
`gateway_policies` (`repositories/sync/registry.rs`; `gateway_policies` is also edited live at
`/admin/gateway/policies`, since core reads `ai_gateway_policies` per request). Every apply
records the declared hash, actor and mode in `sync_state`. Kits own content, this repo owns
access: a kit ships no `access:` block; `rules.yaml` declares `marketplace/<id>` with
`owner: bundle:<name>`, reported as *awaiting its bundle* until the source is active. Kit
template and exporter: `deploy/kit/`, `just kit-export`, `just services-pin`. Doc:
`/documentation/services-sync`.

**`hooks: {governance: true}` belongs to exactly one plugin.** Claude Code runs plugin hooks session-globally, so `astound-commons` owns them because it is the plugin every signed-in user receives through the default marketplace. A second owner is another `govern` call per tool; an owner only some users receive leaves every other session ungoverned and unrecorded (no `plugin_usage_events`, so no skill invocations on the dashboard).
