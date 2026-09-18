# Changelog

All notable changes to this repository are recorded here, newest first.

Conventions (strict — hold every entry to them):

- Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/): an `## Unreleased`
  section at the top, then one `## <version> — <YYYY-MM-DD>` section per release, each with
  only the categories it needs, in this order: `### Breaking`, `### Added`, `### Changed`,
  `### Fixed`, `### Removed`.
- Entries are written for the reader who did not make the change: full sentences, what changed
  and **why**, named files/commands/flags where the reader will need them. No bare "updated X".
- Every user-visible or operator-visible change lands in `Unreleased` **in the same commit** as
  the change itself; internal-only refactors are recorded when they alter an API another crate,
  config, or dashboard consumes.
- A release moves the `Unreleased` content under its version heading; `Unreleased` is never
  deleted, only emptied.
- Version numbers track the root workspace `version` in `Cargo.toml`; bridge-only releases are
  prefixed `bridge-` (e.g. `bridge-0.17.1`).

## Unreleased

### Added

- Tools and artifacts are distinct. `artifact_kind` (schema
  `46_tool_artifacts.sql`, migration 087) is the one rule: a tool call is an
  *artifact* only when a person can view or retrieve what it produced — a
  file edited, written or read (`file`), an MCP Apps UI resource (`ui`), a
  typed card such as a table, chart or report (`card`), or a retained
  structured body with a preview (`body`); a Bash, Grep, search or untyped
  result is a plain tool call. The `tool_activity` view applies it to every
  ledger row and `conversation_facts` counts by it (`artifact_count`, plus
  new `artifact_files` / `artifact_cards`), so the Conversations and Skills
  rows, the conversation record and both new pages agree. The sidebar entry
  **Tools & artifacts** opens `/admin/tools` (every tool call: kind, server,
  input summary, state, duration, governance decision, person, links to the
  session, conversation and artifact; KPIs, charts, breakdowns, ribbon
  filters) and a re-scoped `/admin/artifacts` (only rows with a kind,
  filterable by kind, with a preview or path affordance); the old
  "structured only / show all results" toggles are gone. The conversation
  page splits its tool table into **Tools** and **Artifacts**.
- AI reports on demand. `/admin/analysis/reports` lists and requests
  reports over a scope (global, marketplace, skill, or the current page's
  filters via **Report on this view** / **Generate** in the banner under the
  Conversations and Skills headers). A request stores a pure-SQL digest of
  the record (`analysis_reports.inputs`: totals, top skills, models, people,
  worst and best judged conversations, cost outliers, denial clusters,
  intent and client mix — never a transcript) and starts the one call to the
  judge model on a background task; the report page shows a loading widget,
  polls `…/status` and refreshes itself when the headline, the
  `ok | watch | degraded` assessment, themes with evidence links into
  Conversations, Skills and Tools, and recommendations are in. There is no
  queue and no job. The latest global report's headline sits in the banner.
- Judging from the list. Every unjudged conversation row carries a **Judge**
  button (`POST …/{context_id}/judge` with `back=`), the header offers
  **Judge N unjudged** over the current filters and the bulk bar **Judge
  selected** (`POST /admin/analysis/conversations/judge`, up to 200 per
  click, console-only). A judged row shows the verdict — intent, outcome,
  summary and rationale — in a hover/focus card instead of a truncated
  title attribute.
- Export from a selection or a bucket. Conversations, Skills, Tools and
  Artifacts rows have checkboxes (`components/table-select`, shift-click
  ranges) that enable a sticky bulk bar with **Export selected**; every
  breakdown row and every active filter chip carries a download glyph for
  exactly the rows it selects. The datasets accept `ids=` and ignore the
  window when it is present. The export dialog is rebuilt: one window
  control at a time (the three fieldsets used to render together), two
  panes, a grouped multi-column column checklist with a filter box and
  All / Default / None, a dataset description, a selection or bucket strip,
  and the last format and columns remembered per dataset.
- Presentation. KPI tiles take an `icon` and a `spark` in a reserved trend
  column (`components/kpi`, `05-kpi.css`) so a sparkline can no longer sit
  over the value; a shared inline icon set (`components/icon`) heads tiles,
  columns, ribbon pills and help entries; the nine stacked selects became a
  filter ribbon (`components/filter-ribbon`, `analysis::ribbon`) whose pills
  list only the values the filtered set contains, with counts and chips;
  charts are padded to the whole window (`generate_series` buckets, zeros
  included) and draw a dot per bucket when there are twelve or fewer; the
  row sparkline covers the last 24 turns and a one-turn conversation draws
  a flat line. Every Analysis page, Tools and Artifacts explain their
  columns in a `?` help modal (`components/help-dialog`, `sp-help.js`)
  instead of a paragraph at the page foot.

- Analysis leads with the deterministic record. A `conversation_facts` rollup
  (schema `45_conversation_facts.sql`, migration 085) holds one row per gateway
  conversation — client and attestation, models, turns, tokens by kind, cost,
  latency percentiles, tool calls by ledger state, artifacts, safety findings,
  governance decisions, hook prompts and skill invocations — kept current by
  the new `conversation_rollup` job. `/admin/analysis/conversations` reads it
  with KPI sparklines, three charts on one axis and an eight-way breakdown;
  the new `/admin/analysis/conversations/{context_id}` shows every plane of
  one conversation (turn ledger, tool ledger, governance, safety, skills with
  the marketplace version served). `/admin/analysis/skills` adds marketplace
  adoption (entitled → installed → active, by host) and a per-skill record;
  the skill page splits invocations by model, client, group, project, person,
  version and outcome. `sp-chart.js` adds hover crosshairs and legend toggles
  to the server-rendered charts.

### Changed

- The bridge builds against core `next` `d7964e842` (`bridge/CORE_REF`):
  every GUI request answered (the `mount` regression since 0.53.0), the
  wizard stays closed on an upgraded machine, the Windows Claude Desktop
  profile carries the whole policy, the Cowork tick no longer re-syncs every
  30 s on a warning it cannot read, installation evidence is captured for all
  hosts together within a budget, and the MCP probe sends `tools/list` with
  `params: {}` (Google's MCP refused `null`).

- `just build` (and every coordinated recipe) always compiles the current tree.
  `scripts/build-coordinator.sh` lost its "already green for this source
  tree" cache: on 2026-09-18 a bare `cargo build` from another checkout
  overwrote `target/debug/systemprompt` after a coordinated build had passed,
  and every later `just build` returned without compiling while the binary
  on disk was stale — `just publish` then shipped an old asset manifest and
  `just start` served old code against new templates. The one rule now is
  one build in flight at a time, always of the latest source: a concurrent
  caller is told a build is in progress and either attaches (identical
  source) or waits and then runs. Cargo's own incremental cache is what makes
  an unchanged tree cheap. `BUILD_FORCE` is accepted and ignored.

- One judge. `conversation_classifier` is now `conversation_judge`: a single
  structured call per conversation records a title, summary, intent, outcome
  and one 0–100 `completion` score (did the assistant deliver what was
  originally asked) on `conversation_analyses`. **Judge now** on a conversation
  queues a manual verdict that runs even with `evaluation.automatic` off. The
  feedback assessment fact reads that score.

### Fixed

- The Versions page's **Compare** link is `?days=…&tab=compare…` again; it
  was emitted as `?days=30?tab=compare` and landed on the History tab.
- The export dialog showed all three window fieldsets at once: the
  component-layer `.sp-export__field { display: flex }` outranked the base
  `[hidden]` rule. The hidden state is now declared in the same layer.

- A connector refresh the provider refuses is a reconnect, not an outage.
  Atlassian retires a rotating refresh token with `403 unauthorized_client`
  ("refresh_token is invalid"); `connector_oauth::tokens` treated only
  `invalid_grant`/401 as a rejected grant and everything else as
  `provider_unavailable`, so the account sat in `temporarily_unavailable`,
  every MCP call and every bridge sync retried the same refresh (one revision
  per attempt), the gateway answered `/api/v1/mcp/atlassian` with
  `503 … token accessor returned status 502` and nothing was logged. Every
  4xx token response, and any `error` body, now retires the grant
  (`reconnect_required`, the OAuth error code in the message); only a 5xx, a
  429 or a transport failure is an outage. The token request and every
  recorded grant failure log under `target = "connector_oauth"` (provider,
  status, OAuth error code and description — never a token or body), and an
  account in `temporarily_unavailable` is held for 30 s after its last
  attempt before the provider is called again; **Test connection** always
  goes through. `connector_credentials::lock` returns `LockedGrant` with the
  row's `updated_at`.

### Removed

- The four-criteria session judge (`evaluation_queue`, `evaluation_runs`),
  its attribution views, the hourly AI evaluation report, the scorecard, the
  `/admin/analysis/evaluation/*` pages, `services/evaluation/`, the session
  page's Evaluation panel and `POST /admin/sessions/{id}/evaluate`, and the
  snapshot-based Skills portfolio (`/admin/analysis/skills.json`,
  `portfolio-live.js`). Migration 085 drops the tables.

- The India marketplace (`astound-india-dev`, 34 skills) is a kit:
  `Astound-Digital/sfscore-claude-india-kit`, seeded with `just kit-export` and
  published to GHCR by its CI; the ids left `services/` and `rules.yaml` owns the
  entitlement as `owner: bundle:india`. `sfnext-claude-dev-kit` was brought to
  the same contract (no access block, instance MCP ids, channel-publishing CI,
  rotated signing key). Both profiles follow the kits' `stable` channels;
  `docs/kits-on-another-instance.md` is the runbook for a separate deployment
  and `requirements.txt` lists the tokens and secrets the Astound team provides.

## 0.55.1 — 2026-09-18

Built against the published core 0.55.1 (`bridge/CORE_REF` = `v0.55.1`). 0.55.0 was released but
never served: its image crash-looped on production because core 0.55.0 had rewritten the text
of migration 008 (`artifact_narrow_waist`) after the 2026-09-17 11:16 UTC preview deploy from
`next` had already applied the original, and the migration runner refused the checksum
mismatch (fail-closed, as designed); the machine was rolled back to the 0.54.0 image
(≈ 23:48–00:03 UTC). Core 0.55.1 restores 008 to its applied text and carries the corrections
as migration 012 (`artifact_source_repair`); `InvocationSkillIdentity` gains
`marketplace_hash`, which this repository's projection already emitted.

### Fixed

- Production runs 0.55.1 (the 0.55.0 image was never healthy) — see the release note above.

## 0.55.0 — 2026-09-17

Built against the published core 0.55.0 (`bridge/CORE_REF` = `v0.55.0`): the
`[patch.crates-io]` blocks are commented out in both workspaces and the pins name the
published crates. Core 0.55.0 ingests every tool result as one artifact per MCP call, claims
the gateway intent for an in-process execution atomically, purges users with their rows, and
runs the services refresh in-process (`ServicesRefresh::run(actor)`).

### Breaking

- **Console access is marketplace participation.** Only `admin` and `platform_admin` see the
  instance unscoped; `project_manager` is an entitlement role with no console tier (it lands on
  `/admin/profile` like `developer` and `knowledge_worker`). Everyone else reaches the console
  through a group or project allow rule on a marketplace in `rules.yaml` and sees Analysis
  (Conversations, Versions) and Platform (Code sync) narrowed to what that participation names;
  the header id lookup lands on detail pages, which 404 an owner outside the caller's view.
  The admin HTTP contract baseline is re-recorded accordingly.
- The candidate editor and revision comparison (`/admin/analysis/revisions/{id}/edit`,
  `…/compare/{candidate}`) are gone with core's experiment engine; a revision page shows the
  verified bytes and provenance and offers the bundle and file downloads.

### Added

- **Platform**: `/admin/configuration` lists every kind of configuration the instance loads with
  its source, hash, mode and sync state; `/admin/sync` (Code sync) keeps the sources and adds
  the zip export, the staged import preview (`POST /api/public/admin/sync/import`, a preview
  page, an apply per plane, a discard) and a history read back from `user_activity`; the
  sync-plane card renders as `?tab=sync` on the access-control, groups, gateway-policies,
  gateway-routes and governance pages. Every plane — access control, groups, gateway policies,
  gateway routes, governance chain — has drift, apply (insert-only / overwrite / scoped) and
  export; a participant applies the access-control plane narrowed to their own marketplaces.
- Gateway routes are database-backed (`gateway_routes`, migration 080): the `/admin/gateway`
  editor writes the table and regenerates `services/ai/gateway.yaml`; `/admin/gateway/policies`
  edits the live policy table (quota windows, safety scanners) with a quotas page and the
  calendar-month window job; `/admin/system/observability` shows and tests the OTLP export.
- Artifacts are a first-class entity: every tool result from Claude Code, Cowork, OpenCode or
  the gateway is one linked artifact, listed at `/admin/artifacts`, counted on the session page
  beside tool calls, and joined to the MCP execution and the governance decision through
  `tool_use_id`, `trace_id` and `mcp_execution_id` columns on `plugin_usage_events`
  (migration 075). An oversized MCP passthrough result is stored whole and the model receives
  a pointer to it instead of a truncated table.
- Analysis: `/admin/analysis/conversations` (the `conversation_classifier` job: purpose, outcome,
  summary, tags and skills per quiet conversation), the session evaluation engine
  (LLM-as-judge per criterion, async, host-agnostic; `/admin/analysis/evaluation` with
  **Evaluate now** / **Re-evaluate**, per-criterion **Re-run**, queued closed sessions, and
  `evaluation.automatic` in the profile), the evaluation report, marketplace versions keyed
  on their content hash (`marketplace_versions`, migration 071) with a compare and
  distribution tab, and per-skill pages under `/admin/analysis/skills/{skill}`.
- Every AI request is stamped with its group/project scope at request time
  (`ai_request_scopes`, migration 078) so spend and usage read the scope that applied rather
  than current membership; `organization` and `role` quota subjects are registered and
  `quota_fault_mode` is closed.
- Connectors: `/admin/connectors` and `/admin/connect` replace the settings panels;
  `google_drive` (Agent Search over Drive, Gmail, Calendar and People) replaces `google_rag`.
- `deploy/salesforce/`: the Salesforce side of the connector as source. One External Client
  App (`Systemprompt`) is created in the source org (Astound US), retrieved into an SFDX
  project and built into a 2GP managed package that every other org installs; subscriber orgs
  resolve to the source org's consumer key, so the gateway holds one
  `salesforce_mcp_client_id` / `salesforce_mcp_client_secret` pair for all of them. The README
  is the runbook, proven first on the Developer Edition org (`salesforce-dev`).
- Session evaluation can be started from the console: **Evaluate now** / **Re-evaluate** on a
  session page (`POST /admin/sessions/{id}/evaluate`, with a per-criterion **Re-run** on each
  score card), and **Queue closed sessions** / **Run pending now** on `/admin/analysis/evaluation`.
  A click runs the judge inline and answers when the runs are recorded; the rows carry
  `trigger = manual` and `requested_by = <admin>` (migration 077), while the judge call itself
  is still made as the job actor so it never counts against the person who clicked.
- `evaluation.automatic` in the profile decides whether closed sessions are queued without a
  click (production: `true`; default `false`). With it off the `session_evaluation` job only
  drains what a click queued, so a run that died mid-way still completes.

- `deploy/salesforce/`: the Salesforce side of the connector as source. One External Client
  App (`Systemprompt`) is created in the source org (Astound US), retrieved into an SFDX
  project and built into a 2GP managed package that every other org installs; subscriber orgs
  resolve to the source org's consumer key, so the gateway holds one
  `salesforce_mcp_client_id` / `salesforce_mcp_client_secret` pair for all of them. The README
  is the runbook, proven first on the Developer Edition org (`salesforce-dev`).
- Session evaluation can be started from the console: **Evaluate now** / **Re-evaluate** on a
  session page (`POST /admin/sessions/{id}/evaluate`, with a per-criterion **Re-run** on each
  score card), and **Queue closed sessions** / **Run pending now** on `/admin/analysis/evaluation`.
  A click runs the judge inline and answers when the runs are recorded; the rows carry
  `trigger = manual` and `requested_by = <admin>` (migration 077), while the judge call itself
  is still made as the job actor so it never counts against the person who clicked.
- `evaluation.automatic` in the profile decides whether closed sessions are queued without a
  click (production: `true`; default `false`). With it off the `session_evaluation` job only
  drains what a click queued, so a run that died mid-way still completes.

### Changed

- Salesforce orgs share one OAuth callback, `/api/public/connectors/salesforce/callback`. The
  packaged app's callback list is fixed in the source org, so per-org callbacks would need a
  source-org edit for every new org; instead the consent `state` carries the server id as a
  prefix (`salesforce-uk.<random>`) and the shared callback recovers the org from it before
  consuming the same user-bound state row as before. `client_id_secret` / `client_secret` in
  `services/config/salesforce-orgs.yaml` are now optional and default to the shared pair. The stored
  `resource_name` of a Salesforce grant is the instance URL Salesforce reports at consent
  (`urls.custom_domain`), the key a one-to-many client connector will use later.
- The single `salesforce` MCP server is replaced by one server per org: `salesforce-us`,
  `salesforce-uk` and `salesforce-dev` (`services/mcp/`, `services/config/salesforce-orgs.yaml`,
  `services/access-control/rules.yaml`, the plugins, marketplaces and kit manifests that
  named it). The US and UK My Domains are `REPLACE-…` placeholders until the orgs are wired up.
  Every external server's nominal `port` is now distinct: the loader checks uniqueness across
  external servers too, so three orgs on `5042` refused to boot.

### Fixed

- Every user-keyed extension table references `users(id) ON DELETE CASCADE` (migration 082);
  migration 083 deletes the rows earlier deletes had orphaned, validates each constraint so an
  upgraded database matches a fresh install, and drops `idx_mcp_artifacts_mcp_execution_id`,
  which core 0.55.0 removed from its schema without a migration.
- A Claude Code session is one trace: hook rows fold into the gateway session; MCP execution
  ids and session-less requests resolve on the request and trace pages.
- An uploaded sync archive that names an absolute path is refused on the raw name (`zip`
  relativises the leading slash); a zip the reader rejects is a typed 422
  (`AdminError::Archive`). A governance chain hashes its params without the id and switches,
  so an export that spells out an inherited mode round-trips to the same declared hash.
- A gateway-routes extra_headers column that does not decode is a decode error, not a route
  with no headers; the policies export header names the `governance_bootstrap` loader.
- The Salesforce org registry moved from `services/salesforce/orgs.yaml` to
  `services/config/salesforce-orgs.yaml`. Core's services bundle copies only its allow-listed
  directories into the composed tree the server actually reads, and `salesforce/` was not one of
  them, so the registry had loaded empty (`salesforce_orgs_loaded orgs=0`) on every boot since
  bundles arrived — every org showed as not configured.

- Every `// lint-ok:` gate exemption and every `#[expect(clippy::…)]` in production code is
  gone, by fixing what each one excused. `AdminError` gains `UpstreamFailure`, `Invalid` and
  `Refused` variants (and `upstream` / `invalid` constructors) that carry the typed cause
  instead of a `format!`-flattened message; `MarketplaceError::Crypto` carries its source and a
  `SecretEnvelope` variant names the variable whose stored nonce or plaintext is malformed;
  `ExtensionConfigErrors` entries can carry a source, so a blog config that fails to read or
  parse is reported with the underlying error rather than a stringified copy; the MCP server's
  CLI runner has a typed `CliError` projected onto rmcp's wire error once.
- SSR handlers take one `Page` extractor for the template engine, user and marketplace
  contexts instead of three `Extension`s; the hook handlers take `TrackDeps` / `GovernDeps`
  the same way. SSO start and callback, dev-login redemption and both governance hooks return
  `Result<_, E>` where `E: IntoResponse` is the flow's own failure channel (`LoginRedirect`,
  `DevLoginRejected`, `Unavailable`, `HookVerdict`) instead of a bare `Response`.
- The active analytics tab is serialised from the tab enum (`ActiveTab`) rather than stored as
  one bool per tab; the analysis window bounds are a `WindowBound` newtype that owns its date
  round-tripping; `SourceCard` is assembled by constructors rather than an eight-argument
  function; the numeric casts scattered across display code all go through `numeric`.
- `check-http-errors.sh` no longer honours `lint-ok:` markers, and its untyped-response rule
  applies to `async fn` route handlers only — a synchronous `-> Response` is a renderer with
  nothing to type.

### Removed

- The never-called repository functions kept "for the internal fork" (`compute_effective_permissions`,
  `list_user_marketplace_overrides`, `list_user_management_aggregates`,
  `list_user_runtime_aggregates`), the no-op `authz::{group,project,salesforce}::invalidate`
  and `AdfsConfig::mapped_groups`. `dashboard::list_events` is `get_events_page`.

## 0.54.0 — 2026-09-16

Built against the published core 0.54.0 (`bridge/CORE_REF` = `v0.54.0`). Core
0.54.0 is a breaking release: the evaluation domain (`systemprompt-evaluation`,
`Profile::evaluator`, `admin evals`, every `Eval*Id`, the `/api/v1/campaigns*`
routes and the `evaluation_supervisor` job) is deleted outright, the OAuth
dynamic-client-registration path is hardened, `RequestOrigin` carries the
client attestation tier, and MCP connectors gain fields. This release finishes
that removal on the astound side and carries everything banked on `next` since
0.53.0; the `[patch.crates-io]` blocks are commented out and the pins name the
published crates.

### Breaking

- **The evaluation engine is gone.** Core 0.54.0 deletes its evaluation domain, so this
  repository drops everything that consumed it: `services/evaluations/` (every campaign,
  experiment and fixture definition), `deploy/evaluator/` and `scripts/evaluator/` (the
  pinned native-client runner images and probes), `extensions/evaluator/`, the
  `evaluation-fixture` MCP entry in `services/config/`, the `evaluation_supervisor` entry in
  `services/scheduler/config.yaml`, and the admin pages `/admin/analysis/campaigns` and
  `/admin/analysis/evaluations` with their launch, holdout, approval and report actions
  (`handlers/ssr/analysis/{campaigns,campaign_*,experiments*}.rs`, `routes/evaluation_state.rs`).
  Analysis observes, compares and measures skill versions; it no longer turns a production
  invocation into an evaluation case. Migration `069_drop_reviewed_production_failures`
  drops the only astound table that referenced core's `eval_resource_revisions` and
  redefines `expire_feedback_raw` without it; `070_drop_eval_session_binding_hooks`
  drops the orphaned `capture_feedback_traffic` trigger function and the always-true anti-joins that excluded evaluation traffic from the feedback facts. Core's marketplace migration
  011 drops the `eval_*` tables themselves, so an installed profile must not carry an
  `evaluator:` block (core now rejects it as an unknown field) — the production profile
  never did. Publication review (`/admin/analysis/publications`) now rests on human
  evidence alone; the attestation fixture is deleted with the engine.

### Added

- Client attribution on every conversation surface is evidence-tiered. Core `next` records
  `ai_requests.client_attestation` (`host-token`, `bridge-secret`, `declared`, `native-marker`,
  `user-agent`, `none`) beside `client_kind`, plus an `ai_request_client_evidence` row with what
  the wire carried; `client_kind` gains `pi`. Migration `067_client_attestation` carries the
  column through `conversation_requests`, `conversation_metrics_for` and `conversation_rollups`
  (each dropped and recreated, as `063` did). The client chip on `/admin/sessions`,
  `/admin/contexts`, `/admin/history`, `/admin/conversations` and the user detail Usage tab
  now encodes the tier as its tone — `accent` "Attested by bridge host token", `info` "Declared
  by client", `muted` for a native marker, a bare User-Agent or no evidence — with the tier
  name as the title, so a Claude Code chip the bridge vouched for and one only a User-Agent
  claimed no longer look the same. The "By person" client list carries the strongest tier seen
  per client. `docs/gateway-routes.md` points at the hosted contract (`/documentation/services/gateway`,
  "Client attribution"), which is where a direct client learns to declare itself with
  `x-systemprompt-client`.

- **`/admin/sync` — "Code ↔ Instance"**, one sync system for every declared service,
  local and external (Platform → Sync; `/admin/access-control/sync` redirects here).
  A **source** is where declarations come from, each with a content hash: `base` is
  this repository's `services/` tree (hashed with core's own bundle algorithm, so the
  number equals the published base bundle's `content_hash`) and `bundle:<name>` is each
  remote kit the profile pins — pinned reference vs active digest, version, content hash,
  fetch time, what it owns, provenance (`fetched` / `last_good` / `bundled_fallback` with
  the error), and badges `in step` / `pinned ≠ active` / `following a tag` / `restart
  pending` / `kit declares access`. A **plane** is what a source's declarations project
  into the database — `access_control` today; `groups` and `gateway_policies` are shaped
  for (`repositories/sync/registry.rs`) and not built — with a declared hash, the last
  apply (hash, when, by whom, mode) from the new `sync_state` table (migration 065), drift
  totals, the three directions (Insert only · Overwrite from code · Export) and the drift
  table. **Refresh sources** proxies core's `POST /api/v1/admin/services/refresh?restart=true`
  with the caller's own session, so no token is minted. API:
  `/api/public/admin/sync/{status,planes/{id}/drift,planes/{id}/apply,planes/{id}/export,sources/refresh}`.
  Doc: `/documentation/services-sync`.
- **Kits own content; this repository owns access.** `rules.yaml` entities accept
  `owner: bundle:<name>` for a marketplace, plugin or skill that arrives in a remote
  bundle: the id may be absent until that bundle is active (reported on the sync page as
  *awaiting its bundle*, not a boot failure), the services gate accepts it without a local
  definition, and the export carries it back. `marketplace/astound-europe-dev` is declared
  this way. A kit that carries its own `access:` block is flagged on its Sources row and
  the rows it wrote (`source = bundle:<name>`) show as *bundle* orphans an Overwrite
  deletes.
- **Kit template and exporter.** `deploy/kit/` holds the runbook
  (`README-INTEGRATION.md`), the `publish-bundle.yml` workflow (import → validate against
  the base bundle → sign → publish to GHCR → print the digest), the two sidecar templates
  (no `access:`), and `known-kits.json`, the one registry `just services-pin` and the e2e
  seed both read. `just kit-export <marketplace> <dir>` (`extensions/cli/kit-export`, a
  developer tool like dev-login) writes a marketplace from `services/` as a kit tree using
  core's own plugin bundler and proves it by re-importing it strictly and diffing the
  result against `services/`. `release.yml` now publishes the base services bundle
  (`ghcr.io/astound-digital/systemprompt-astound-base:<version>`, also a release asset)
  that a kit's CI validates under.
- The catalog pages read the **composed** services root (`ServicesRootBootstrap`), so a
  bundled marketplace is listed in the console; writers (the gateway editor) keep editing
  the baked tree.

- **Every skill invocation is attributed, and to the tree that shipped it.** The
  feedback projection stamps each invocation fact with a skill identity — the hook's
  `plugin_id` and `skill`, the marketplace, and the services source (`base` or
  `bundle:<name>`) with that source's content hash (core `NormalizedInvocationFact.skill`).
  The sources and what each owns are recorded at boot in the new `service_sources` /
  `service_owned_ids` tables (migration 066) from the same composition `/admin/sync`
  shows. The Analysis → Skills page now keys its rows and its **Attributed** headline on
  that identity, so a configured skill, a kit's skill and a managed resource all count
  and carry their source hash; **Revision-verified** (proven against a published managed
  resource on the device — the figure Campaigns and Impact cohort on) is shown beside it
  and is unchanged. Rows drill down by `?plugin=&skill=`; the `?resource=` link other
  pages use still works. Facts normalised before this carry no identity until the
  feedback backfill re-normalises them.

### Changed

- The admin sidebar is six collapsible groups — AI activity (opening on Overview), People &
  access, Analysis, Governance, Platform, Account — with exactly one group open at a time
  (the one holding the current page, opened by CSS before `services/nav-groups.js` runs)
  and a "Find a page" search box bound to `/`. The Analysis sub-navigation loses the
  Campaigns and Experiments tabs with the engine. The trace detail page drops its
  waterfall panel and the session, context, device and governance-audit detail pages are
  reworked on the same shell.
- `just deploy-release <version>` is restored (it was dropped by mistake in the `/admin/sync`
  commit while `scripts/deploy-release.sh` survived), and `scripts/sync-release-version.sh`
  no longer expects a `systemprompt-evaluation` pin in `tests/Cargo.toml`.
- `just services-pin <kit> <digest> [profile]` takes the kit name and inserts the
  `services.sources[]` entry when the block is absent (it used to `sed` a commented-out
  block and fail); the image and public key come from `deploy/kit/known-kits.json`.
- The activity entity `access_control` is now `sync`, with the plane in the metadata, so
  every plane's apply/export and the source refresh share one entity kind.

- The analytics **Skills** tab is back at `/admin/analytics?tab=skills`, between
  Models and Tools, reshaped around what a skill actually is. A skill is
  instructions pasted into a conversation and has no cost of its own, so the old
  "measured cost" (the price of the one turn in which the model chose the `Skill`
  tool), the "unattributed" bucket and the by-model table are gone. Each row now
  shows adoption — invocations, slash/tool split, users — and facts about the
  **conversations** that invoked the skill: how many, their gateway requests and
  their spend, labelled as conversation spend and stated to overlap between rows.
  The KPI strip counts every conversation and request once. Rows read the same
  facts as the Analysis section (`analysis_skill_version_events` joined to
  `ai_requests` on owner + `client_session_id`, synthetic and evaluation traffic
  excluded), so the two screens agree; each row links to the Analysis
  conversations for its managed resource, shows what share of invocations resolved
  to a published version through a verified install, and shows its evaluation
  state through core's campaign repository, the one the Campaigns page lists from
  (`not evaluated`, `N active`, or count and latest
  status) linking to Campaigns. `/admin/entities/skills` and `/admin/demo/skills`
  redirect to the tab again.

### Fixed

- The admin front-end gates the release pipeline runs (`web-unit-tests`) pass again
  on the banked admin redesign: dead waterfall, cost-spark and analysis-filter rules
  are deleted, the access-control entity columns and the trace legend's `--ok` key
  have rules, the data-rendered `is-error`/`is-rejected`/`is-warn` trace states are
  exempted, `20-page-access-control.css` and `20-page-groups.css` are split by
  component under the 200-line ceiling, the sync page delegates clicks through
  `services/events.js`, the project-detail empty-state count follows the withdrawn
  commits table, and `docs/profile.schema.json` is regenerated for a core profile
  without `evaluator`.
- The integration and contract tiers follow the banked redesign's deliberate contract
  changes: a lone tool-less request is a side call only in a harness-bound context
  (migration 063), an invocation no device authenticated has no conversation row,
  role rules carry a `justification`, the access plane's export and drift live at
  `/sync/planes/access_control/*`, the trace page's section is "Timeline",
  `/admin/{entities,demo}/skills` redirect with 308, contract principals are real
  UUIDs (a browser session is minted for the id), and the sources refresh proxy's
  by-design 502 without a loopback listener is registered in `KNOWN_5XX`.

- The Skills tab's skill link opened `/admin/skills/<kebab-name>` and 404'd, because
  the hook renders a skill id's underscores as hyphens while the catalog is keyed by
  the id. The catalog link now reverses that (it is the fallback for a skill with
  no verified managed resource).

### Removed

- The Rating column on the Skills tab. `skill_ratings` has had no writer since the
  control-center UI was removed in May 2026 and skill evaluations have not launched,
  so the column could only ever show a dash.
- The code-impact metrics — the analytics Overview's "Code impact" panel (AI lines
  added/removed, committed lines, commits, AI edit operations), the Commits table
  on the user detail Usage tab and on the project detail Usage tab — are withdrawn
  until there is a sound way to measure them. AI line counts were the line count of
  Edit/Write tool inputs seen by the `PostToolUse` hook (a whole-file Write counted
  every line as new), and commits were regex-parsed from `git commit` stdout in
  Claude Code Bash calls, so anything done in another terminal was invisible.
  Neither frame answered "what did the AI produce", and showing them side by side
  invited exactly that reading. The ingestion (`hooks_track/loc.rs`,
  `hooks_track/commits.rs`), the `user_commits` table, the `loc_added`/`loc_removed`
  columns on `plugin_usage_events`, `plugin_usage_daily` and
  `plugin_session_summaries`, and the commit/LOC columns on
  `admin_usage_daily_rollups` are dropped by migration `064_drop_code_impact_metrics`,
  which also redefines `drain_ingestion_outbox` and `expire_feedback_raw` without them.

## 0.53.0 — 2026-09-15

Built against the published core 0.53.0 (`bridge/CORE_REF` = `v0.53.0`). Core
0.53.0 carries three streams this release depends on: the feedback program
(native evaluator adapters for Claude Code, Codex, OpenCode and Hermes under one
container supervisor, device-scoped consumer credentials with installation
receipts and session bindings, marketplace inventory with complete Git bundle
verification, analytics rebuilt on durable outbox facts, projections and
snapshots with privacy-coordinated retention, and a bridge that enrols a device
and keeps a bounded feedback outbox); review-2 wave 1 (a typed, I/O-free shared
layer, a fail-closed extension registry, a bridge without the mTLS provider);
and review-2 wave 2 (`verify-full` TLS, typed foreign-key drift, RAII advisory
locks, an injected governance engine, owner-bound agent tasks, authz on proxied
MCP identities, refusal of unpriced models, and the headline security fixes —
OAuth open redirect, MCP anonymous fallback, gateway rate limiting, OTLP
authentication, JWT lifetimes, reusable approvals). Every commit on `next` since
0.52.0 is recorded below; the `[patch.crates-io]` blocks are commented out and
the pins name the published crates.

### Added

- An organisational skill portfolio and optimisation campaigns on the admin console.
  `/admin/analysis/resources` (and `.json`) lists configured skills, plugins and marketplaces with their
  request-deduplicated cost, tokens, quality and latency, adapted from the authenticated
  hook evidence into core's resource metrics; `/admin/analysis/campaigns` creates, attaches experiments to, launches
  and reports optimisation campaigns — a baseline revision, a candidate, a shared budget,
  paired comparisons and a fresh holdout before any publication — with the policy,
  budgets and comparisons owned by core's evaluation domain and only the catalogue, the
  hook-event adapter, the templates and forms living here. Managed resources and campaigns
  run under the configured system owner's organisational scope while requests keep the
  acting administrator; project managers keep their existing pages and no longer see
  organisational revision and evaluation evidence (those shared authoring routes now
  answer 403 for them — contract baseline re-recorded). `docs/skill-optimization.md`
  states the ownership boundaries, the bounded opt-in automatic development loop,
  and the remaining client-attribution, scalability and paid-pilot acceptance gaps.
- Holdout proposals are reviewed independently of execution admission: a campaign's
  completion page shows each proposal's spec digest and asks the reviewer to confirm an
  independent holdout (`POST .../holdouts/{proposal}`, typed `EvalHoldoutProposalId`)
  before it is admitted, and portfolio baselines can be prepared for a campaign from a
  measured resource.
- Snapshot-fed resource effectiveness. Every figure on the resources page is read from the
  retained feedback snapshots, grouped marketplace → plugin → skill with per-group rollups,
  resolved audience counts (users reached across every marketplace carrying the skill),
  health verdicts (availability faults first, then a failed-request share above ten per
  cent) and an explicit binding action for entries that conflict; campaigns pre-select a
  measured baseline. OpenCode skill use lands in the invocation ledger through the
  bridge-written hook plugin (migration `061_opencode_skill_events`), with the view arm,
  entity linking and clean-client assertions to match.
- The automatic attribution chain. Core issues the device credential at login, grants by
  manifest reach, publishes the latest configured revision every minute and stamps the host
  header where each host's hooks file is written; on this side the manual "prepare
  baselines" step is gone, both analysis pages offer "Sync inventory now"
  (`POST /admin/analysis/inventory/sync`: refresh → publish latest → back), the inventory
  table gains a Published column read from `managed_publications` heads (warn badge when
  the latest capture is not yet published), and the Skills drill-down reads retained data
  only — totals from snapshot contributions, per-resource rollups joined to
  `managed_resources`, per-resource conversations over the normalised facts; the live
  `analysis_skill_events ⨝ ai_requests` query is deleted. The admin MCP tool schema is
  Gemini-declarable (`days` defaults to 7 instead of being nullable, `report` is inlined),
  pinned by a test over every tool the server lists. `ingestion_session_owners` cascades on
  user delete (migration `062_session_owner_cascade`) so `admin users delete` works for
  anyone who ever reported a hook.
- The snapshot portfolio dashboard (`/admin/analysis/portfolio`) renders the retained
  per-day snapshots with coverage, bounded custom-range jobs (keyed by a digest of owner,
  resource and range as the `AnalyticsSnapshotJobId`) and a live resync driven by core's
  durable generation stream (`storage/files/js/services/portfolio-live.js`).
- Feedback ingestion, facts and snapshots. Authenticated hook evidence is ingested through
  durable corrections (`extensions/web/jobs/src/feedback_ingestion`, schemas
  `30_feedback_ingestion` / `31_feedback_projection` / `32_feedback_retention`, migration
  `059_feedback_ingestion`, which also retires `29_skill_version_impact`); the scheduler
  runs `feedback_ingestion`, `feedback_facts_processing` and `feedback_snapshot_processing`
  every five seconds and `managed_inventory_refresh` every minute
  (`services/scheduler/config.yaml`), so the organisational catalogue is refreshed at
  startup and on schedule and raw expiry is coordinated with all-owner snapshot compaction.
- Typed admin-analytics tools on the `systemprompt` MCP server. A production admin session
  tried to answer "which users do real work, and has their prompting improved" and failed on
  every surface — the dashboard is aggregate, the free-form CLI tool made the model discover
  flags by trial (fifteen `--help` calls, `--until` and `--export` invented, the existing
  `--user` never found), and one `infra logs audit` came back as a 400 KB single line. The
  server now carries `usage_by_user`, `request_log` (since/until/user, cursor-paged),
  `conversation_audit` (offset/limit/max_chars over one request's transcript) and `users`
  beside the passthrough, each with its flags in a flat, Gemini-declarable JSON schema and
  shelling out to the same CLI so auth, remote routing and the access audit are unchanged.
  The passthrough result is bounded at 64 KiB with a hint naming the narrower query, the
  summary no longer echoes the whole body, `--export` is stripped along with the format
  flags, and a CLI call times out after 120 s instead of hanging the client. The
  `systemprompt_cli` skill teaches the typed tools first, the paging loop and a per-command
  table of the flags that exist; `admin_ai_usage` gains the per-user workflow.
  `scripts/check-skill-cli-flags.sh` (a lint gate) ties every documented flag to the CLI's
  `--help` so the skills cannot drift from core again; `roles.yaml` names `super_admin` on the
  MCP server rule as it already did on the plugin.
- Native evaluator adapters. `deploy/evaluator/` packages pinned runners for Claude Code
  (image digest pinned), Codex (`codex-runner.cjs`), OpenCode (`opencode-runner.cjs`) and
  Hermes (`hermes-runner.cjs`, authenticated custom relay with an isolated runtime token),
  each behind a bounded native relay; `scripts/evaluator/*-native-probe.sh` and
  `native-subnet.cjs` run the isolation probes (native path permissions, parent isolation,
  isolated fixture subnets allocated without pruning) that core's capability registry
  requires before a native target is admitted, and the evaluator capability registry is
  surfaced on the admin console. Exhausted OpenCode process groups are terminated with
  verified cleanup.
- An upgrade gate. `tests/integration/schema-upgrade` restores
  `tests/fixtures/schema/release-baseline.sql` — a schema dump of a fresh install of the
  last release, `extension_migrations` rows included — into an empty database, runs the
  current installer over it, and diffs the resulting catalog against a fresh install of the
  current tree. Every other tier installs the declarative baseline on an empty database; no
  deployed instance is empty, and on 2026-09-14 that second path crash-looped production
  while every gate was green. `just schema-baseline` records the fixture from the local
  Postgres container's own `pg_dump`; `scripts/check-schema-baseline.sh` (lint gate and
  `just release`) refuses a fixture whose header does not name the workspace version, so a
  version bump forces a re-record before the release. Run against a 0.51.0 fixture the gate
  reported three core-side drifts (two missing `UNIQUE` keys, one missing owner-paired
  foreign key), now fixed by core migrations 011 and 012. The fixture now records the
  0.53.0 schema.
- The India BA skills are refreshed to upstream `5efee126` (Markdown-style Jira formatting).

### Changed

- Shared revision pages (revision detail, editing and comparison) require the
  manage permission through the same `require_manage` check the other authoring pages use,
  instead of a console-membership test that let a project manager past the door; the
  contract suite now records the managed-resource routes it exercises.
- `docs/evals.md` adds the verified source attestation to the publication sequence: an
  improvement is committed through the normal source workflow with server-side Git
  verification plus the campaign's evaluation attestation, and changed content requires
  re-evaluation — a score alone cannot authorise publication.
- `reviewed_production_failures` declares its owner-paired composite foreign keys inline
  again, under migration 057's constraint names. Core's installer now applies every
  declarative foreign key after migrations and indexes and skips one an existing constraint
  already provides, so fresh and upgraded databases converge on the same keys; the
  single-column workaround keys stay. The `[patch.crates-io]` blocks in `Cargo.toml` and
  `tests/Cargo.toml` are commented out for this release, every `systemprompt-*` pin is
  `0.53.0` and `bridge/CORE_REF` is `v0.53.0`; `next` re-activates the patch when it moves
  past the published core.
- Marketplace Git credentials are resolved through core's `GitSourceOrchestrator` instead
  of a local resolver, so source verification here runs with the same cleared environment,
  bounded output and deadlines as core's own.
- Sessions are composed from the users repository (`systemprompt::users::SessionRepository`
  / `UsersAiSessionProvider`), following core's move of session lifecycle out of analytics.
- Adapted to core 0.53.0's typed contracts: `EvalHoldoutProposalId` on the holdout
  confirmation route, `AnalyticsSnapshotJobId` on custom-range snapshot jobs,
  `AnalyticsWorkerId` on the facts and snapshot workers, a typed `CallId` on
  `DecisionAudit`, `SkillOptimizationOrchestrator::new(managed, evaluations)`, pooled
  handles, injected governance and fixture admission; the root `.sqlx/` cache carries core
  0.53.0's queries.

### Fixed

- The bridge builds again on macOS and Windows. Core 0.53.0 replaced `MarketplaceItem::new`
  with a builder; the Astound marketplace source (`bridge/src/registry.rs`) is the only
  bridge code this repository compiles on the GUI targets, and the Linux gates never see it.
  `just clippy` now also lints the bridge for `x86_64-pc-windows-gnu` on Linux, so the
  next such break fails the Gates run instead of the release job.
- Every managed MCP server in `services/mcp/*.yaml` declares `tool_policy: allow`, and the
  header comments plus `docs/CONFIGURED-CONNECTORS.md` no longer describe an absent key as
  "allow". Core 0.53.0 makes the key mandatory — a server without it is withheld from the
  signed bridge manifest and fails boot validation — so without this the deployment would
  refuse to start. `allow` keeps today's effective behaviour.
- Reporting privacy is fenced before initialisation and across owners: a merge or deletion
  waits for pending reporting evidence for every owner, not only the first one prepared, and
  raw feedback expiry expires the reporting evidence with it (migration
  `060_reporting_privacy_retention`).
- Suggestion reservations are atomic and the human approval lifecycle is exposed, so two
  concurrent reviewers cannot both reserve one development-only suggestion.
- The snapshot portfolio HTML survives wide aggregate values (totals no longer overflow
  the row template), snapshot-only portfolio metrics and bounded inventory pages are
  retained rather than dropped when an entry has no live counterpart, and fallible
  projections on the admin pages surface as errors while valid evidence is preserved.

## 0.52.0 — 2026-09-14

Built against the published core 0.52.0 (`bridge/CORE_REF` = `v0.52.0`).


### Added

- Evaluation is a supervised experiment lifecycle instead of the cron loop: shared budget
  accounts, pure preflight estimates, owner-scoped baseline capture, revision history and file
  comparison, exact skill candidate authoring, verified revision bundle export with cost
  boundaries, and reviewed publication of managed resources with reconciliation and withdrawal
  (`/admin/managed-resources/*`, `/admin/analysis/impact` and `/admin/analysis/publications`,
  migrations 056–057). The evaluation fixture MCP serves deterministic evidence to evaluator
  executions through a closed operation set and is denied to administrators in the
  access-control rules; the execution-scoped relay carries timeouts, a header allow-list and
  tracing, and `just evals-relay` runs it against this clone's gateway.
- Skill use is measured from durable ingestion evidence (`/admin/analysis`, migration 054):
  statusline sessions are validated for identity, owners are isolated, and native sessions
  interrupted mid-ingest are recovered (migration 055).
- `services/governance/config.yaml` declares the installation's secret signatures explicitly
  (GitHub classic, fine-grained, OAuth and App tokens; Anthropic, OpenAI and Google API keys;
  PEM private keys; structured `secret`/`client_secret` fields) because core 0.52.0 ships no
  built-in credential patterns — an installation without this section scans for entropy only.

### Changed

- The admin extension uses typed identifiers end to end: `GroupId` and `ProjectId` (declared
  through a local `web_define_id!`), and core's `PluginId`, `CallId`, `McpServerId`,
  `McpExecutionId`, `RouteId` and `MarketplaceId` replace the stringly ids in repositories,
  handlers, scope queries and page views; `ScopeQuery` names its container with a
  `ScopeTarget`. The unused `requirements-model` crate is removed.
- Built against the published core 0.52.0: both workspaces pin `0.52.0`, the
  `[patch.crates-io]` blocks are commented out and `bridge/CORE_REF` is `v0.52.0`, so the
  bridge and the server resolve to crates.io rather than the sibling checkout.
- `just prepare` is single-flight through the build coordinator and never writes the tracked
  `.sqlx/` in place: every `cargo sqlx prepare` pass lands in a scratch directory beside the
  cache, the passes are unioned there, and one rename replaces `.sqlx/` only after every pass
  succeeded. The `tests/.sqlx` symlink is parked for the tests pass and restored by the trap;
  a run killed with SIGKILL leaves the previous cache parked at `.sqlx.prepare-parked`, which
  the next run recovers first. `scripts/check-sqlx-cache.sh` now fails if `tests/.sqlx` is not
  a symlink to `../.sqlx` or a parked cache is left behind.

### Fixed

- The analysis facts query no longer excludes judge traffic through `eval_judge_calls`: core
  0.52.0 (evaluation migration 010) drops the standalone judge-run tables, so the predicate
  would have failed the first `/admin/analysis` load after the upgrade. Judge inference is
  already reserved through `eval_session_bindings`, which the query still excludes.
- The 0.52 preview failed to boot on production: the declarative schema for
  `reviewed_production_failures` carried inline owner-paired foreign keys
  (`(owner_id, invocation_id) → plugin_usage_events(user_id, id)` and the
  `eval_resource_revisions` twin) whose unique indexes only exist after the dependent phase
  and migration 057, and core installs every `CREATE TABLE` before either. On any database
  that lacked the table — production, and a fresh install — statement 50/50 failed with
  "no unique constraint matching given keys" and the server crash-looped; locally it was
  masked because migration 056 had created the table first. The declarative table now
  carries primary-key-backed single-column keys, the owner-paired constraints stay in
  migration 057 for existing databases, and the install was proven on a copy of the
  production schema and on an empty database, twice each.

### Removed

- `independently_metered_tool_charges` (migration 058 drops it) and the "independently
  metered tools" figures on `/admin/analysis/impact`. Nothing in core or this repo prices an
  MCP tool execution, so the column rendered an unmeasured zero as a measurement.
  Request-deduplicated spend is the only cost dimension until a metering source exists.
- The `evaluation_supervisor_registration_anchor` in `src/lib.rs` and its call in `main`.
  Every core job registers through `submit_job!` and none has needed an anchor; core's
  inventory test now pins all twelve built-in job names and looks the supervisor up through
  `inventory::iter`, so a dropped registration fails core CI rather than needing a guard here.

## 0.51.0 — 2026-09-11

Built against core 0.51.0. Everything since the 0.50.0 section is recorded here.

### Added

- Salesforce orgs are declared in a registry, one hosted MCP server per org. Every org is its
  own issuer with its own External Client App, so the connector no longer assumes a single org
  held in three global secrets: `services/config/salesforce-orgs.yaml` names each org's My Domain,
  environment, secret names, optional org id and entitled groups, and each becomes a distinct
  MCP server (`salesforce`, `salesforce-<slug>`). `Provider::Salesforce` carries the server id;
  entitlement is group-scoped per org and enforced on the profile listing and the start, callback
  and token endpoints; `organization_id` from userinfo is checked against a declared `org_id`, so
  a consent in the wrong org is rejected. `salesforce_user_identities` is keyed
  `(user_id, provider)` so JWT-bearer pre-authorization maps one Username per org (migration 053),
  and `POST /api/public/admin/connectors/{provider}/reprovision` resets every account on one org
  after a sandbox refresh (contract baseline: +1 row). A missing secret surfaces on the profile
  page as `not_provisioned` rather than failing boot. Salesforce DX is out of scope.
- The Vertex catalog is served by discovery. At boot the loader merges every Vertex model the
  core rate card prices into the `vertex` / `vertex-maas` providers, the bridge profile and
  `/v1/models` advertise the whole catalog to every host (Claude Desktop excepted), and the daily
  `vertex_discovery` job is scheduled. The three-way rule — published and priced = served;
  published, unpriced = reported; priced, unpublished = reported — is documented, and a unit test
  proves the full rate card validates against `gateway.yaml` and matches a route.
- Native Windows Claude Code can enrol through the gateway: PowerShell setup, routing,
  verification and disconnect instructions are published, and the bridge tests cover the path.
- The India BA marketplace skills are refreshed from their private source repository
  (`scripts/india/import-skills.py`, `skills-inventory.json`), and CI runs the importer checks.
- A lint gate fails when a tracked Dockerfile or workflow names a `systemprompt-mcp-*` binary no
  manifest declares, or a root-Dockerfile `COPY` source git does not track
  (`scripts/check-dockerfile-paths.sh`). Nothing in the per-push tier builds the image, so the
  0.50.0 release was the first thing to notice the two removed extensions.

### Changed

- The `vertex` endpoint is the global location — Gemini 3.x is global-only and us-central1
  returned 404 for every call — and the explicit catalog is the currently supported set:
  `vertex-gemini-3.8-flash` (default), 3.5-flash and 3.5-flash-lite on `vertex`;
  `qwen.qwen3-coder-480b`, `zai.glm-5`, `zai.glm-5.2` and `openai.gpt-oss-120b` on `vertex-maas`.
  The 2.5 family and the seven MaaS models deprecated 2026-07-21 are no longer declared; the core
  rate card keeps them priced with retirement dates, so discovery serves them until 30 days before
  and then withholds them without an edit here.
- Every microdollar cell in the admin cost report carries a `_usd` sibling and the dashboard KPI
  row is fed from metrics that were always empty; the admin skills name the Confluence search
  tools, count `data.issues.length`, treat an absent `duedate` as unset and re-read on every run.
- `fetch-remote-downloads.sh` sources the bridge artifacts from the GitHub release first and the
  live host only as a fallback, so a bad artifact can no longer survive every subsequent deploy.
- The evaluation experiment tests read the schema from `EvaluationExtension::schemas()` instead
  of `include_str!` against the patched core tree, so the suite builds against published crates.
- Two stray screenshots and two stray notes (`evals.md`, `issues.md`) are gone from the repo root.

### Fixed

- Both Vertex endpoints named a personal Google Cloud project literally, and Vertex echoes the
  project id in every IAM error the gateway relays, so a third-party developer saw it when the
  deployment's own service account was refused. The endpoints carry `{project}`, filled at
  dispatch from the service-account key; core refuses a literal id at boot.
- The release image no longer builds or copies the removed knowledge-bank and
  requirements-workflow artefacts, and the release smoke no longer asserts the removed binary —
  the two failures that stopped the first 0.50.0 release runs.

### Operator notes

- **Core 0.51.0 removals.** The A2A push-notification config methods, the
  `pushNotificationConfig` field on `message/send` / `message/stream`, and the
  `task_push_notification_configs` table (core migration 011) are gone. This instance ships no
  A2A agents and nothing here referenced them; the only trace is five stale entries in the root
  `.sqlx/` cache, dropped by the next `just prepare`.
- **Managed-MCP tool policy.** Core 0.51.0 defaults every bridge-managed server's tool policy
  to `allow` (`*: allow` in the manifest): the governance chain already judges every call, so the
  client's per-call prompt is off. No `services/mcp/*.yaml` here sets `tool_policy`; add
  `tool_policy: prompt|deny` under a server's `deployment:` to opt it back into confirmation.
- **Two copies of the core shared crates in `tests/Cargo.lock`.** `tests/unit/bridge` depends on
  `bridge/`, which depends on the unpublished `systemprompt-bridge` by path, so the tests lockfile
  resolves `systemprompt-{models,extension,identifiers,traits,provider-contracts}` once from the
  registry and once from `../systemprompt-core`. Both are 0.51.0 and nothing mixes them today; a
  test crate that takes a type from both will fail to compile, not misbehave.
- **`services/config/salesforce-orgs.yaml`** is new and required for the Salesforce connector; each
  declared org needs its own secrets provisioned before the org shows as anything other than
  `not_provisioned`.
- **`vertex_discovery`** runs daily and needs the Vertex service-account secret; a published,
  unpriced model is a log line, not a served model.

## 0.50.0 — 2026-09-10

Everything since 0.47.0 is recorded here: the `v0.48.0` and `v0.49.0` tags were cut from this
line without their own sections, so their entries appear under this heading.

### Added

- The Europe development marketplace can arrive as a fetched bundle. `astound-europe-dev`, its
  plugins `astound-europe-storefront` and `astound-europe-delivery`, and their ten skills are
  authored in [Astound-Digital/sfnext-claude-dev-kit](https://github.com/Astound-Digital/sfnext-claude-dev-kit)
  in Claude Code marketplace format, published by that repository's CI as an ed25519-signed
  services bundle on GHCR, and composed into the running services tree at boot when a profile
  carries a `services.sources` entry for it. Composition is by id: a marketplace, plugin or skill
  id present in both `services/` and a bundle fails the boot naming both sources. **This release
  ships with no `services.sources` block in either profile**, so nothing is fetched at boot;
  `just services-pin <digest>` restores the block once a bundle is published. See
  `docs/sfnext-marketplace.md`.
- The admin console states the running release at the foot of the sidebar, ruled off below
  Help & docs, replacing the Discord link. The number is the crate version the workspace inherits
  from the release pin, so it cannot drift from the release actually serving the page.
- Evaluation: owner-scoped APIs store immutable case/rubric/dataset inputs and queue frozen
  client/model comparisons with transactional budget accounts; admin APIs enroll and revoke
  evaluator workers and upload immutable workspaces with same-origin checks and non-cacheable
  credential responses; a pinned native-client evaluator image, `just evals-up`, `evals-probe` and
  `evals-down` give an isolated local evaluation database and offline probes. Worker assignment,
  event transport, expiring execution capabilities, rotation, environment isolation, revocation and
  provider-bound admission are covered by PostgreSQL and HTTP tests. Client workers, authoritative
  request accounting and the improvement dashboard are not yet connected.
- Dashboard query regression tests and disposable-database SQL/HTTP benchmark tooling document
  pagination, ownership, time-window behaviour and scaling limits.
- Public documentation separates supported workflows from unfinished evaluation work; internal
  diagnostics and planning are archived locally, and test/import data lives with its consumers.

### Changed

- Builds against the published core 0.50.0 from crates.io. Both `[patch.crates-io]` blocks are
  commented out, `bridge/CORE_REF` is `v0.50.0`, and all three lockfiles resolve the core crates
  from the registry. `bridge/Cargo.toml`'s `systemprompt-bridge` stays a path dependency: that
  crate is unpublished, so it follows the sibling checkout rather than the registry.
- Adopted core 0.50.0. `SecretsBootstrap::init` and `try_init` are `async`, so every caller
  awaits them; `AppPaths::from_profile` takes the active services-root override
  (`ServicesRootBootstrap`), which is what lets a fetched bundle become the running services
  tree; every authz rule writer stamps `access_control_rules.source`, so bundle-ingested rules are
  pruned only within their own bundle and never overwrite a dashboard-authored rule;
  `gateway.quota_fault_mode` selects what the gateway does when it cannot evaluate a quota window
  or read its policy row; and the profile JSON schema is regenerated for the new `services` and
  `secrets` keys.
- Adopted core 0.49.0 on the way: `SubjectAttributeProvider::values_for` and
  `gather_subject_attributes` are fail-closed (`Result<_, AuthzError>`), so the providers in
  `extensions/web/admin/src/authz/` propagate a database failure instead of treating it as "this
  user belongs to no group or project". This closes a path on `/v1/messages`: the gateway's
  `RouteEntitlementGuard` resolves route entitlement through `authz::subject_attributes_for`, and
  a lookup failure previously reached `resolve()` as an empty attribute set rather than a denial.
- Outbound HTTP that a caller can influence runs on core's guarded client, whose DNS resolver
  re-checks every resolved address against the SSRF block list before a socket opens and on each
  redirect hop. This covers this fork's authz webhook, MCP transports and gateway image fetches.
- The governance decisions log is one row per evaluated call rather than one per policy
  evaluation. Three producers write against a single trace (the gateway prompt gate, this fork's
  authz webhook and core's authz sink), so every request had been three near-identical `allow`
  lines. Each row carries a chain of pills and every evaluation still links to its own audit
  detail; the user column shows a person rather than a UUID and the target column names what was
  decided on. A "Needs attention" band summarises the calls that denied or warned, and
  `attention=1` filters to them. The CSV export keeps the flat row-per-evaluation query and gains
  `trace_id`.
- The `systemprompt` MCP server no longer proxies Atlassian. It wraps only this instance's own
  audit tables; Jira and Confluence are read by the client through the Atlassian connector it
  already holds, and the traffic-light arithmetic moved into the `admin_daily_brief` and
  `admin_critical_projects` skills.
- Marketplaces are consolidated to four (`astound-commons`, `astound-india-dev`,
  `astound-sales-uk`, `astound-super-admin`), five plugins and one daily-brief dashboard; the
  `admin_report` MCP server collapses to brief, projects and costs, and derives the lights from
  them. The contract baseline is re-recorded for the five routes removed with the Slack and
  requirements servers.
- Every core version pin flows through one place per workspace, so adopting a core release is a
  single mechanical step: `systemprompt-api` moved into `tests/Cargo.toml`'s
  `[workspace.dependencies]`, and `scripts/sync-release-version.sh` rewrites the
  `systemprompt-evaluation` and `systemprompt-api` pins in the test workspace too.
- The 0.48.0-specific release notes in `docs/install/docker.md` are marked
  `<!-- pinned-release -->` so the blanket version sweep leaves them as statements about that
  release.
- Contexts default to the last 30 days with an explicit All time option, paginate 50 users with
  three conversation previews each, and share filtered metrics and totals. Users, groups, projects
  and sessions reduce repeated lookups through batch aggregation; expensive dashboard read
  transactions disable PostgreSQL JIT without dynamic SQL. User detail pages use consistent
  form-section cards; admin data providers consolidate conversation request reporting.
- Bridge installation and download guidance follow the pinned core reference and supported
  connection flow; the obsolete macOS repair walkthrough is removed.

### Fixed

- `.sp-table__cell--stacked` set `display: flex` on the `<td>` itself, taking the cell out of the
  table box model so the painted cell and its zebra stripe stopped short of the column on the
  sessions page. The rule is scoped so the `td` keeps `display: table-cell`.
- The user tab's conversation table keeps to the listing width.
- The admin header's help icon and install button do something again.
- `just start` no longer lets a Docker hiccup preempt its takeover prompt.
- The test tiers build again against core 0.50 and ship the offline SQLx cache it needs.

### Removed

- The Slack integration and the `admin_console` agent that existed only to serve it.
- The knowledge-bank and requirements-workflow MCP servers, with their crates, configs, admin
  routes, jobs and tests. The Google Drive knowledge sync and the three-skill requirements
  workflow described in earlier notes are gone with them. The seven `[workspace.dependencies]`
  entries that existed only for the knowledge bank (`calamine`, `quick-xml`, `regex`, `futures`,
  `htmd`, `pdf-extract`, `zip`) are pruned.
- The Playwright suite is out of the repository and kept as a local dev flow.

### Operator notes

- **Profile keys.** `services.sources[]`, `services.cache_dir` and `services.on_fetch_failure`
  are new and unused this release; `gateway.quota_fault_mode` is new and defaults to `open`, so a
  profile that omits it behaves as before.
- **Bundle pinning, when you do it.** `ghcr_pull_token` (`<github-user>:<read:packages PAT>`)
  must exist in the production `secrets.json` before the first boot with a `services.sources`
  entry. `on_fetch_failure: use_bundled` boots without the Europe workspace on a failed fetch and
  reports it at `GET /api/v1/admin/services/status`; switch to `use_last_good` once a fetch has
  succeeded. Move the pin with `just services-pin <digest>` then `just deploy`, or
  `POST /api/v1/admin/services/refresh?restart=true`; rollback is the same with the previous digest.
- **Core migrations.** 017 adds `access_control_rules.source`. 018 adds a `BEFORE UPDATE`
  trigger that refuses every `UPDATE` on `governance_decisions` (a recorded decision is evidence;
  `DELETE` stays available for retention and erasure). 022 adds `ai_requests.upstream_latency_ms`,
  NULL for rows written before the migration.

## 0.47.0 — 2026-09-06

### Breaking



- **The web extension's migration chain is retired: a clean database bootstraps from the
  declarative schema and boot seeds alone.** All 22 `extensions/web/schema/migrations/*.sql`
  files are deleted. Core ≥0.32 stamps migrations as applied on fresh installs instead of
  executing them, so migration files only ever ran on established databases — and every known
  deployment (including the production Fly tenant, verified at head) had applied them; orphaned
  `extension_migrations` rows are inert. `departments` moves from `12_management.sql` to
  `16_organizations.sql` with its final shape (`org_id NOT NULL`, `departments_org_fk`,
  `idx_departments_org_name` — the names the old backfill migration created, so fresh and
  established databases converge). Boot seeds grow to three insert-if-absent files
  (`admin_oauth_client`, `marketplace_plans`, `house_organization`); the seed-contract test
  enforces both the manifest and `ON CONFLICT … DO NOTHING`. Demo tenants, users, and traffic
  leave the boot path entirely: they are seeded only by the demo flow
  (`demo/02-seed-demo-tenants.sh` + `demo/fixtures/demo-tenants.sql`), and
  `services/access-control/plans.yaml` no longer recreates the demo organizations on every
  boot. This removes the seed-data-in-migrations pattern whose checksum-drift/repair-replay
  failure mode caused the 2026-08-19 production boot loop.

### Added



- **A value-architecture diagram now accompanies the deployment topology.**
  `/documentation/deploy-target-architecture` gains a "Where the value sits" section showing
  the control plane full size above a compressed data-plane strip of the resources the platform
  team provisions, plus the four capabilities (WAF, IAM, Vault, monitoring) whose infrastructure
  equivalents cannot see agent semantics. `scripts/make-value-architecture.py` generates both
  `docs/architecture/value-architecture.drawio` — editable in app.diagrams.net — and the published
  `storage/files/images/architecture/value-architecture.svg` from one model, so the two cannot
  drift; edit the script, not the outputs.

- **Proposed target deployment architecture is now published documentation.**
  `/documentation/deploy-target-architecture` carries the platform team's OCI end-state
  diagram (`storage/files/images/architecture/oci-target-architecture.png`), states where the
  boundary between the platform team's infrastructure and this software falls, and buckets
  every component as already built, deliberate non-goal, or one of four named gaps — Vault-backed
  secrets, an object-storage backend, node-local generated content, and per-node scoping of
  spawned MCP processes. It is labelled proposed throughout and defers to
  `/documentation/deploy-oracle-cloud`, which remains the only architecture that runs. The
  reasoning, and the criterion for moving a line from the proposed page to the one that
  runs, were recorded in a decision record kept with the local engineering notes.

- The knowledge bank is now the full project-context RAG engine (Phase 0 of the AI Kit
  migration; the component specifications are kept with the local engineering notes, and
  the requirement ids they define are pinned by `tests/fixtures/project-context/`), served
  as a **remote MCP server** at
  `https://astound.systemprompt.io/api/v1/mcp/knowledge-bank` over **one shared index**.
  Users connect from Claude Code / Cowork with their own Atlassian e-mail and API token as
  `x-atlassian-email` / `x-atlassian-api-token` headers (optional `x-atlassian-base-url`);
  `sync_knowledge` — now open to every signed-in user — reads Confluence (v2 API, footer
  comments) and Jira (JQL search, comments) **as that caller** inside the
  `systemprompt-mcp-knowledge-bank` process and grants every document it writes to
  `user:<caller>`. A page several people can read is chunked and embedded once (content
  hash + embedding cache) and granted to each of them; on a complete fetch the caller's
  grants on pages that disappeared upstream are removed and documents left with no grants
  are deleted. Sync state is per `(source_type, user_id)`, so `list_knowledge_sources`
  reports the caller's own counts and last sync. Hits carry `date`, `date_source`, `lines`
  and chunk position so the agent policy in the `project_context` skill can rank by
  authority and recency.

- Admin-only `prune_knowledge {stale_days?, dry_run?}` drops the grants of users with no
  sync in `stale_days` (default 90) or whose account is gone, deletes orphaned documents,
  and prunes the date and embedding caches; `knowledge_index_stats` (now admin-only) adds
  syncing users, grants, cache rows and database size. The same numbers and the same prune
  SQL are reachable from psql with `just knowledge-db-{report,prune}`.

- The index gets its own Postgres on Fly: `deploy/fly/knowledge-db/` and
  `just knowledge-db-{create,provision,psql}` provision `sp-a2f658d8bc5f-knowledge` with
  pgvector (the shared `systemprompt-db-prod` cluster has no `vector` extension), and the
  `knowledge_database_url` secret points the child at it; where the secret is absent the
  instance database is used. Procedure: `docs/install/knowledge-db.md`.

- `services/skills/project_context` and the `project-context` plugin ship the agent-side
  contract (scope every query, archive opt-in, authority > recency) to every signed-in
  user through the `enterprise-demo` marketplace.



- The bundled `systemprompt` MCP server stamps `tools/list` and
  `resources/templates/list` through core's `build_tool_list_result` and
  `build_resource_template_list_result`, so both carry the SEP-2549 cache metadata
  (`ttlMs`, `cacheScope`) that protocol `2026-07-28` requires. A client that parks
  connectors on a missing stamp now sees this server as conformant.



- **Slack is now an inbound surface, restricted to admins.** `/systemprompt <cli command>` in
  Slack runs against this instance and answers in the channel. `services/slack/astound.yaml`
  declares the app (signing secret and bot token by reference, the bot shared with the existing
  outbound alerting in `slack_alerts.rs`) and routes the slash-command key — not a channel id,
  because core dispatches on every `message` in a routed channel, not only `app_mention`. Four
  gates stand between a Slack message and a CLI call, each denying by default: the workspace
  rule projected from `authz.allowed_roles`, the sender's identity, the scope of the token core
  mints for them, and the MCP server's own audience/scope/role requirements. A non-admin gets an
  ephemeral `⛔`, audited like any other refusal.

- **`services/agents/admin_console.yaml` — the first A2A agent this instance ships.** The
  inbound messaging pipeline can only dispatch to an agent; this one carries the `systemprompt`
  MCP server and nothing else, runs on port 9101, and declares `oauth.scopes: [admin]` so a
  token minted for a non-admin sender is refused at the A2A door. It is bound into the
  `astound-admin` plugin and declares its own admin-only rule in `roles.yaml`, which opts it out
  of the marketplace's `[user]` cascade.

- **`POST|DELETE /api/public/admin/users/{user_id}/slack-identity`** — link or detach the Slack
  account a user drives the platform from, for accounts whose Slack profile carries an
  unconfirmed or different email and so cannot take the automatic path. Body:
  `{"slack_user_id": "U…"}`. Writes the same `federated_identities` row the Salesforce Connect
  flow uses, under `https://slack.com`, and refuses to steal a mapping owned by another user.

### Changed



- **This repository now builds against published systemprompt core 0.47.0.** Both
  `[patch.crates-io]` blocks (`Cargo.toml`, `tests/Cargo.toml`) are commented out again,
  every core pin reads `0.47.0`, `bridge/CORE_REF` names the `v0.47.0` tag, and all three
  lockfiles resolve the core crates from crates.io. Core 0.47.0 counts reasoning tokens,
  excludes cached prompt tokens from `input_tokens`, retries transient upstream 429/503,
  unions every enabled marketplace into the bridge manifest, and refuses to boot a gateway
  route to a token-billed model with no declared cache-read rate — every priced model in
  `services/ai/providers.yaml` already declares one.

- **The gates run on one toolchain.** `clippy.toml` `msrv` now matches the manifest's 1.96,
  `preflight-static` format-checks `tests/` and `bridge/` as well as the root (the `tests/`
  workspace had never been gated and is formatted here), and the E2E job starts the stack
  detached so it reaches its health wait instead of spending its timeout on `just start`.
  Five repository functions the admin redesign no longer calls, but the internal fork still
  does, carry the dead-code gate's `lint-ok: unused-pub` annotation.



- **This repository now builds against published systemprompt core 0.46.0.** Both
  `[patch.crates-io]` blocks (root `Cargo.toml` and `tests/Cargo.toml`) are commented out, every
  pin moved to `0.46.0` via `scripts/sync-release-version.sh`, and `bridge/CORE_REF` is `v0.46.0`
  so the bridge is proven against the same core the server crates come from. Three consequences
  reach this repo directly. Governance gained `mode: enforce | warn` and a `Decision::Warn`
  verdict carrying the same `DenyReason` an enforcing policy would — the webhook handler in
  `extensions/web/admin/src/handlers/webhook/governance/` treats it as an allow and annotates the
  response, so a policy can be measured before it is enforced. User listings now exclude anonymous
  visitors by default: `create_anonymous` stores every visitor as an ordinary `users` row, so the
  admin directory had been spending a page of 50 on traffic rather than accounts; the opt-in is
  explicit (`list_including_anonymous` and siblings) and `--role anonymous` is unaffected. And the
  high-entropy secret backstop no longer denies filesystem paths — a macOS `$TMPDIR` scored as one
  high-entropy token, which denied **every** Claude Code request on an affected Mac.

- The knowledge-bank schema moved from `extensions/web/schema/19_knowledge_bank.sql`
  (never registered) to `extensions/mcp/knowledge-bank/schema/knowledge_bank.sql`, applied
  by the binary itself. **pgvector and `bedrock_api_key` are now hard requirements**: the
  child exits non-zero without them (`knowledge-bank: database <host> has no pgvector; it
  is required`) and core fails the platform boot — the inert `StubRetriever` mode is gone,
  so a deployment can no longer boot with a knowledge bank that silently cannot search.
  `services/mcp/knowledge-bank.yaml` stays `enabled: true` and the server grant in
  `services/access-control/roles.yaml` is every signed-in user; only `user:<id>` grants
  are written now, one per user who synced the document, and the retrieval-time grant join
  decides what each caller sees. `knowledge_sync_state` is replaced by
  `knowledge_user_sync_state` (keyed by source and user) and `knowledge_document_grants`
  gains `granted_at`. The decision records for the shared remote index and for running
  ingestion in process are kept with the local engineering notes.

- Model access sits behind a `ModelProvider` trait (`model.provider` in the config);
  Bedrock is the only implementation. A failing full-text arm or reranker now degrades a
  search instead of failing it.

- `services/knowledge-bank/config.yaml` changed shape: `bedrock:` moved under `model:`,
  `atlassian.base_url` is required, `knowledge_database.secret` names the index database
  secret, `ingestion` keeps only `doc_concurrency`, and every source declares `kind`
  (`confluence` | `jira`), kind-specific scope and `strategy` — nothing about who may read it.
  The `project_context` skill gained a "Connect" section with the `.mcp.json` headers.



- Adopted systemprompt core 0.38.0 from crates.io (typed marketplace keep-sets,
  the `keep_sets` authz resolver with bulk entity loading, `SubjectRef`/`DeviceId`
  identifiers, fallible `ContextId` construction, and the
  `ai_gateway_policies.priority` / `users.name`-uniqueness migrations). The
  marketplace filter's hand-rolled shrinking (`marketplace_filter/keepsets.rs`)
  is deleted in favour of core's `MarketplaceCandidate::retain_entries`, and
  polymorphic entity/subject references across the admin repositories and
  handlers now use typed ids (`EntityRef`, `SubjectRef`, `DeviceId`,
  `MarketplaceId`, `UserId`) instead of raw strings; JSON/template output is
  unchanged.

- Plan and ACL YAML loaders now validate the whole document before writing:
  a grant naming an entity with no catalog row is an error instead of minting a
  phantom catalog entry. Two kinds of inert grant left `plans.yaml`: the
  `astound-admin` marketplace grants (the admin control plane is not a
  marketplace — access rides the `roles.yaml` admin gating), and the pilot
  plan's `claude-opus-tier` gateway-route deny, whose route id no profile or
  roles pass ever registered.

- Governance SSR repositories are gated behind a new `governance-ssr` cargo
  feature (default off in this fork, which does not ship the upstream
  `ssr_governance` surface); the shared repository files stay identical across
  the fork family while their dead-in-this-fork queries compile out.

- The bridge is rebuilt against systemprompt-core 0.38.0 (the sibling
  `../systemprompt-core` checkout at the v0.38.0 release commit,
  `3730961ea4cd`) and restaged into `storage/files/downloads/`, tagged
  `bridge-v0.18.0`.

- Tracked systemprompt-core 0.36.0, then 0.37.0, on the way to the 0.38.0
  adoption above. The 0.36.0 hop was pin-only: the breaking
  `McpDomainError::PortHolderUnverifiable` variant is not matched in this repo, and
  the messaging and Slack APIs 0.36.0 changed are not used here.



- **A Slack app's `authz.allowed_roles` is now enforced, not just documented.** Core wrote the
  projection (`ingest_slack_apps`) but nothing ever called it, so the field described an
  intention no rule backed. `repositories/config/acl_yaml_loader.rs` now runs it at startup
  beside the `roles.yaml` pass, writing a `slack_workspace:<workspace_id>` entity with
  `default_included=false` and an allow rule per listed role. The Slack app file therefore stays
  the single place that says who may drive a workspace — `roles.yaml` deliberately carries no
  `slack_workspace` rules, only a pointer.

- **The `astound-dev` plugin (v2.0.0) now ships the full Astound development suite** — 68 skills,
  mirroring the team's [sfcc-next-cursor](https://github.com/Astound-Digital/sfcc-next-cursor)
  Cursor tooling repo. 62 skills are ported 1:1 into `services/skills/` (snake_case ids:
  `b2c-hooks` → `b2c_hooks`), spanning B2C build (`b2c_*`, incl. the 435-file offline `dw.*`
  corpus in `sfcc_api_classes`), Storefront Next (`sfnext_*`), operations (`atlassian`,
  `b2c_config`, `b2c_logs`, …), release (`git_commit`, `b2c_code`, `b2c_mrt`, …), and test
  (`playwright_cli`, `code_review`, `a11y_audit`, `systematic_debugging`). Because this instance
  ships skills rather than agents, the Cursor repo's 22 non-OpenSpec rules are folded into the
  new `dev_rules` skill and its 12 relevant agents/commands are folded into the skills they
  orchestrated (code-review + security-auditor → `code_review`, git-manager → `git_commit`,
  diagnosis/fix/fix-verification → `systematic_debugging`, perf-optimizer →
  `sfnext_performance`, scapi-cartridge-dev → `b2c_scapi_custom`, planning/feature-conductor/
  implement → the `dev_plan`/`dev_build` entry skills). The OpenSpec/opsx, project-context, and
  eval machinery was deliberately not ported. The four existing `dev_*` skills remain as
  entry-point routers with their `## Astound rules` drop-in sections now filled.

- **Four admin user-management skills in the `astound-admin` plugin**, all driving the
  admin-only `systemprompt` MCP CLI passthrough: `manage_users` (create/update/delete/merge/
  bulk/export), `block_users` (account suspension + IP bans — note `admin users ban` is keyed to
  an IP address, not a user), `manage_roles` (promote/demote/assign, with the
  sign-out-and-back-in token-reissue caveat), and `manage_sessions` (list/force-end/cleanup).
  Destructive commands document the confirm-then-`--yes` guardrail the CLI enforces. Each skill
  carries an admin-only entity rule in `services/access-control/roles.yaml`.



- **Adopted systemprompt-core 0.32.1 (published).** Patch bump via
  `scripts/sync-release-version.sh 0.32.1` (workspace version, `tests/` pins, image tags);
  lockfiles re-resolve the published crates. Carries core's `admin bootstrap` owner-email fix
  (explicit `system_admin.email`; already-bootstrapped installs unaffected) and the bridge
  browser-launch stdio fix, so both bridge binaries were rebuilt and redeployed.

- **Adopted systemprompt-core 0.32.0 (published).** All `systemprompt`/`systemprompt-security`/
  `systemprompt-extension` pins move from 0.31.0 to 0.32.0, both `[patch.crates-io]` blocks are
  re-commented (marker: `# INACTIVE: core 0.32.0 is published`) so the build resolves the
  published crates, lockfiles re-resolve against crates.io, and the `.sqlx` offline caches were
  regenerated (936 queries). Adoption fallout across the extensions tracks core's 0.32 API:
  the CLI session store's `load`/`load_or_reset` split, the Salesforce CLI's staged bootstrap
  errors (`stage_err`), and assorted admin/jobs error-adaptation sites now annotated under the
  tightened `check-http-errors` gate.

- **Adopted systemprompt-core 0.31.0 (published).** All `systemprompt`/`systemprompt-security`/
  `systemprompt-extension` pins move from 0.30.1 to 0.31.0 via
  `scripts/sync-release-version.sh 0.31.0` (workspace version, `tests/` pins, Helm chart 0.9.0
  with appVersion 0.31.0, and the CasaOS/DigitalOcean image tags), with both `[patch.crates-io]`
  blocks left commented so the build resolves the published crates. No adoption fallout: the
  extensions already compile against the 0.31 API surface and the `.sqlx` offline caches
  validate unchanged.

- **Adopted systemprompt-core 0.30.1 (published).** All `systemprompt`/`systemprompt-security`/
  `systemprompt-extension` pins move from 0.29.0 to 0.30.1 via
  `scripts/sync-release-version.sh 0.30.1` (workspace version, `tests/` pins, Helm chart 0.8.0
  with appVersion 0.30.1, and the CasaOS/DigitalOcean image tags), with both `[patch.crates-io]`
  blocks left commented so the build resolves the published crates, not the 0.31-in-progress
  sibling. Adoption fallout handled per `docs/RELEASING.md` Step A0: the `settings:` block in
  `services/config/config.yaml` moves to core's snake_case keys (`agent_port_range`,
  `mcp_port_range`, `auto_start_enabled`, `schema_validation_mode` — the camelCase forms now
  fail boot validation), migrations ran with the new binary, and the `.sqlx` offline caches were
  regenerated (932 queries).

- Three shared files re-synced with the template sibling to satisfy the fork-drift gate: the
  `systemprompt` MCP server's text fallback pairs the artifact with a `Ran `<command>``
  summary so structured clients don't see stdout twice, SSR render failures convert through the
  new `From<AdminTemplateError> for AdminError` instead of a formatted internal error, and a
  duplicated comment token in the governance authn handler is gone. Integration tests updated to
  core 0.30.1's wire shape, where a tool result's text block carries `summary\n\nbody`.

- `admin_user_report` is now strictly read-only: its role-mutation commands moved to
  `manage_roles`/`manage_users`, and its `session list` invocation was corrected to the CLI's
  positional form (`admin users session list <user-id>`).

- The `enterprise-demo` marketplace catalogue includes all 67 new skills; the marketplace
  version is unchanged, so the checked-in marketplace JSON needed no regeneration (the
  validation gate pins plugin list + marketplace version only).

- Both MCP server extensions (`systemprompt`, `knowledge-bank`) track core's current MCP API:
  output schemas come from `McpOutputSchema::validated_schema()` instead of
  `ToolResponse::schema()`, and tool dispatch threads the new `ClientProfile` through a
  `Dispatch` context struct. Integration tests updated to match.

- Contact email migrated from `ed@tyingshoelaces.com` to `ed@systemprompt.io` everywhere it was
  published: workspace `authors` (root + bridge), plugin author blocks, the web theme
  `support_email`, `AGENTS.md`, docs, examples, and the recorded demo SVGs.

- `scripts/build-coordinator.sh` fingerprints the `bridge/` tree, so bridge edits correctly
  invalidate coordinated build results.

### Fixed



- **The MCP CLI location and the ingestion job's settings were read from
  process-global state**, so tests that varied them were only correct one per
  process. The CLI's binary and working directory now come from a `CliLocation`
  resolved at the composition root, replacing the `SYSTEMPROMPT_CLI_PATH` and
  `SYSTEMPROMPT_WORKDIR` reads; the ingestion job takes `delete_orphans` as a job
  parameter and resolves its blog config from the job context's own `AppPaths`
  rather than the process-wide `BlogConfigValidated::cached()`. None of those
  environment variables was sanctioned, and nothing outside the tests set them.
  The subject-dimension registry here was already keyed per database.

- The inline-comment gate walked only top-level `tests/**/*.rs`, so every nested
  `extensions/**/tests/*.rs` passed vacuously. Widening the glob surfaced 22 real
  `///` uses in test code, now converted, and two clippy failures that had been
  invisible to the gate.

### Removed



- The knowledge bank's host-owned ingestion surface: `kind: files` sources and the PDF/DOCX
  parsers, the `storage/knowledge/` tree and the compose `app_knowledge` volume, the
  `ingestion.schedule` cron, the `systemprompt-mcp-knowledge-bank ingest` subcommand, the
  grant translator (`GrantResolver`, `atlassian.group_map`, `unmapped_groups`, per-source
  `grants:`), Confluence restriction reads and the Jira security-level fail-closed rule
  (Atlassian enforces both for the syncing user), and the `atlassian_base_url` /
  `atlassian_email` / `atlassian_api_token` service-account secrets. Users read Atlassian
  with their own credentials and are granted what they synced; there is no service account.



- **The repo is slimmed to what this instance actually deploys.** The Helm chart (`helm/`),
  the DigitalOcean/CasaOS deploy scenarios (`deploy/` now keeps only `clean-client/` and
  `salesforce/`), the `demo/` recording, scenario, architecture, and governance suites
  (`demo/` now keeps the preflight, seed, and fixtures used by live flows), and the
  `docs/install/` tree are deleted. The preflight source-gate suite grows to 23 gates and
  `preflight-full` adds a cargo-hack pass; `scripts/check-file-size.sh` and friends tolerate
  the removed directories.

- ~~`coverage/baseline.json`: the tracked coverage ratchet baseline is retired.~~ Reverted in
  the same release: `coverage-check` (part of `just preflight`) hard-fails without a baseline,
  so retiring the file silently disabled the ratchet. The baseline is re-recorded at the current
  coverage instead — total 82.51% (up from 81.17), global floor restored at 80.0.

- Stale `Unreleased` bullets describing the systemprompt-core 0.22→0.23 pin bump — that upgrade
  shipped long ago and is superseded by the released `0.26.0` entry below.

## bridge-0.17.1 — 2026-07-23

### Fixed

- The Windows bridge GUI now actually renders styled. The webview served itself from `http://sp.app`; the `.app` TLD is HSTS-preloaded in Chromium, so WebView2 force-upgraded every stylesheet/script request to `https://sp.app`, bypassing wry's `http://sp.*` interception filter — the document rendered but every subresource died on the real network. Core now serves the Windows GUI over `https://sp.app` with wry's `with_https_scheme`, so the HSTS upgrade is a no-op and assets are intercepted normally.

## bridge-0.17.0 — 2026-07-23

### Fixed

- The Windows bridge splash no longer renders as raw unstyled HTML. Core's generated GUI asset manifest keyed entries with host filesystem paths, so a native Windows build produced backslash keys (`css\main.css`) that never matched the webview's forward-slash URL lookups — every stylesheet and script 404'd silently. Core now normalizes manifest keys to URL form, fails the build on a backslash key or a missing `css/main.css`, and logs asset 404s instead of serving them silently.
- The Windows bridge no longer opens to a blank window. Core's GUI asset router missed the new session service module; core now generates its routing table from the staged web tree, and this repo's overlay opts out of core's Windows resource embed (`SYSTEMPROMPT_BRIDGE_WINRES=off`) so the branded icon is the only one linked.
- The bridge setup overlay is rebased onto core's current GUI: it gains the one-way setup latch and settled-snapshot guard (removes the splash flicker during startup probing), the preserved logo slot required by core's DOM reconciler, locale fallbacks in the gateway probe strings, and the splash-to-app fade.

## 0.14.8 — 2026-07-03

### Removed

- The vestigial pre-generated `storage/files/plugins/` tree and the `just marketplace` recipe (`core plugins generate`) that produced it. The bridge plugin-file endpoint assembles every bundle live from the `build_plugin_bundle` pipeline — the same bytes the gateway hashes into the signed manifest — so the static tree was never served and only invited drift from the manifest hash.
## 0.26.0 — 2026-07-28

### Changed

- Tracks systemprompt-core 0.26.0. The governance webhook supplies the `call_id` that `PolicyContext` now requires: the webhook is this call's only enforcement point and nothing upstream hands it an identity, so it mints one per request. A policy that accumulates state can use it to tell a repeat evaluation of one call from a second call.
- The deployment manifests (Helm, CasaOS, DigitalOcean) pin the 0.26.0 image; the Helm chart is 0.7.0 with appVersion 0.26.0. `render.yaml` tracks `latest` by design and is unchanged.

## 0.14.7 — 2026-06-03

### Fixed

- The `systemprompt` MCP server no longer self-deadlocks on reentrant CLI calls. The tool handler shelled out via the blocking `std::process::Command::output()` from inside its async `handle`, parking a Tokio worker for the lifetime of the child. When the child command was itself one that connects back to the same server (for example `plugins mcp tools --server systemprompt` calling `list_tools`), the parent held a worker waiting on the child while the child waited on the parent's server to answer, so the reentrant call only unblocked when the client's 30s timeout fired and returned an empty tool list. `cli::execute` is now `async` and uses `tokio::process::Command::output().await`, so the parent future yields while the child runs and the reentrant request is serviced normally.

## 0.12.0 — 2026-05-27

### Breaking

- **Aligned with core's split `access_control_entities` + `access_control_rules` schema.** Every direct sqlx call against `access_control_rules` either drops `default_included` from its column list or JOINs `access_control_entities` for it. Template-side `repositories::access_control` (`set_entity_rules`, `bulk_set_rules`) now upserts the catalog row before inserting grants — required by the FK migration 007 added on core. `AccessControlRule` and `AccessControlRuleInput` lose the `default_included` field; dashboard payloads carry it via the entity-level endpoints instead.
- **`gateway_acl::get_default_included` / `set_default_included` replaced** by `gateway_acl::get_entity` (returns `Option<EntityRow>`) and `gateway_acl::upsert_entity(pool, route_id, default_included, source)`. The webhook handler, marketplace filter, gateway catalog, `entity_access`, and `effective` modules transit through the new API; an absent catalog row resolves to `default_included: None`, which the core resolver maps to `DenyReason::UnknownEntity`.
- **Publish pipeline gains an `access_entity_bootstrap` stage** before `acl_yaml_load`. The stage upserts one `access_control_entities` row per `gateway.routes[]` declared in the active profile (`source = "profile:<path>"`). Without this stage the new FK on `access_control_rules` would reject every grant ingested from `services/access-control/`. MCP / agent / skill / plugin / marketplace bootstrapping comes with task #3.
- **Tool-use governance now runs on core's shared `Decision` / `GovernancePolicy` plane.** Every built-in policy (`secret_scan`, `scope_check`, `tool_blocklist`, `rate_limit`) implements `systemprompt_security::policy::GovernancePolicy` and returns the typed `systemprompt_security::authz::Decision` (`Allow { matched_by }` / `Deny { reason: DenyReason::… }`). Audit rows in `governance_decisions.evaluated_rules` are now produced from the typed `DecisionAudit { decision, principal, target, chain }` blob — the previous `serde_json::json!([{rule, result, detail}])` shape is gone. Downstream dashboards or alert rules that decoded the old `evaluated_rules` shape must be updated to the new schema; the top-level columns (`decision`, `policy`, `reason`) are unchanged.
- **`webhook::governance::types::GovernanceContext`, `RuleEvaluation`, `EvaluatedRule`, and `AuditRecord` were removed**, along with `webhook::governance::rules` (replaced by an inlined chain walk inside `webhook::governance::handler`). `Policy` / `PolicyContext` / `PolicyOutcome` no longer exist in `webhook::governance::policy`; the module re-exports `GovernancePolicy` from core. Extensions that registered third-party policies via `inventory::submit!` must switch to the core trait and return `Decision`/`DenyReason` typed values.

## 0.11.2 — 2026-05-25

Aligned with `systemprompt-core` 0.11.2: the gateway model allow-list moves from `services/ai/gateway-policies.yaml` into the profile catalog (`.systemprompt/profiles/<name>/catalog.yaml`).

### Breaking

- **`services/ai/gateway-policies.yaml` no longer carries `allowed_models:`.** Core's `GatewayPolicySpec` has dropped the field; the spec uses `deny_unknown_fields`, so a stale `allowed_models:` will fail boot. Exposed-model declarations move to the profile catalog instead.
- **`endpoint:` and `api_key_secret:` removed from every `gateway.routes[*]` entry.** Both fields now live exclusively on `GatewayProvider` in the catalog; the route references its provider by id and resolves endpoint + secret through the catalog. Core 0.11.2's `deny_unknown_fields` rejects route YAML that still carries them. Operators upgrading from 0.11.1 whose admin UI wrote those fields must strip them before boot — one-shot fix: `yq -i 'del(.gateway.routes[].endpoint) | del(.gateway.routes[].api_key_secret)' .systemprompt/profiles/<name>/config.yaml`. Endpoint + secret are managed at the provider level going forward.
- **`GatewayRouteView` admin DTO drops `endpoint` + `api_key_secret`.** Admin API clients posting `POST /api/admin/gateway/routes` no longer need to send (or can send) these two fields; serde drops them silently on input, and the persisted YAML omits them on output. The companion `validate_route` check loses the inline-secret-prefix detector along with the field it guarded.
- **Two-pass authz on `/v1/messages` (model + route).** The `extensions/web/admin/src/handlers/webhook/governance/authz.rs` webhook now sees both `EntityRef::GatewayModel(ModelId)` and `EntityRef::GatewayRoute(RouteId)` per request — the handler is entity-kind agnostic so no code change, but operators should expect roughly 2× rows in `governance_decisions` per inference call and may want to add model-scoped rules to `access_control_rules` to start exercising the new gate.

### Added

- **Profile gateway catalog (`gateway.catalog_path`)** points at a sibling `catalog.yaml` declaring providers + models (with optional aliases). The dispatcher's `is_model_exposed` gate consults the catalog before route resolution, so a wildcard route (`claude-*`) cannot leak a model the catalog has not declared. Adding a model means editing one file.
- **`just setup-local`** generates the catalog alongside the profile so fresh clones have a consistent baseline.

### Changed

- **`services/ai/gateway-policies.yaml` renamed to `services/gateway/policies.yaml`.** Tracks core's loader path move. The one-release fallback that briefly lived in core was removed before 0.11.2 ships — deployments still on the legacy path MUST move the file before upgrading (see core's 0.11.2 breaking notes).
- **`demo/scenarios/airgap/{02-load.sh,03-governance.sh,architecture.md}`** updated to reflect the new gate ordering, the new policy path, and that policies carry quotas/safety only.
- **`services/content/documentation/gateway-api.md`** points operators at the catalog as the model-exposure surface.
- **`justfile airgap-test` comment** updated to point at the new policy path.

## 0.11.0 — 2026-05-21

Aligned with `systemprompt-core` 0.11. Workspace version bumped from 0.9.2 → 0.11.0.

### Changed

- **Governance policy renamed `secret_injection` → `secret_scan`.** Clean break, no backward compatibility. The policy value emitted into `governance_decisions.policy` is now `secret_scan` (`extensions/web/admin/src/handlers/webhook/governance/policies/secret_scan.rs`). All read paths — repositories (`governance_grp/{portfolio.rs,risk_score.rs}`), the `14_audit_event_notify.sql` trigger, the homepage narratives in `extensions/web/site/src/homepage/demo_scanner/`, and every demo script — were updated to the new name in the same release. The dead `POLICY_SECRET_INJECTION` constant in `extensions/web/admin/src/types/constants.rs` was removed. **Any external dashboard, alert rule, or analytics query pinned to the literal `secret_injection` must be updated to `secret_scan`; historical rows still carrying the old policy string will no longer match any query and will not trigger the `audit_event_notify` breach severity.**

### Added

- **`016_swap_marketplace_admin_owner_to_admin.sql`** seeds the bootstrap `admin` user (`status='active'`, `roles=['admin','user']`) and re-owns the `marketplace-admin` OAuth client to it. Core's `oauth_clients.owner_user_id` NOT NULL constraint (core migration `004_oauth_client_owner`) wiped the synthetic owner introduced in `015_reseed_oauth_client_owner`; this migration replaces it with the real admin row that the scheduler resolves at startup.
- **`017_align_admin_email_with_cli.sql`** aligns the seeded admin's email with what core's CLI local-trial resolver expects (`admin@localhost.dev`). Without this, `admin agents message` would `find_by_email` miss and then collide on the `users.name='admin'` unique key when trying to auto-provision.
- **`015_reseed_oauth_client_owner.sql`** (band-aid kept for upgrade ordering) creates a synthetic `system` user owning `marketplace-admin` so fresh clones get past `010_seed_oauth` once core enforces NOT NULL `owner_user_id`. Superseded by 016 on the next migrate; the row is cleaned up there.

### Fixed

- **`docker/entrypoint.sh`** now runs `systemprompt admin bootstrap` between `infra db migrate` and `infra services start --foreground`. The scheduler refuses to start unless an active admin user resolves; the entrypoint previously assumed a human had run the bootstrap manually.
- **`justfile setup-local`** mirrors the same call after `just migrate`. Fresh clones on a developer machine now get an `admin` user without manual intervention.
- **`demo/00-preflight.sh` Step 0** now pre-checks `.systemprompt/credentials.json` expiry. Expired or absent creds produce a single actionable line and set `CLOUD_OFFLINE=1` for downstream demos; local-profile demos continue normally. Replaces the old behaviour where the cloud-token-expired WARN line was repeated on every CLI invocation throughout the suite.
- **`demo/00-preflight.sh` Step 3** now fails loud when the `/admin/profile` plugin-token scrape returns nothing. The previous silent fallback ("falling back to admin token") wrote the admin-scope JWT to `demo/.token`, so every `scope=service` demo silently degraded to `scope=admin` and analytics filtered on `session_id=plugin_cowork-bundle` returned empty. The fallback was tech debt masking the absence of a plugin-token mint command — see core issue D-4 for the missing `admin keys issue-plugin-token`. Demos that need plugin scope will not run until that command lands.
- `plugins mcp list`, `plugins mcp logs`, `plugins mcp validate`, and `admin agents registry` all work end-to-end against the template clone — the earlier AppPaths-not-initialized, missing-log-file, missing-`--service`, and registry JSON parse errors are gone with core 0.11.
- **`demo/users/01-user-crud.sh`** renamed to `01-user-listing.sh` to match its actual operations (list/count/stats/search only — no C/U/D). Mutating user demos remain isolated per the existing convention (see `04-ip-ban.sh`).

### Security

- Every scheduled job now requires an explicit `owner:` field in `services/scheduler/config.yaml`. The owner is a real admin username — there is no special "system" user. Existing installs must add `owner:` to each job entry; startup fails loudly until they do. The configured owner becomes `JobContext.actor` for every `execute()` call and the principal recorded in audit rows. See `services/content/documentation/authentication.md` for the full attribution model.
- Removed the synthesized `"admin"` fallback in the plugin-env handler. Requests without an authenticated cookie session and without an explicit `user_id` query parameter now return `401 Unauthorized` instead of impersonating the first admin user.
- Replaced the hardcoded `'system'` literal in the secret-migration audit log with the configured job owner. Every row in `secret_audit_log` now traces to a real `users` row.
- Added a `just lint-no-synthesis` guard (wired into `just clippy`) that fails the build if `UserId::new("…")` appears with a string literal in non-allowlisted extension code. Prevents future synthesis from sneaking in.

### Fixed

- `services/plugins/enterprise-demo/config.yaml`: dropped the dead `scripts:` block that referenced two missing files (`demo/01-seed-data.sh`, `demo/sweep.sh`). `core plugins validate` now reports zero errors for this plugin.
- `extension_migrations` tracking-table drift on the `web` extension reconciled (was 15 rows applied vs 11 declared). Migration-status summary now shows clean `11/11`. Clones with the same drift can either run `just repair-migrations` or `DELETE FROM extension_migrations WHERE extension_id = 'web' AND version IN (1, 4, 8, 13)` — the four legacy migrations consolidated out of the source tree.
