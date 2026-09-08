# Dashboard query scaling

The dashboard reads live data. Conversation pages now share one filtered numeric
aggregate for rows, KPIs and counts. They resolve titles and opening prompts for
the selected rows; title search is the deliberate exception. No cached or stored
conversation summaries are introduced.

## Behavior and invariants

- `/admin/contexts` defaults to 30 days and By user. `since=all` restores All time;
  a nonempty `since` takes precedence over `preset`. Invalid values use 30 days.
- By user pages through 50 users and renders three latest matching conversations
  each. Its counts and sort describe users; All pages through 50 conversations.
  Preview links preserve scope/search/time/model/side-call filters.
- Time selects conversations by last activity. Old turns within those
  conversations still contribute to classification and lifetime totals.
- A thread's second request can change the first request from utility to turn.
  Classification uses the complete thread, including requests by another user.
  Conversation ownership still comes from its latest request. Equal timestamps
  use request ID as a deterministic tie-breaker.
- Empty deep pages retain filtered counts. Internal JSON result envelopes decode
  persisted context IDs through a companion typed SQLx result array; external
  identifier validation is unchanged.
- Expensive interactive reads disable PostgreSQL JIT inside a transaction only.
  At one million synthetic requests, JIT compilation alone took about two seconds
  in an execution plan. Commit/rollback restores the pooled connection setting.

## Audit

| Surface | Finding and treatment |
|---|---|
| Contexts | Repeated rollups, 400-row preview cap, incorrect user pagination: one statement, 50 users × three previews, no unused user-summary read on All. |
| Users roster | Per-user spend scans and latest-context enrichment before pagination: grouped 30-day spend, identity/search predicates early, latest context after selecting the page. KPI input restricted to 60 days. Reuse group/project names. |
| Sessions list | Current metrics read twice: reuse the page's current totals; read only the preceding window separately. |
| Profile and recent conversations | Shared conversation list now bounds candidate contexts by user. Usage-pane recent rows explicitly pass the user's context set to numeric aggregation. Preserve request-window metrics, which differ from whole-conversation metrics. |
| History | Repeated classified-request reads, regex over every message, headlines before paging: reuse classification, defer opening prompts and headlines until page selection, retain search-dependent processing before filtering. Both sources and existing redaction remain. |
| Context detail | Existing predicates already select one context. The compatibility classification view and indexed request/message joins remain; no listing-wide title rollup is needed. |
| Session detail | Whole-site rollup joined to one session: aggregate only contexts belonging to the requested session and resolve titles after grouping. |
| Groups | Marketplace access rules/defaults loaded once per group: use the existing batch resolver so every group shares the same inputs. Usage queries already use explicit time windows and preserve exclusive/full membership distinctions. |
| Projects | Three correlated usage aggregates per project: group each activity source by project before joining. Existing 500-project listing cap/truncation indicator remains. |
| Analytics dashboard | Active tabs already load only their own data. Current/prior KPIs already share a statement and bound request history to the required windows. Code metrics already use existing daily rollups. Preserve these patterns. |
| Requests/traces and other lists | Inspected repository calls for conversation-view dependencies and message/title processing: no additional conversation rollup consumers remain outside the listed surfaces. Their independent operational aggregates and filtering retain their existing semantics. |

Query-level pool wait and execution timing are logged at debug level for the
shared conversation page. Scope resolution and SSR rendering have separate debug
timings. SQL plans and the HTTP benchmark distinguish database costs from template
rendering; a successful 200 response alone is not performance evidence.

## Reproduction

Run on a disposable database initialized from the current schema. The benchmark
refuses database names without `_test`. Supply its URL through
`DASHBOARD_BENCH_DATABASE_URL`; do not put credentials in committed files.
Python requires `psycopg` 3.

```bash
python3 scripts/benchmark-dashboard.py --seed 10000 --measure
python3 scripts/benchmark-dashboard.py --seed 100000 --measure
python3 scripts/benchmark-dashboard.py --seed 1000000 --measure
just dashboard-benchmark
```

Use separate freshly initialized databases for each size when comparing final
changes. The deterministic fixture has 1,000 users, ten requests per conversation,
activity spread across 120 days, side calls, three models and opening messages.
The million-request fixture therefore contains 100,000 conversations. It is not
a substitute for an estate with large transcripts, high tool volume or many
membership rules.

SQL results and `EXPLAIN (ANALYZE, BUFFERS)` plans are written under
`.build/dashboard-performance/`. The SQL runner reports five warm samples and
30 samples across ten concurrent readers for default contexts/users views.
The release HTTP benchmark exercises the real route authentication, SQL,
templates and response body with ten readers. It excludes TCP and the standalone
binary's outer middleware; it does not restart the live server.

The performance target is warm default-page p95 below 500 ms at the million-request
size. It is a target, not an assertion baked into CI: shared-host timing is noisy.
All-time/search/aggregate-sort/deep-page variants are measured separately. Report
missed targets explicitly; do not hide old conversations or change numeric
semantics to produce a faster benchmark.

## Validation record

Status at wrap-up (2026-09-08): validation is incomplete.

- A local conversation query improved from approximately 772 ms to 70 ms in
  rolled-back schema experiments. This is query timing, not HTTP load time.
- At one million synthetic requests, later query samples were approximately
  2.1 seconds for All and 3.8 seconds for By user. The 500 ms target is not met;
  these shared-host samples are not a completed concurrent HTTP benchmark.
- The focused database run passed 23 of 25 tests. History tests
  `both_sources_appear_for_the_same_viewer` and
  `search_matches_a_gateway_conversation_by_its_opening_prompt` fail because
  utility-only conversations lose their opening preview/search match. This is
  a known regression requiring follow-up before deployment.
- At wrap-up, formatting and SQLx cache presence checks passed. Full preflight
  failed source gates covering fail-open handling, layers, repository construction,
  JSON values, SQLx usage, HTTP errors, raw IDs, comments, duplicate types, dead
  repository code, and the core reference. These include dashboard and existing
  repository findings. Full integration, contract, release HTTP, and coverage
  validation have not completed successfully. Benchmark tooling is included for
  follow-up runs.
- No application restart or deployment was performed.

