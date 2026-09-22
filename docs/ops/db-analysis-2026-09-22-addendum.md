# Production database analysis — addendum (second pass, 2026-09-22)

Companion to `docs/ops/db-analysis-2026-09-22.md`. Two sessions ran the brief in parallel on the same
restore (`sp_prod_analysis`); this file holds only what the main report does not already state, plus
corrections where the two passes disagree. Numbering continues from the main report's findings table.
Every query is in `docs/ops/db-analysis/queries/*.sql` (runner: `docs/ops/db-analysis/run.sh`,
results in `docs/ops/db-analysis/results/`). No message bodies, emails or names appear here.

Code references: astound `next` HEAD d0debca5 and core `next` HEAD ba2ebfbed, checked at tag `v0.58.0`;
"in a working tree" means the change exists only as an uncommitted edit in the sibling checkout.

**Status (2026-09-22, after the fix round).** A2, A3, A4, A5 and A7 are fixed and pushed:
astound `164ed51f` (conversation_facts attribution + web migration 099, authz allow
de-duplication, `plugin_usage_daily.plugin_id`) and core `94facf50b` (failed-request usage
settlement, bridge feed 5xx at error). A1 (ghost sessions) landed with
db-bloat-cleanup-schema's `session_source = 'web'` filter; A6 and A9 are owned by the
session that holds the main report. Nothing below was rewritten after the fixes — the
evidence is what the production dump showed before them.

## Corrections to the main report

- **Finding 10c is not a drift.** The two UNIQUE constraints the baseline declares
  (`mcp_tool_executions_user_id_mcp_execution_id_key`, `plugin_usage_events_user_id_id_key`) exist in prod as
  unique *indexes* `mcp_tool_executions_owner_id (user_id, mcp_execution_id)` and
  `plugin_usage_events_owner_id (user_id, id)` (`b07_schema_drift.sql`). Same shape, different name.
- **Finding 10d (`finish_reason` / `served_provider` NULL) is unproven at 0.58.0.** `settle_completion` writes
  both at the tag (`crates/domain/ai/src/repository/ai_requests/settlement.rs`, 3 refs to `finish_reason`),
  but the columns were only added by ai migrations 028/029 at 07:31:59 on 2026-09-22 and the last request
  in the dump is 07:31:23 — zero requests ran on the new binary (`d22_ai_requests.sql`,
  "requests after today's 07:32 upgrade": 0). Re-check after the instance has served traffic on 0.58.x.
- **Every bridge session is a "behavioral bot"** — the main report lists 1,620 flagged sessions as "not a
  fault". It is a fault; see A1.

## A1 · P1 — `ghost_session_cleanup` flags every bridge and API session as a bot

```
SELECT session_source, count(*), count(*) FILTER (WHERE is_behavioral_bot) behavioral,
       count(*) FILTER (WHERE is_behavioral_bot AND EXISTS (SELECT 1 FROM ai_requests r WHERE r.session_id=s.session_id)) behavioral_with_ai
FROM user_sessions s GROUP BY 1;
-- bridge | 1070 | 1070 | 103      api | 2 | 2 | 2      web | 803 | 362 | 0      oauth | 272 | 186 | 0
SELECT behavioral_bot_reason, count(*) FROM user_sessions WHERE session_source='bridge' GROUP BY 1;  -- ghost_session 1068
```

1,070 of 1,070 bridge sessions and both `api` sessions — i.e. **every session that carries AI spend** — are
`is_behavioral_bot = true, reason = ghost_session`. Two faults compound:

1. The gateway path never touches `user_sessions.request_count` (the HTTP analytics middleware that calls
   `increment_request_count` — core `crates/domain/users/src/sessions/mutations/mod.rs:35-50` — does not run
   for `/v1/*`), so 1,068/1,070 bridge sessions have `request_count = 0` while holding up to 613 AI
   requests each. Same root as main-report finding 1.
2. `GhostSessionCleanupJob` (core `crates/app/scheduler/src/jobs/ghost_session_cleanup.rs:33-47`, every
   10 min, 2,794 runs) marks `request_count = 0 AND landing_page IS NULL AND entry_url IS NULL` as a ghost.
   Bridge sessions have no landing page by construction.

Readers that exclude these sessions today: every query in
`extensions/web/admin/src/repositories/dashboard/traffic/{mod,kpis,queries}.rs`
(`AND NOT COALESCE(is_behavioral_bot, false)` at mod.rs:65,111,131,154; queries.rs:19,80,94; kpis.rs:18) —
the Traffic dashboard counts developer sessions as bot traffic. The flag is also projected into
`analytics_report_user_sessions` (core `crates/domain/analytics/src/projection/sources.rs`), so it will
propagate to every analytics surface once the rebuild lands. Each flag write is an UPDATE that fires
`reporting_capture` on `user_sessions` (one outbox fact per ghost mark).

**Fix.** Exclude `session_source IN ('bridge','api')` (or any session with a gateway request) from the ghost
predicate; count gateway requests into `request_count` or stop treating it as the liveness signal. Backfill:
`UPDATE user_sessions SET is_behavioral_bot=false, behavioral_bot_reason=NULL WHERE behavioral_bot_reason='ghost_session' AND session_source IN ('bridge','api')`.
Migration: data backfill only.

## A2 · P1 — `conversation_facts` multiplies hook and governance counts across contexts that share a client session

```
SELECT sum(hook_event_count), (SELECT count(*) FROM plugin_usage_events) FROM conversation_facts;   -- 14923 | 3348
WITH shared AS (SELECT client_session_id FROM conversation_facts WHERE client_session_id IS NOT NULL GROUP BY 1 HAVING count(*) > 1)
SELECT sum(f.hook_event_count), (SELECT count(*) FROM plugin_usage_events p WHERE p.session_id IN (SELECT client_session_id FROM shared)),
       sum(f.gov_allow+f.gov_warn+f.gov_deny), (SELECT count(*) FROM governance_decisions g WHERE g.session_id IN (SELECT client_session_id FROM shared))
FROM conversation_facts f WHERE f.client_session_id IN (SELECT client_session_id FROM shared);
-- 14529 | 2604 | 13090 | 1783
SELECT n contexts_per_client_session, count(*) FROM (SELECT client_session_id, count(*) n FROM conversation_facts WHERE client_session_id IS NOT NULL GROUP BY 1) x GROUP BY 1 ORDER BY 1;
-- 1:82  2:28  3:6  4:3  5:1  6:3  7:3  8:1  9:2  10:2  11:1  12:1  16:1
```

`refresh_conversation_facts` (`extensions/web/schema/45_conversation_facts.sql:179-203`; body last redefined in
`migrations/089_tool_artifacts_without_view.sql`) joins `plugin_usage_events`, `governance_decisions` and
`analysis_skill_events` on `session_id = a.client_session_id`. One Claude Code session produces many gateway
contexts (sub-agents, compaction, `/clear`; up to 16 here), so the same hook events are credited to every
context of the session: **4.5× over-count instance-wide (14,923 vs 3,348), 5.6× on shared sessions**, and
the same for `gov_*` and `skill_invocations`. The refresh is self-consistent (C12: 0 diffs on re-run), so
the fault is in the join, not in staleness.

Readers: Conversations page (`repositories/analysis/conversations/page.sql`, columns `hook_event_count`,
`gov_*`, `skill_invocations`), Skills page `facts` CTE (`analysis/skills/page.sql` joins
`f.client_session_id = s.session_id` and sums per skill — the same over-count, squared), session cost list
(`analytics/site/sessions.rs:28`, now reading `conversation_facts`).

**Fix.** Attribute session-scoped rows to *one* context per session (the context whose request window contains
the event's `created_at`, else the latest), or move `hook_event_count`/`gov_*`/`skill_*` to a per-client-session
fact table and join from there. Migration: function body + refresh.

## A3 · P2 — Two authz hooks audit the same evaluation twice per gateway request

```
SELECT count(*) pairs, count(*) FILTER (WHERE a.decision=b.decision AND a.reason IS NOT DISTINCT FROM b.reason) same
FROM governance_decisions a JOIN governance_decisions b ON a.trace_id=b.trace_id AND a.tool_name=b.tool_name
 AND a.policy='authz' AND b.policy='authz_rule_based' AND abs(extract(epoch FROM a.created_at-b.created_at))<1;   -- 4695 | 4695
```

`policy='authz'` rows are written by the astound authz webhook (`extensions/web/admin/src/handlers/webhook/governance/authz.rs:33,164-181`);
`policy='authz_rule_based'` rows by core `RuleBasedHook` through `DbAuditSink::record`
(`crates/infra/security/src/authz/audit/db_sink.rs:30-108`, policy name at `authz/audit/mod.rs:47`). Both are
mounted in the composite (`authz/runtime.rs:11-31`: "every webhook or extension composition carries
`RuleBasedHook` in front") and both evaluate `access_control_rules` for the same entity — the `evaluated_rules`
blobs differ in shape but the decision and reason are identical in 4,695/4,695 pairs. The composite doc
(`authz/composite.rs:14-24`) calls N rows per trace intended, but readers count *rows*:
`governance/hook_events.rs:24 count_pretool_fired_24h` (COUNT(*) over all decisions), `overview/series.sql:35`,
`traces/stats.sql`, and `conversation_facts.gov_allow` (13,679 vs 5,821 `default_allow` + 879 `secret_scan`
+ 73 `quota` — the authz pairs are two-thirds of the number). `evaluated_rules` is 14 MB, half of it this
duplicate.

**Fix.** Either drop `RuleBasedHook` from the composite when an extension hook is present (it is the same
rules table), or have readers count `DISTINCT trace_id` per policy family. Migration: none (data may be
left; readers change).

## A4 · P2 — Aborted streams settle as failed with no usage: uncosted spend

```
SELECT count(*), sum(tokens_used), sum(cost_microdollars) FROM ai_requests WHERE error_message LIKE 'stream ended without stop%';  -- 34 | NULL | 0
```

34 requests (haiku 19, opus 10, sonnet 5; 10 on 09-18, 9 on 09-21) ended with `stream ended without stop
event` and carry no tokens and zero cost, although the provider streamed — and bills — the output up to the
abort. The Failed branch of settlement (`crates/domain/ai/src/repository/ai_requests/settlement.rs:78-123`,
UPDATE … WHERE status <> 'completed') records only the error; `complete.rs` computes usage only on the
Completed path. Same class as the 3 stuck `pending` rows (main report 10f): every non-completed request is
$0 in every cost view, quota bucket and rollup.

**Fix.** On the failed path, settle whatever `message_delta.usage` / partial usage the stream tap accumulated
(`crates/entry/api/src/services/gateway/stream_tap.rs`) and cost it; if none arrived, estimate input tokens from
the prepared body. Migration: none.

## A5 · P2 — `/v1/bridge/latest` has returned 502 on every hit since 2026-09-11

```
SELECT timestamp::date, substring(message FROM '-> ([0-9]{3})') st, count(*) FROM logs WHERE message LIKE 'GET /v1/bridge/latest%' GROUP BY 1,2;
-- 09-08..09-10: 404 only (feed unconfigured); 09-11 onward: 502 every day (34,19,10,15,71,130,78,62,34,26,63,35) alongside 404s
```

577 × 502 out of 960 hits. The handler (core `crates/entry/api/src/routes/gateway/bridge_release/mod.rs`)
returns 404 when `gateway.bridge_releases` is absent and 502 when GitHub resolution fails — so the feed was
configured on 09-11 and has never resolved since. Every desktop bridge's self-update check has failed for 11
days, and the cached-answer fallback in the module head cannot help because no answer was ever cached. The
diagnostic (`logged_failure`, `tracing::warn!` with `detail`) does **not** reach the `logs` table — only the
access-log line does — so the operator cannot see *why* from the console.

**Fix.** Check the `bridge_releases` repo/token in the production profile; make the warn line persist
(module filter on the DB log layer) or fold `detail` into the access-log metadata for non-2xx. Migration: none.

## A6 · P2 — Bridge mints a fresh `user_sessions` row every hour per user

```
SELECT percentile_cont(0.5) WITHIN GROUP (ORDER BY extract(epoch FROM expires_at-started_at))::int,
       percentile_cont(0.5) WITHIN GROUP (ORDER BY extract(epoch FROM ended_at-started_at))::int, count(DISTINCT user_id)
FROM user_sessions WHERE session_source='bridge';   -- 3600 | 5955 | 20
SELECT user_id, count(*) FROM user_sessions GROUP BY 1 ORDER BY 2 DESC LIMIT 1;  -- 256 sessions for one user in 18 days
SELECT n distinct_client_sessions_per_gateway_session, count(*) FROM (SELECT session_id, count(DISTINCT client_session_id) n FROM ai_requests GROUP BY 1) x GROUP BY 1;
-- 0:23  1:54  2:13  3:8  4:1  5:2
```

1,070 bridge sessions for 20 users in 20 days, expiry 1 h after start, 1,068 with zero HTTP requests. A
gateway "session" is a one-hour token window, not a Claude Code session: one gateway session spans up to 5
client sessions and one client session spans several gateway sessions. Every per-session number
(`list_project_sessions`, session cost list, `list_mcp_server_activity.distinct_sessions`) therefore counts
token refreshes. Combined with A1 it also makes each hour's session a new "ghost".

**Fix.** Key bridge sessions on the client session (`client_session_id`) or renew instead of re-minting;
report sessions from `conversation_facts.client_session_id` in the console. Migration: none.

## A7 · P3 — `plugin_usage_daily.plugin_id` is NULL on every row

```
SELECT plugin_id IS NULL, count(*), sum(event_count) FROM plugin_usage_daily GROUP BY 1;   -- t | 514 | 3348
```

The rollup reconciles exactly with `plugin_usage_events` (C13) but `upsert_daily_aggregation`
(`extensions/web/admin/src/repositories/dashboard/usage_aggregations/daily.rs:19-55`) never carries the
plugin, so nothing downstream (`admin_usage_daily_rollups`, per-plugin dashboards) can split hook usage by
plugin from the daily table; they fall back to the events table or the skill-key prefix (migration 090).
**Fix.** Stamp `plugin_id` from the event (or drop the column). Migration: data backfill.

## A8 · P3 — The `systemprompt` MCP omni-tool fails on CLI flags that do not exist

```
SELECT left(regexp_replace(error_message,'[0-9]+','#','g'),90), count(*) FROM mcp_tool_executions WHERE server_name='systemprompt' AND status='failed' GROUP BY 1;
-- unexpected argument '--offset' (2), '--since' (1), '--until' (1), '--export' (1); unrecognized subcommand 'report' (2), 'create' (1); Unknown role 'developer' (2); Atlassian connection failed (2)
```

10 of the 19 `systemprompt`-server failures are the admin skills invoking `systemprompt infra …` /
`systemprompt admin …` with arguments the shipped CLI does not accept — skill text and CLI contract have
drifted. **Fix.** Regenerate the CLI reference in the `astound-admin` skills from `--help` (or add the
flags). Migration: none.

## A9 · P3 — `sync_state` is empty; `analytics_ingestion_producers.pending_count` drifts

Both confirm main-report findings 4 and 6 from a different angle: `sync_state` has 0 rows although
`access_control_rules` holds 159 `source='yaml'` rows (seeding bypassed `repositories/sync/state.rs:83,117`),
and `pending_count` (11,164) ≠ pending rows (11,161) after a single morning's backfill. The only writer is
`enqueue_feedback_fact`'s `was_pending IS DISTINCT FROM true` branch (`30_feedback_ingestion.sql:55-59`);
the mechanism of the +3 is not isolated here, but a counter that can drift by 3 in one run will drift by
more, and `expire_feedback_raw` gates on it (main report 6).

## Verified clean (in addition to the main report)

- `tokens_used = input + output + cache_read + cache_creation` on all 4,287 completed rows; `reasoning_tokens`
  is a subset of `output_tokens` in 1,844/1,844 rows (C11).
- `request_id` unique; `id = request_id` on 4,392/4,392 (gateway path only — the internal path never ran).
- `ai_request_scopes` covers 4,392/4,392; 501 requests from users in no group are stamped with the system
  `Unassigned` group, none with a missing group/project (C16, C20).
- `ai_safety_findings` 0 orphans; `mcp_artifacts` ↔ `mcp_tool_executions` 284/284, `artifact_payloads`
  `ref_count` exact (C17, C18).
- `admin_usage_daily_rollups` vs `ai_requests`: only today's two partial rows differ (job at 07:05, dump 10:25).
- No admin repository SQL (32 files, generic plans in `results/plans/`) seq-scans a table over 100k rows;
  the two heaviest (`analytics/requests/{paged,kpis}.sql`, cost ≈7.6k) are index scans with two correlated
  sub-plans per row. `users/roster/page.sql` references `user_last_seen`, which does not exist in prod
  (migration 092 is uncommitted in this tree).
- `E30`: zero rows keyed to a non-existent user in any user-keyed table (`user_id='system'` sentinel excluded).

## Finding → code path → proposed fix → migration?

| # | finding | code path | fix | migration |
|---|---|---|---|---|
| A1 | every bridge/API session flagged `ghost_session` bot | core `crates/app/scheduler/src/jobs/ghost_session_cleanup.rs:33-47`; readers astound `repositories/dashboard/traffic/*.rs` | exclude gateway-sourced sessions from the ghost predicate; backfill flags | data |
| A2 | conversation_facts ×4.5 hook/gov/skill over-count | astound `extensions/web/schema/45_conversation_facts.sql:179-203` (+ mig 089 body) | attribute session-scoped rows to one context, or per-client-session facts | function + refresh |
| A3 | authz audited twice per request | astound `handlers/webhook/governance/authz.rs:164`; core `crates/infra/security/src/authz/audit/db_sink.rs:30`, `runtime.rs:11-31` | one evaluator per rules table, or readers count DISTINCT trace | none |
| A4 | aborted streams uncosted | core `crates/domain/ai/src/repository/ai_requests/settlement.rs:78-123`, `gateway/audit/complete.rs` | settle partial usage on the failed path | none |
| A5 | `/v1/bridge/latest` 502 since 09-11; diagnostic never persisted | core `crates/entry/api/src/routes/gateway/bridge_release/mod.rs:78-91` | fix `bridge_releases` config; persist the warn detail | none |
| A6 | one gateway session per hour per bridge user | bridge session minting / `expires_at` = +1 h | renew instead of re-mint; key on client session | none |
| A7 | `plugin_usage_daily.plugin_id` always NULL | astound `repositories/dashboard/usage_aggregations/daily.rs:19-55` | stamp plugin_id | data |
| A8 | admin skills call CLI flags that do not exist | `services/skills/*` (astound-admin), CLI | regenerate skill CLI references | none |
| A9 | `sync_state` never written; `pending_count` drift | astound `repositories/sync/state.rs:83,117`; `30_feedback_ingestion.sql:55-59` | seed writes state; derive pending from `count(*) WHERE pending` | none |

## Follow-ups that belong on core `next` (with the proving query)

1. Ghost predicate (A1) — `SELECT count(*) FROM user_sessions WHERE behavioral_bot_reason='ghost_session' AND session_source='bridge'` must be 0 after a day of traffic.
2. Failed-path usage settlement (A4) — `SELECT count(*) FROM ai_requests WHERE status='failed' AND error_message LIKE 'stream ended%' AND coalesce(tokens_used,0)=0` trends to 0.
3. Bridge release feed diagnostics (A5) — `SELECT count(*) FROM logs WHERE module LIKE '%bridge_release%'` > 0 when the feed fails.
4. Composite authz double audit (A3) — `SELECT count(*) FROM governance_decisions WHERE policy='authz_rule_based'` stops growing when the extension hook is mounted.
5. Uncommitted in the core working tree and needed by main-report finding 7: `crates/infra/logging/schema/migrations/007_drop_logs_projection.sql` and the `POLLING_ROUTES` exclusion in `routes/gateway/access_log.rs` — neither is at HEAD ba2ebfbed.
