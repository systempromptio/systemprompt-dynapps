# Production database analysis — systemprompt-astound 0.58.0 (self-host, Astound)

Date: 2026-09-22 · Source: `systemprompt_full_db.sql.gz` (plain `pg_dump`, PG 17.6, taken ≈10:25 UTC) · Brief: `docs/ops/db-analysis-brief.txt`

Restored read-only into `sp_prod_analysis` on this clone's Docker Postgres (`pgvector/pgvector:pg18`, container `sp-systemprompt-astound-dcd323ed-local-postgres-1`): 237 tables, 1.45 GB, zero restore errors. The database is left in place for re-querying; `DROP DATABASE sp_prod_analysis` removes it. Every query is in the appendix and in `docs/ops/db-analysis-2026-09-22/`; the only write was one `ANALYZE` and the rolled-back `refresh_conversation_facts(NULL)` in §12. No message bodies, emails or names appear below.

Code references are against astound `next` at 9ae630a4 and core `next` at ba2ebfbed (= `bridge/CORE_REF`); both carry tag `v0.58.0` and every finding's code path was confirmed to exist at that tag unless stated.

---

## Executive summary

The instance is 20 days old (first row 2026-09-02), 18 real AI users behind 1,179 anonymous visitor rows, 4,392 gateway requests ($279.64), 421 conversations, 550 MCP tool executions. It was upgraded to 0.58.0 on the morning of 2026-09-22 and hit the known baseline-rebuild boot loop (evidence in §15/§27). The data is small, so nothing is slow *today*; the findings are about numbers that are already wrong and about designs that stop scaling within the year.

| # | Rank | Finding | Evidence | Code path |
|---|------|---------|----------|-----------|
| 1 | **P1** | **Session AI counters are never written.** `user_sessions.ai_request_count / total_tokens_used / total_ai_cost_microdollars / error_count` are 0 on all 2,147 sessions while 4,392 requests / 312 M tokens / $279.64 / 95 failures exist. Every gateway request is settled by `settle()`, which never calls `increment_ai_usage`; only the legacy in-process `RequestStorage::store` path does. | §10 | writer that should run: core `crates/domain/users/src/sessions/mutations/mod.rs:221`; gateway settlement that skips it: core `crates/entry/api/src/services/gateway/audit/journal/settle.rs:75`; the only caller: core `crates/domain/ai/src/services/core/request_storage/writes.rs:93-110` |
| 2 | **P1** | **Two context-id spaces.** Gateway rows (`ai_requests`, `user_contexts`) use `ContextId::derived_from_client_session`; authz decisions, MCP executions, `analytics_events` and hook artifacts use `derived_from_session` (the bridge session). Result: 8,784 of 13,241 governance decisions carry a context_id that is not the request's; 550/550 MCP executions, 17,743/17,743 analytics events and 11,706/16,163 decisions point at contexts that do not exist in `user_contexts`. | §9, §12 | core `crates/shared/identifiers/src/context.rs:59-69`; authz: core `crates/infra/security/src/policy/audit.rs:182-185`; session middleware: core `crates/entry/api/src/services/middleware/session/context.rs:77,99,138`; gateway: core `crates/entry/api/src/routes/gateway/messages/extract/mod.rs:230` |
| 3 | **P1** | **Tool calls never pair with executions.** `ai_request_tool_calls.mcp_execution_id` is NULL on 2,901/2,901 rows; `mcp_tool_executions.ai_tool_call_id` NULL on 550/550. `conversation_facts.tool_calls_executed` is therefore 0 for all 421 conversations (intended = 2,901, executed = 0). Replaying the claim predicate at each execution's `started_at` shows 138 of 303 in-process executions *were* claimable, so the claim path is not executing or not matching. | §12, §17 | core `crates/domain/mcp/src/repository/tool_usage/intent.rs:17-74` (`claim_unclaimed_intent`), call site core `crates/domain/mcp/src/tool/mod.rs:146-176`; consumer astound `extensions/web/schema/*` `refresh_conversation_facts` `executions` CTE |
| 4 | **P1** | **Access-control DB is the pre-Salesforce entitlement set.** The DB holds 63 entities (UK-sales marketplace, `admin_console` agent, a Slack workspace, knowledge-bank/requirements-workflow servers, 11 generated `gateway_route/*`); the 0.58.0 `rules.yaml` declares 20, of which `marketplace/astound-salesforce` and all 7 `mcp_server/salesforce-*` + `google_workspace` are **absent from the DB**. `sync_state` is empty — no plane has ever recorded an apply. Boot only compares, so the shipped Salesforce marketplace is unentitled until someone presses Overwrite on `/admin/sync`. | §19 | seeding/compare: astound `extensions/web/admin/src/repositories/sync/*` (boot contract), declarations `services/access-control/rules.yaml` |
| 5 | **P2** | **Request payloads are stored three times and grow 20 GB/yr at today's rate.** `ai_request_payloads` is 718 MB for 4,392 rows (avg 164 KB/row, p99 573 KB): `request_body` carries the full tools array (135 KB avg, 68 tools) **and** `offered_tools` stores the same array again (identical in 3,183/4,392 rows; only 151 distinct tool sets in total); `request_body.messages` (149 KB avg, 45 messages) is re-sent every turn and additionally exploded into `ai_request_messages` (154,985 rows, 228 MB, of which only 9,084 are distinct per context — 94 % duplicate, 16 MB if deduplicated). | §4 | core `crates/entry/api/src/services/gateway/audit/open.rs:123-135` (history rows per request), core `crates/domain/ai/src/repository/ai_request_payloads.rs:60-105` (`upsert_payload`, `upsert_offered_tools`) |
| 6 | **P2** | **A single global row lock serialises every request insert and settlement.** `capture_feedback_source` (AFTER trigger on `ai_requests` and `plugin_usage_events`) and `enqueue_feedback_fact` each run `SELECT … FROM analytics_ingestion_producers WHERE producer='astound.feedback' FOR UPDATE`, so every gateway request open, every settlement (which also writes the ~230 KB payload in the same transaction) and every hook event queue behind one row. Its counter has already drifted: `pending_count`=11,164 vs 11,161 actual pending, and `expire_feedback_raw` refuses to run while `pending_count<>0`, so retention will never fire once the drain catches up. | §14, §34 | astound `extensions/web/schema/30_feedback_ingestion.sql:47,67,91,126`, `39_artifact_facts.sql:31,180`; retention gate `32_feedback_retention.sql:11-12` |
| 7 | **P2** | **`logs` is 89 % bridge polling access-log lines, and until this week each line was a per-row reporting-capture trigger.** 390,822 of 439,002 rows are `systemprompt_api::gateway` INFO lines; `GET /v1/bridge/profile` 176,436, `POST /v1/bridge/heartbeat` 103,934, `GET /v1/bridge/profile/usage` 61,371 — 89 log rows per AI request. Projection 8.6 GB/yr. A single `DELETE FROM logs WHERE timestamp < $1` at 09:05:01 UTC fanned out through `reporting_capture` into 77,797 `event_outbox` tombstones (17 MB) plus 77,797 `pg_notify` calls in one transaction. Core has an **uncommitted** `007_drop_logs_projection.sql` in the sibling worktree that drops this trigger. | §26, §27 | access log: core `crates/entry/api/src/routes/gateway/access_log.rs:120,192`; trigger: core `crates/infra/logging/schema/reporting_capture.sql`; the one-shot delete: core `crates/infra/logging/src/repository/operations/mutations.rs:132` (CLI `infra logs cleanup` / `jobs cleanup-logs`) |
| 8 | **P2** | **Feedback ingestion has never drained and the analytics projection is uninitialised** (known boot-loop; evidence): `analytics_projection_state.initialized=false, generation 0`; every `analytics_report_*` table empty; `feedback_ingestion_outbox` 11,161 pending, `attempts=0`, oldest `occurred_at` 2026-09-03, producer `checkpoint 0, last_drained_at NULL`; 20 configured jobs (`feedback_ingestion`, `feedback_facts_processing`, `conversation_rollup`, `oauth_cleanup`, `user_rate_limit_prune`, `access_expiry`, `quota_month_window`, …) have **no** `scheduled_jobs` row. `user_rate_limit_buckets` has 1,651 buckets older than a day that nothing prunes. | §14, §15, §28, §29 | core next 70bf034 / ba2ebfb (phased rebuild) — not re-reported |
| 9 | **P2** | **Hook governance decisions cannot be paired with hook events.** 2,069 `default_allow` decisions from Claude Code hooks (`Bash`, `Read`, `Edit`, …) match no `plugin_usage_events` row by `trace_id` (0/2,069), only by session; `governance_decisions.tool_use_id` is NULL on 16,163/16,163 rows although every `PostToolUse` event carries one (2,049/2,049). | §16 | decision writer: core `crates/infra/security/src/policy/audit.rs:186-200` (`tool_use_id` never set); hook handler: astound `extensions/web/admin/src/handlers/webhook/governance/handler/mod.rs:130` |
| 10 | **P3** | **Schema hygiene.** 87 redundant index pairs (narrower index is a prefix of a wider one on the same table; `ai_requests` has 22 indexes for 2.5 MB of heap, `mcp_tool_executions` 26); 37 orphan tables from removed extensions (`eval_*` 19, `knowledge_*` 12, `requirements_*` 6) plus 3 orphan `extension_migrations` rows for `evaluation`; duplicate CHECK constraints `event_outbox_actor_id_check` + `event_outbox_actor_id_nonempty`; `served_provider` NULL on 4,392/4,392 requests; `finish_reason` NULL on 4,287/4,287 completed; `upstream_latency_ms` NULL on 1,227 completed; `mcp_artifacts.data` (778 KB) duplicated in `artifact_payloads` for 284/284 rows; migration checksum `web/28` stored as 15 hex chars (`52b31491b1335ac`, un-padded) — string comparison against `{:016x}` output would flag drift. | §3, §6, §7, §11, §18 | see section notes |

Not a fault but worth knowing: 1,179 of 1,205 `users` rows are `anonymous.local` visitors (spikes of 481 on 09-08 and 383 on 09-12 — scanner traffic; 828 of them have no session at all) and `cleanup_anonymous_users` has run 20 times without removing them; 1,620 of 2,147 sessions are flagged `is_behavioral_bot`.

---

## Part A — Inventory and growth

### 1. Sizes (post-restore; dead tuples / autovacuum timestamps do not survive a plain-SQL dump)

Top by total size (`a1_sizes.sql`):

| table | rows | total | heap | toast | indexes |
|---|---:|---:|---:|---:|---:|
| ai_request_payloads | 4,392 | **718 MB** | 4 MB | 706 MB | 384 kB |
| ai_request_messages | 154,985 | 300 MB | 87 MB | 180 MB | 31 MB |
| logs | 439,002 | 281 MB | 146 MB | — | **134 MB** (13 indexes) |
| event_outbox | 77,797 | 45 MB | 32 MB | — | 13 MB |
| governance_decisions | 16,163 | 27 MB | 22 MB | — | 5 MB (14 indexes) |
| analytics_events | 17,743 | 14 MB | 11 MB | — | 2.6 MB |
| plugin_usage_events | 3,348 | 5.8 MB | | | |
| ai_requests | 4,392 | 5.4 MB | 2.5 MB | — | 2.7 MB (22 indexes) |
| feedback_ingestion_outbox | 11,161 | 4.5 MB | | | |
| mcp_tool_executions | 550 | 3.5 MB | 1 MB | 1.6 MB | 0.8 MB (26 indexes) |

169 of 237 tables are empty. Full row counts: `a1_counts.out`.

### 2. Growth (`a2_growth.sql`) — instance is 20 days old, so the 7-day rate is the honest one

| table | rows/day (7d) | now | 12-month projection at 7d rate |
|---|---:|---:|---:|
| ai_request_payloads | 325 | 718 MB | **≈20 GB** ⚠ |
| logs | 35,886 | 281 MB | ≈8.6 GB |
| ai_request_messages | 9,868 | 300 MB | ≈7.3 GB |
| event_outbox | (one-shot) | 45 MB | — |
| governance_decisions | 1,174 | 27 MB | ≈0.7 GB |
| analytics_events | 991 | 14 MB | ≈0.3 GB |

Only `ai_request_payloads` crosses the 10 GB line, and it does so for 18 users. `ai_requests` itself projects to 147 MB — the payload side-table is 130× the fact table.

### 3. Bloat and indexes

Bloat cannot be measured from a plain dump (the restore repacks every heap and index) — take `pgstattuple` on the live instance. `pg_stat_user_indexes.idx_scan` is likewise reset, so "unused" is not measurable here; **redundancy** is (`a3_indexes.sql`): 87 pairs where a non-unique, non-partial index's column list is a strict prefix of another index on the same table. Every one of the narrower indexes is dead weight on every insert. The heavy hitters:

- `logs` — 13 indexes, 134 MB of index for 146 MB of heap.
- `ai_requests` — 22 indexes; `idx_ai_requests_request_id` duplicates `ai_requests_request_id_key`; `idx_ai_requests_user_id` is a prefix of both `idx_ai_requests_user_created` and `idx_ai_requests_user_model`; `idx_ai_requests_session_id` ⊂ `idx_ai_requests_session_created`; `idx_ai_requests_provider` ⊂ `idx_ai_requests_provider_status`.
- `mcp_tool_executions` — 26 indexes; `idx_mcp_tool_executions_mcp_execution_id` duplicates the primary key; `…_tool_name` ⊂ `…_tool_status` and `…_tool_started`; `…_user_id` ⊂ `mcp_tool_executions_owner_id` and `…_user_created`.
- `ai_request_messages` — `idx_ai_request_messages_request_id` (1.3 MB) ⊂ `idx_ai_request_messages_sequence` (10 MB) ⊂ `ai_request_messages_request_id_sequence_number_key` (same columns, 10 MB) — three indexes on one key.
- `users` — `idx_users_email` duplicates `users_email_key`; `user_sessions` has six prefix pairs.

Full list of 87 in `a3_indexes.out`. Proposed fix: one migration per crate dropping the narrower member of each pair (declarative files too), starting with `logs`, `ai_requests`, `ai_request_messages`, `mcp_tool_executions`, `user_sessions`.

### 4. Column payloads (`a4_payloads.sql`, `pg_column_size` = on-disk after TOAST compression)

| column | rows | avg | p99 | max | total |
|---|---:|---:|---:|---:|---:|
| ai_request_payloads.request_body | 4,392 | 115 kB | 573 kB | 1000 kB | **490 MB** |
| ai_request_payloads.offered_tools | 4,392 | 59 kB | 126 kB | 129 kB | **184 MB** |
| ai_request_messages.content | 154,985 | 1.5 kB | 15 kB | 81 kB | **228 MB** |
| logs.metadata | 439,002 | 113 B | 193 B | 223 B | 46 MB |
| logs.message | 439,002 | 42 B | 115 B | 148 B | 17 MB |
| event_outbox.fact | 77,797 | 227 B | 227 B | 227 B | 17 MB |
| governance_decisions.evaluated_rules | 16,163 | 964 B | 1.5 kB | 1.6 kB | 15 MB |
| feedback_ingestion_evidence.evidence | 2,049 | 1.4 kB | 1.9 kB | 2 kB | 2.9 MB |
| mcp_tool_executions.output | 550 | 3.3 kB | 91 kB | 241 kB | 1.7 MB |
| ai_request_payloads.response_body | 4,392 | 600 B | 736 B | 2 kB | 826 kB |
| mcp_artifacts.data | 284 | 2.8 kB | 22 kB | 182 kB | 778 kB |
| artifact_payloads.body | 208 | 2.9 kB | 53 kB | 182 kB | 594 kB |
| ai_request_payloads.prepared_tools | 4,392 | — | — | — | never populated |

Uncompressed, `request_body` averages 230 kB (max 1,031 kB): `tools` 135 kB (67.7 tools avg), `messages` 149 kB (45.1 messages avg), `system` 34 kB. Duplication measured directly:

```
offered_tools = request_body->'tools' in 3,183 / 4,392 rows; 151 distinct tool sets; 141 distinct system prompts
ai_request_messages: 154,985 rows, 9,084 distinct (context, role, md5(content)) → 228 MB stored, 16 MB if deduplicated
avg 63.8 requests per context (max 311): each turn re-stores the whole history
```

Proposed fix (core): content-address `offered_tools` and `system` (sha256 → one row in a `gateway_tool_sets` table, FK from payload); store `request_body` without the `tools`/`system` members (they are reconstructable); write only the *new* messages of a turn into `ai_request_messages` (the previous turn's rows are the prefix — `open.rs:123-135` currently inserts every message of every request); or move `request_body` behind the existing `request_body_sha256` dedup column, which is populated but never used as a key.

### 5. Postgres configuration

Not present in a dump. Collect from the customer: `SHOW shared_buffers, work_mem, maintenance_work_mem, max_connections, autovacuum_vacuum_scale_factor, autovacuum_naptime; SELECT * FROM pg_settings WHERE source <> 'default';` and the machine memory. The relevant fact for the OOM is not Postgres memory but that the baseline rebuild materialises 439k `logs` rows in one transaction (§7 above, core 70bf034).

---

## Part B — Schema drift and migration state

### 6. `extension_migrations` vs the v0.58.0 trees (`b6_*`, xxh64 of the file bytes, as `systemprompt_extension::Migration::checksum` computes it — core `crates/shared/extension/src/migration.rs:95`)

208 rows / 15 extensions. Every migration file present at `v0.58.0` in both repos has a row with a **matching checksum** (0 drift in agent 12, ai 30, analytics 11, authz 18, content 3, events 6, files 1, logging 6, managed_resources 12, mcp 12, oauth 16, scheduler 1, users 16, web 56). Rows without a file:

| extension | version | name | note |
|---|---|---|---|
| web | 28, 31, 35, 38 | plans_soft_cap, budget_warning_kinds, default_organization, projects | tombstoned in the tree (`*.tombstone`) — expected |
| managed_resources | 12 | local_tree_roots_follow_current | tombstoned — expected |
| evaluation | 1, 2, 3 | reconcile_template_evals, result_repair_hint, execution_deadline | **extension no longer exists**; rows and its 19 `eval_*` tables are orphans |

`web/28` checksum is `52b31491b1335ac` (15 chars) — written before zero-padding; harmless today because the file is a tombstone, but a reminder that checksum comparison must be numeric or the padding must be repaired.

### 7. Live schema vs declarative files

Reproduced offline by diffing `pg_dump --schema-only` of the restored DB against a fresh install made by this tree's test harness (`schema_upgrade_tests_tpl_9792a6f8a29690f7`, 200 migrations, same declarative files). After removing column-order and PG17/PG18 deparse noise (`b7_schema.diff`):

| difference | production | fresh install | verdict |
|---|---|---|---|
| 37 tables `eval_*`, `knowledge_*`, `requirements_*` (+ their 25 indexes) | present, empty | absent | orphans of removed extensions; no migration drops them |
| `mcp_tool_executions UNIQUE (user_id, mcp_execution_id)` | unique **index** `mcp_tool_executions_owner_id` (mig web/057) | table **constraint** (core `mcp_tool_executions.sql:28`, commit 3f6afb2b2) | equivalent; `infra db doctor` will report it forever |
| `plugin_usage_events UNIQUE (user_id, id)` | unique index `plugin_usage_events_owner_id` (web/057) | constraint (`05_plugin_usage.sql:27`) | same |
| `idx_ai_safety_findings_blocked` | present (ai mig 019) | absent (not in `ai_safety_findings.sql`) | upgraded and fresh installs differ |
| `event_outbox_actor_id_nonempty` CHECK | present **twice** (`_check` + `_nonempty`) | once | duplicate constraint from an older migration |

Everything else (columns, types, NOT NULL, defaults, FKs, views, 239 functions) matches.

### 8. Triggers (`b8_triggers.sql`) — 53 non-internal triggers, all `FOR EACH ROW`

On the tables the brief names:

| table | rows | triggers |
|---|---:|---|
| logs | 439,002 | `reporting_capture` AFTER I/U/D → `sp_capture_reporting_change` (INSERT event_outbox + pg_notify **per log line**) |
| ai_request_messages | 154,985 | `reporting_capture` |
| analytics_events | 17,743 | `reporting_capture` |
| governance_decisions | 16,163 | `audit_event_notify_governance` (pg_notify), `governance_decisions_append_only` BEFORE UPDATE |
| ai_requests | 4,392 | `audit_event_notify_ai_requests` (pg_notify), `feedback_capture` (`capture_feedback_source`), `reporting_capture`, `request_scope_stamp_ai_requests` (INSERT ai_request_scopes) |
| plugin_usage_events | 3,348 | `audit_event_notify_plugin_usage`, `feedback_capture`, `ingestion_delivery` BEFORE, `ingestion_enqueue` AFTER, `ingestion_owner` BEFORE |
| user_sessions | 2,147 | `reporting_capture` |
| mcp_tool_executions | 550 | `reporting_capture` |
| conversation_analyses | 0 | `feedback_capture` (`capture_feedback_judgement`) |

All match the schema files at v0.58.0. Per-row triggers on tables > 100k rows: `logs` and `ai_request_messages` (both reporting-capture; the `logs` one is being dropped by the uncommitted core `007_drop_logs_projection.sql`).

### 9. Logical references without a foreign key (`b9_orphans.sql`) — 232 columns checked, 87 covered by a real FK, 15 with orphans

| table.column | → parent | non-null | orphans | share | explanation |
|---|---|---:|---:|---:|---|
| logs.session_id | user_sessions | 439,002 | 429,190 | 97.8 % | `'system'` (428,990) and `untracked_*` (200) sentinels, not ids |
| logs.user_id | users | 439,002 | 43,158 | 9.8 % | `'system'` sentinel |
| event_outbox.user_id | users | 77,797 | 16,641 | 21.4 % | `'system'` sentinel copied from logs |
| analytics_events.context_id | user_contexts | 17,743 | 17,743 | **100 %** | `derived_from_session` id space (finding 2) |
| governance_decisions.context_id | user_contexts | 16,163 | 11,706 | 72.4 % | finding 2 (authz stage) |
| governance_decisions.session_id | user_sessions | 16,163 | 2,069 | 12.8 % | Claude Code session uuid on hook decisions |
| mcp_tool_executions.context_id | user_contexts | 550 | 550 | **100 %** | finding 2 |
| mcp_artifacts.context_id | user_contexts | 284 | 284 | 100 % | finding 2 |
| logs.context_id | user_contexts | 3,245 | 3,245 | 100 % | finding 2 |
| plugin_usage_events.session_id | user_sessions | 3,348 | 3,348 | 100 % | Claude Code session uuid — a different id space by design |
| plugin_session_summaries.session_id, session_entity_links.session_id, mcp_sessions.session_id, mcp_proxy_identities.session_id, mcp_external_sessions.session_id | user_sessions | 259 / 112 / 63 / 18 / 6 | all | 100 % | other id spaces (Claude Code uuid, MCP session, `r13-*`) |

The context-id split, proven on one trace (`b9c_gov_ctx.out`):

```
decisions joined to their request on trace_id: 13,241
  same context_id: 4,457   different: 8,784   (same session_id and user_id: 13,241)
  by policy: authz 4,392/4,392 differ, authz_rule_based 4,392/4,392 differ,
             default_allow 0/3,513, secret_scan 0/871, quota 0/73
  the 8,784 mismatched decisions use 105 distinct context ids = exactly the 105 distinct
  ai_requests.session_id values (1:1) → ContextId::derived_from_session(bridge session)
  52 of those 105 also appear as mcp_tool_executions.context_id
```

So the authz stage (core `policy/audit.rs:182-185`, fallback when `audit.context_id` is None) and the session middleware (`middleware/session/context.rs:77,99,138`) derive from the bridge session, while the gateway derives from the caller's Claude Code session (`gateway/messages/extract/mod.rs:230`). `refresh_conversation_facts` papers over it with `OR d.session_id = a.client_session_id` for governance, but has no such fallback that works for executions (§12). Proposed fix: thread the request's `ContextId` into the authz `PolicyAudit` (it is known at that point — the other three policies record it) and have MCP execution recording take the context from the `x-context-id`/client-session the bridge already sends, falling back to `derived_from_client_session`; then a one-off backfill `UPDATE governance_decisions g SET context_id = r.context_id FROM ai_requests r WHERE r.trace_id = g.trace_id AND g.context_id <> r.context_id`.

---

## Part C — Integrity: counters and derived data

### 10. `user_sessions` counters vs sources (`c10_sessions.sql`)

| counter | sessions mismatched | stored Σ | actual Σ | example session ids |
|---|---:|---:|---:|---|
| ai_request_count | 105 / 2,147 (every session that has requests) | 0 | 4,392 | sess_4c7617e1-…, sess_57e49bd9-…, sess_e584da01-… |
| total_tokens_used | 102 | 0 | 312,091,073 | same |
| total_ai_cost_microdollars | 102 | 0 | 279,641,627 | same |
| error_count | 24 | 0 | 95 | sess_4c7617e1-…, sess_e584da01-…, sess_bbb45b66-… |
| task_count / message_count | 0 | 0 | 0 | no A2A tasks on this instance — consistent |
| request_count | — | 17,743 | = analytics_events (17,743) | this one is maintained (HTTP hits) |

Every gateway request is `request_kind ∈ {turn, probe}` from `client_kind ∈ {opencode 3,644, other 732, unknown 16}` and settles through `settle.rs`, which updates `ai_requests` only. The session leaderboards in core `crates/domain/analytics/src/repository/core_stats/leaderboards.rs` and `sessions/queries.rs` read these columns, so any "top sessions by cost/tokens" surface shows zeros. Fix: call `increment_ai_usage` (or a set-based recompute) from the settlement transaction, and a one-off backfill `UPDATE user_sessions s SET ai_request_count=r.n, total_tokens_used=r.tok, total_ai_cost_microdollars=r.cost, error_count=r.err FROM (SELECT session_id, count(*) n, sum(tokens_used) tok, sum(cost_microdollars) cost, count(*) FILTER (WHERE status='failed') err FROM ai_requests GROUP BY 1) r WHERE r.session_id=s.session_id`.

### 11. Tokens and cost (`c11_tokens.sql`)

`tokens_used = input + output + cache_read + cache_creation` (reasoning excluded — `CanonicalUsage::billable_total`, core `crates/shared/models/src/wire/canonical/usage.rs:72`) holds on **4,287/4,287 completed** rows (1,844 with reasoning tokens, none where reasoning > output). 95 failed, 7 rejected and 3 pending rows have NULL tokens — correct.

Uncosted: **0** of 4,287 completed rows with tokens > 0. Recomputing from `services/ai/providers.yaml` prices (input/output + cache read 10 % / creation 125 %) reproduces the stored cost to the cent for `claude-sonnet-5` ($115.68), `claude-opus-5` ($87.74), `claude-sonnet-4-6` ($14.39). Spend by model/kind: haiku turns $35.01, sonnet-5 turns $111.66, opus-5 turns $85.00, probes $9.40 total (1,338 probes = 30 % of requests, 3 % of spend). Models served that are **not** in `providers.yaml` (discovery-published): `gemini-3.1-pro-preview`, `gemini-3.6-flash`, `gemini-3.7-flash`, `gemini-3.1-flash-lite` — 27 requests, $0.75, all costed.

Columns never populated: `served_provider` (0/4,392), `finish_reason` (0/4,287 completed), `upstream_latency_ms` NULL on 1,227 completed. The first two were added in 0.58.0 (`ai` migrations 029/030) and the gateway does not fill them yet.

### 12. `conversation_facts` vs `refresh_conversation_facts(NULL)` (rolled back, `c12_facts.sql`)

421 rows in, 421 rows out, 0 new/gone contexts; **0 differences** in request_count, turn_count, cost, input/output tokens, first_at, last_at, error_count, tool_calls_intended/executed, gov_allow, hook_event_count, skill_invocations, group_id, safety_findings. `refreshed_at ≥ last request` on all 421. Table hash identical after rollback (`unchanged_after_rollback = t`). The rollup is deterministic and current — but deterministically wrong on one column:

```
Σ tool_calls_intended = 2,901   Σ tool_calls_executed = 0   mcp_tool_executions = 550
tool_activity rows with an execution: 550
  execution_context_id ∈ conversation_facts.context_id : 0
  execution_trace_id   ∈ conversation_facts.client_session_id : 0
  execution_trace_id   ∈ ai_requests.trace_id : 0
mcp_tool_executions: context_id ∈ ai_requests.context_id 0; trace_id ∈ ai_requests.trace_id 0;
  session_id ∈ ai_requests.session_id 549; mcp_execution_id ∈ ai_request_tool_calls 0
```

Every conversation page therefore shows "0 tools executed" against a real 550. The only key that links an execution to a conversation today is the bridge `session_id` (549/550), and that maps to a session, not a conversation.

### 13. Daily rollups (`c13_rollups.sql`)

`plugin_usage_daily` vs `plugin_usage_events` grouped by (date, user, plugin, event_type, tool_name): 514 keys, 0 rollup-only, 0 events-only, 0 count differences (3,348 = 3,348). `admin_usage_daily_rollups` vs `ai_requests` by (user, day): 71 keys; the only differences are today's two rows (hourly job last ran 07:05, dump 10:25) and one zero row for a user with no requests that day. Both rollups are sound.

### 14. Feedback ingestion state (`c14_18.sql`, `c14b_feedback.out`)

```
analytics_ingestion_producers: astound.feedback  pending_count 11,164  checkpoint 0  last_drained_at NULL
feedback_ingestion_outbox: 11,161 pending (resource_association 4,133, request 2,894, invocation 2,067, assessment 2,067),
  attempts = 0 on all, last_error NULL on all, created_at all 2026-09-22, occurred_at back to 2026-09-03
feedback_ingestion_backfill: plugin_usage_events done=f, ai_requests done=f
analytics_fact_checkpoints / _changes / _consumers / _deltas / snapshot_state: all empty
```

The queue was populated by the upgrade's backfill this morning and no worker has taken a single lease (`attempts=0`). `pending_count` (11,164) is already 3 above the true pending count (11,161): the counter is incremented in `enqueue_feedback_fact` when `was_pending IS DISTINCT FROM true` and decremented on drain, but rows removed by FK cascade (event/request deletion) never decrement it. Because `expire_feedback_raw` raises `'Feedback retention waits for pending producers'` whenever `pending_count <> 0` (`32_feedback_retention.sql:11-12`), a drifted counter blocks retention permanently. Fix: derive `pending_count` from `count(*) WHERE pending` (or a statement-level trigger that recounts), and drop the singleton `FOR UPDATE` (finding 6).

### 15. Analytics projection (known boot loop — evidence only)

`analytics_projection_state = {generation 0, cutoff_revision 0, initialized f, rebuilt_at NULL}`; `analytics_projection_revisions` empty; all nine `analytics_report_*` tables 0 rows; `event_outbox`: 77,797 rows, all `channel=reporting / consumer=analytics_reporting`, all pending, all `logs` DELETE tombstones (revisions 155,403–233,199 — the earlier 155k facts were consumed before the upgrade), single `created_at` 09:05:01.924, oldest pending age 1 h 19 m at dump time.

### 16. Governance vs requests (`c14_18.sql`)

Requests without any decision: **0 / 4,392** on every day. Decisions without a request: 2,922 of 16,163 — 853 match an MCP execution by trace, 2,069 match nothing but a hook session (Claude Code built-ins: Bash 833, Read 300, Edit 78, Write 77, Skill 44, …). Per policy: default_allow 5,821, authz 4,695, authz_rule_based 4,695, secret_scan 879 (871 warn on requests), quota 73.

Scopes: 4,392/4,392 requests have an `ai_request_scopes` row; 0 point at a missing group or project; 844 have NULL project (users whose default has none). Distribution: india-devs/core 3,531, unassigned 493, business-users 285, india-devs/∅ 66, europe-devs/commerce 9, unassigned/core 8. Scope rows agree with `user_scope_defaults` on 4,392/4,392.

### 17. Safety findings and tool calls

`ai_safety_findings`: 3,451, 0 orphans, 0 blocked (warn mode per `services/gateway/policies.yaml`); pii_email 2,702, pii_phone 616, pii_credit_card 105, secret 28. Excerpts are redacted (email/card findings store none; phone/secret store ≤ 25 chars that match no email/card pattern). `ai_request_tool_calls`: 2,901 rows, `mcp_execution_id` set on **0**, `ai_tool_call_id` set on 2,901; `mcp_tool_executions`: 550, `ai_tool_call_id` set on **0**, referenced by no tool call. Replay of the `claim_unclaimed_intent` predicate at each execution's `started_at` (same session, tool name or `%__<tool>` suffix, intent ≤ 120 s old, unclaimed): **138 of 303 in-process executions had a claimable intent**, 131 more had one arrive within 120 s *after* start. The claim never lands; there is no "Intent claim failed" log (MCP server processes do not log to the DB, so absence is not proof). This needs a single instrumented run; candidates are `ctx.session_id()` in the MCP server process not being the bridge `sess_…` id the intent rows carry, or `claim_intent` racing the settlement transaction that writes the intent 5 ms after `completed_at`.

### 18. Artifacts

284 `mcp_artifacts`, 208 `artifact_payloads`, 0 `artifact_parts`/`task_artifacts` (no A2A). 0 artifacts without payload, 0 unreferenced payloads, 0 `ref_count` errors, 0 without an execution. All 284 keep the body inline in `data` (778 kB) **and** in `artifact_payloads` (1,389 kB) — the narrow-waist migration left the inline copy.

### 19. Access control vs `rules.yaml` (`c19`, files `rules_v058.txt`, `ac_db_entities.txt`)

DB: 63 entities / 159 rules, all `source='yaml'` on rules but entities carry legacy sources (`ingestion:access_control_config` 44, `marketplace:*` 8, `access-control/groups.yaml` 6, `access-control/projects.yaml` 3, `slack:T00000000000` 1, `access-control/salesforce.yaml` 1). v0.58.0 `rules.yaml`: 20 entities.

- In DB, not in code (43): `agent/admin_console`, `slack_workspace/T00000000000`, `marketplace/{astound-cowork, astound-sales-uk, enterprise-demo, systemprompt-admin}`, `plugin/{astound-admin, astound-executive-insights, astound-sales-uk}`, `mcp_server/{github, knowledge-bank, requirements-workflow, salesforce}`, 24 `skill/*` (admin_*, uk_sales_*, executive_*, inspect_*, manage_*), 11 generated `gateway_route/*` (expected: the file declares `gateway_route/*`).
- In code, not in DB (8): **`marketplace/astound-salesforce`, `mcp_server/{salesforce-crm-dev,-uk,-us, salesforce-headless-dev,-uk,-us, google_workspace}`**.
- `sync_state`: 0 rows. `access_control_rule_validity`: 0 rows (nothing expiring).

The console's drift banner should be showing this; either it is not, or nobody has acted on it. Either way a customer on 0.58.0 has no entitlement rows for the marketplace 0.58.0 ships.

### 20. Groups, projects, defaults (`c20_21.out`)

5 groups (india-devs 10 members, business-users 2, europe-devs 2, uk 0, unassigned system 0), 4 projects, 13 project memberships. 0 duplicate memberships, 0 members pointing at missing users, 26 `user_scope_defaults` all `source=auto` with valid group/project. 1,191 of 1,205 users are in no group, but 1,179 of those are `anonymous.local` rows; of the 18 users with AI requests, 5 are in no group (they land in `unassigned`, 493 requests). One default is `unassigned` + project `core` — a project default without its group.

### 21. `managed_*`

All managed-resources tables are empty (0 resources, revisions, publications, receipts, grants, conflicts) — this instance consumes kits from the tree, not the managed inventory. `ingestion_repairs`: 3,644 rows, all `native_metadata_v1`, all repaired 2026-09-22 = every `opencode` request (3,644/3,644) needed its `client_session_id` recovered from metadata at upgrade; 748 requests (`client_kind` other/unknown) have no `client_session_id` at all and can never join to hook events.

---

## Part D — Temporal and state anomalies

### 22. `ai_requests` (`d22_29.out`)

0 completed_before_created, 0 latency NULL on completed, 0 future, 0 duplicate `request_id`, 0 accounting failures, max latency 241.8 s. **3 stuck `pending`** (2026-09-16 12:40, 09-18 21:21, 09-18 23:04 — no error_message; the client disconnected before headers and nothing sweeps them to `failed`). Status per day is in the output; failure shapes (top): `stream ended without stop event` 34, Anthropic `credit balance is too low` 22, Gemini `model … no longer available to new users` 15 (2.5-flash / -lite / -pro still routed after retirement), `prompt is too long` 4, `Gateway API key secret 'vertex_maas' not configured` 4, Vertex `functionDeclaration parameters` 400s 8, missing `openai` / `cerebras` secrets 3, secret-scan denials 3.

### 23. `user_sessions`

14 open of 2,147; 7 `expires_at` in the past but not ended; 0 open-but-idle > 7 d; 1,263 ended rows with NULL `duration_seconds`; 1,620 `is_behavioral_bot`, 517 `is_scanner`, 169 `is_ai_crawler`, 0 `is_bot`; one user opened 121 sessions on 2026-09-09. Sources: bridge 1,070, web 803, oauth 272, api 2.

### 24. `mcp_tool_executions`

0 incomplete > 1 h; success 523 (p50 166 ms, p99 2.0 s, max 9.6 s), failed 27. `server_name`: systemprompt 257, **atlassian 228, knowledge-bank 46, github 19** — three servers that no longer exist in `services/mcp/*.yaml` (which now lists atlassian, google_workspace, six salesforce-*, systemprompt; `knowledge-bank`, `github` are gone). Historical rows, but the MCP servers page derives its catalog from them.

### 25. Tasks — 0 agent_tasks, 0 steps. Consistent with "no A2A agents".

### 26. `logs` (`d26_logs.out`)

439,002 rows (INFO 437,068, WARN 1,333, ERROR 601; no DEBUG), 14 days retained. Module `systemprompt_api::gateway` = 392,599 (89.4 %). Top INFO shapes are all bridge polling (`/v1/bridge/profile` 176,436, `/v1/bridge/heartbeat` 103,934, `/v1/bridge/profile/usage` 61,371) — the per-request access log at core `routes/gateway/access_log.rs:120` writes one DB row per HTTP hit including polls, and each of those fired the `logs` reporting trigger. ERROR shapes: `GET /v1/bridge/latest -> 502` **577** (2026-09-11 → 22, `phase: headers`, 669 ms — the bridge update endpoint proxying a failing upstream), `MCP backend error response` 9 (knowledge-bank 404s), `POST /v1/messages -> 502` 7, `Proxy server error` 4, `Service … is not running (token accessor …)` 4. WARN: `/v1/bridge/stream -> 429` 362 and 401 32, `/v1/bridge/latest -> 404` 381, `JWT validation failed` 11, Vertex retirement notices 21. 0 log rows belong to deleted users (there are none).

### 27. Outboxes (`d27_outbox.out`)

`event_outbox`: 77,797 pending / 0 processed, one consumer, one `created_at`, all logs tombstones (§15). `feedback_ingestion_outbox`: 11,161 pending / 0 processed (§14). `ingestion_outbox`, `managed_distribution_outbox`: empty.

### 28. Scheduled jobs (`d28_jobs.out`) vs `services/scheduler/config.yaml`

20 rows, all `enabled`, all `last_status=success`, all `next_run NULL`. Running hourly/daily as configured: behavioral_analysis, cleanup_* (anonymous 20 runs, inactive 476), content_analytics_aggregation 1,862, database_cleanup 30, ghost_session_cleanup 2,794, publish_pipeline 1,881, service_registry_gc 26,646, usage_anomaly / usage_daily_rollup 466, vertex_discovery 8, plugin_usage_retention 19. **Configured but never registered (no row, no log line):** feedback_ingestion, feedback_snapshot_processing, feedback_facts_processing, managed_inventory_refresh, oauth_cleanup, user_rate_limit_prune, scope_defaults_recompute, access_expiry, quota_month_window, conversation_judge, conversation_rollup, otlp_export (+ the publish sub-steps, which run inside publish_pipeline). In DB but not in config: `knowledge_drive_sync` (last run 2026-09-11, extension removed). `conversation_facts` is nevertheless current, so `conversation_rollup` runs through another path (the request-time refresh) — but `oauth_cleanup` and `user_rate_limit_prune` have no other path (§29).

### 29. Anomalies and buckets (`d29.out`)

`usage_anomalies`: 68 fired (cost 33 at avg 26.7× baseline, requests 33 at 15.7×, errors 2); `anomaly_thresholds` empty (defaults). `ai_quota_buckets`: 152 hourly user buckets, Σ requests 4,384 vs 4,392 actual, Σ cost = actual to the microdollar; 82 windows closed > 7 d ago never removed; **no daily/monthly window rows** although `quota_month_window` is configured. `user_rate_limit_buckets`: 1,780 `mcp` buckets, **1,651 older than a day**, nothing prunes them (job never registered).

---

## Part E — Privacy and retention

### 30. Deleted users

0 users with `status='deleted'` (1,204 active, 1 inactive). Checked all 95 user-keyed tables for rows whose `user_id` no longer exists in `users`: **0** (excluding the `'system'` sentinel). No privacy residue today; note that 1,179 anonymous visitor users are retained with `cleanup_anonymous_users` running daily.

### 31. Retention

Oldest rows: users/sessions/contexts 2026-09-02, ai_requests/payloads/messages 09-03, logs 09-08 (14 d — 30 d configured, so something ran a 14-day cut manually; §27), governance_decisions 09-03. No window has elapsed yet, so the question is which tables *have* a mechanism:

| table | mechanism | status |
|---|---|---|
| logs | `database_cleanup` (`retention.logs_days`, default 30, 5,000-row batches) + ad-hoc CLI | working |
| ai_requests + cascades | `expire_feedback_raw` (90 d) | **blocked** while `pending_count<>0` (§14) — will never run on this instance as-is |
| ai_request_messages | `database_cleanup` (`ai.history.retention_days`) | working |
| analytics_events | `database_cleanup` (90 d) | working |
| mcp_tool_executions | `database_cleanup` (365 d) | working |
| event_outbox processed | `database_cleanup` (7 d) | working — but pending rows are never aged out |
| ai_request_payloads | cascade from ai_requests only | tied to the blocked path above |
| governance_decisions, ai_safety_findings, secret_audit_log, ai_gateway_thought_signatures (own cleanup job exists), oauth_jti_revocations, webauthn_challenges, bridge_exchange_codes (141 rows), dev_login_codes, id_jag_replay, mcp_connector_oauth_states, user_rate_limit_buckets, ai_quota_buckets | none / job not registered | no retention |

### 32. Secrets in clear (`e32_secrets.sql`, counts only)

| table.column | `sk-…` | `Bearer …` | PEM | JWT |
|---|---:|---:|---:|---:|
| ai_request_messages.content | 0 | 64 | 0 | 32 |
| ai_request_payloads.request_body | 0 | 34 | 0 | 32 |
| **mcp_proxy_identities.auth_token** | 0 | 0 | 0 | **18** |
| ai_request_tool_calls.tool_input | 0 | 2 | 0 | 0 |
| plugin_usage_events.metadata | 0 | 1 | 0 | 0 |
| feedback_ingestion_evidence.evidence | 0 | 1 | 0 | 0 |

The first two are user prompts (the secret scanner is in warn mode and flagged 28 response-side and 3 request-side secrets). `mcp_proxy_identities.auth_token` stores the caller's bearer JWT in clear by design (core `crates/domain/mcp/src/repository/proxy_identity.rs:51`), 2 of 18 already expired and not cleaned. Proposed: encrypt at rest with the instance key or keep it only in the process-local session cache.

---

## Part F — Query scaling

### 33. EXPLAIN (ANALYZE, BUFFERS) of the heaviest repository queries (`explain.py`, `f33_*.out`)

Method: every `SELECT`/`WITH` in the `.sqlx` offline caches (astound root: 1,579 queries; core `domain/ai`, `domain/analytics`, `infra/security`, `infra/logging`, `app/runtime`) that touches a large table, ranked by tables touched × length, parameters substituted with the busiest user/session/context, a 2026-09-01…23 window and `LIMIT 50`; 75 executed. Container: `shared_buffers=128MB`, `work_mem=4MB`.

At 4,392 requests nothing exceeds 500 ms and nothing sorts to disk. Everything that will scale badly scans `logs`:

| query | ms | seq scans | note |
|---|---:|---|---|
| trace list `WITH tool_traces …` (core `crates/app/runtime/src/trace/list_queries.rs`) | 235 | **logs 439,002**, ai_requests 4,392, mcp_tool_executions ×3 | `infra logs trace list` reads every log row with a trace_id, unions, then correlated sub-selects per trace; linear in logs |
| `SELECT trace_id, MIN(timestamp) FROM logs GROUP BY trace_id ORDER BY 2 DESC LIMIT 20` | 63 | logs (17,574 blocks read) | same page |
| `SELECT module, COUNT(*) FROM logs WHERE timestamp >= … GROUP BY module` | 68 | logs (57k blocks, parallel) | logs overview |
| `SELECT DISTINCT user_id FROM logs` (orphan-log pass of `database_cleanup`) | 54 | logs | runs nightly |
| conversations list (`1cef0790…`, `conversation_requests` + users + sessions) | 18 | users 26 | fine |
| MCP servers/tools page (`a39bb34d…`) | 22 | mcp_tool_executions 550 | fine now; add `(server_name, created_at)` when > 100k |
| governance page (`3d55636a…`) | 24 | — | index-driven |
| logs page filtered by level + time + ILIKE | 27 | — | index on (timestamp) then filter; ILIKE cannot use it — acceptable at 50 rows |
| costs by user 30 d, conversation_facts list, request audit payload fetch | ≤ 5 | — | fine |

With logs at the projected 8.6 GB the three logs scans become multi-second and the trace list query unusable. Fixes: the uncommitted core `007_drop_logs_projection.sql` plus (a) stop persisting access-log lines for `/v1/bridge/profile`, `/v1/bridge/heartbeat`, `/v1/bridge/profile/usage` (or sample them) at `access_log.rs:120`, (b) a partial index `ON logs (trace_id, timestamp) WHERE trace_id IS NOT NULL AND trace_id <> 'system'` for the trace list, and (c) drop the 8 prefix-duplicate indexes on `logs` so the remaining ones fit in cache.

### 34. Statements per AI request (from the trigger inventory + code)

For one `turn` request at this instance's averages (45 messages, 0.66 tool calls, 3.5 decisions, 0.8 safety findings, 89 access-log lines):

| phase | statements | rows written | triggers fired |
|---|---:|---:|---|
| governance chain (authz, authz_rule_based, default_allow, quota, secret_scan) | 3.5 INSERT governance_decisions | 3.5 | 3.5 × pg_notify |
| open: INSERT ai_requests | 1 | 1 | `audit_event_notify` (notify), `feedback_capture` (**FOR UPDATE** on producer row, +1 outbox row, +1 counter UPDATE, +1 `enqueue_feedback_session`), `reporting_capture` (+1 outbox, notify), `request_scope_stamp` (+1 ai_request_scopes) |
| open: history rows | 45 INSERT ai_request_messages | 45 | 45 × `reporting_capture` (45 outbox rows + 45 notifies) |
| settle (one tx): UPDATE ai_requests, INSERT payload, UPSERT offered_tools, INSERT tool calls, INSERT assistant message | ≈5 | ≈3 + ~230 KB TOAST | `feedback_capture` again (FOR UPDATE, 2 more outbox upserts for new+old), `reporting_capture` again, `feedback_capture_tool` per tool call, `reporting_capture` per message |
| safety findings | 0.8 INSERT | 0.8 | — |
| quota bucket | 1 UPSERT ai_quota_buckets | 1 | — |
| session | 0 (should be 1 — finding 1) | | |
| access log | 1 INSERT logs (+ 88 more for the polls around it) | 89 | 89 × `reporting_capture` (until the trigger is dropped) |

≈ **60 statements and ≈150 trigger invocations per request today**, of which ≈140 exist only to feed a projection that is not initialised, and two of which take the same global row lock. After the logs trigger drop: ≈60 statements, ≈60 trigger firings. The two structural fixes are the message-delta write (45 → ~2 rows per turn) and removing the producer-row lock.

---

## Finding → code path → proposed fix → migration?

| # | finding | code path | proposed fix | migration |
|---|---|---|---|---|
| 1 | session AI counters never written | core `domain/users/src/sessions/mutations/mod.rs:221` unused by gateway; `entry/api/src/services/gateway/audit/journal/settle.rs:75` | call `increment_ai_usage` (tokens, cost, +error on failure) inside the settlement tx; backfill UPDATE in §10 | data backfill (one-off) |
| 2 | two context-id spaces | core `shared/identifiers/src/context.rs:59-69`; `infra/security/src/policy/audit.rs:182-185`; `entry/api/src/services/middleware/session/context.rs:77,99,138`; MCP execution recording | pass the gateway `ContextId` into `PolicyAudit`; derive MCP/analytics context from client session; backfill `governance_decisions.context_id` from `ai_requests` by trace | data backfill |
| 3 | tool calls ↔ executions never pair | core `domain/mcp/src/repository/tool_usage/intent.rs:17-74`, `domain/mcp/src/tool/mod.rs:146-176` | instrument `with_claimed_intent` (log session id + predicate outcome) on one production run; likely fix is the session id the MCP server sees vs `ai_requests.session_id`; then backfill by the replay predicate in §17 | data backfill |
| 4 | access-control DB = stale entitlement set, `sync_state` empty | astound `extensions/web/admin/src/repositories/sync/*`, `services/access-control/rules.yaml` | make the boot compare surface loudly when code declares entities the DB lacks (or auto-insert-only for *new* entities); operator action now: `/admin/sync` → Overwrite from code | no |
| 5 | payload triple storage, 20 GB/yr | core `entry/api/src/services/gateway/audit/open.rs:123-135`; `domain/ai/src/repository/ai_request_payloads.rs:60-105` | content-address tool sets + system prompt; write only the turn's new messages; strip `tools`/`system` from `request_body` | schema (new table) + backfill/dedup |
| 6 | global `FOR UPDATE` on `analytics_ingestion_producers` per request; `pending_count` drift blocks retention | astound `extensions/web/schema/30_feedback_ingestion.sql:47,67,91,126`, `39_artifact_facts.sql:31,180`, `32_feedback_retention.sql:11` | drop the lock (outbox rows are keyed by md5 and upserted — no lock needed); compute `pending_count` from the outbox; retention gate on `count(*) WHERE pending` | schema (function bodies) |
| 7 | logs = polling access log + per-row reporting trigger; 77,797 tombstones from one DELETE | core `entry/api/src/routes/gateway/access_log.rs:120,192`; `infra/logging/schema/reporting_capture.sql`; `infra/logging/src/repository/operations/mutations.rs:132` | commit `007_drop_logs_projection.sql`; exclude `/v1/bridge/{profile,heartbeat,profile/usage}` from DB access logging; batch the CLI cleanup like `database_cleanup` does; purge the 77,797 pending tombstones on upgrade | migration (already drafted, uncommitted) |
| 8 | feedback/analytics pipelines never started; jobs unregistered; rate-limit buckets unpruned | core next 70bf034/ba2ebfb; scheduler registration of `web` jobs | verify the 20 missing jobs register after 0.58.1; add `user_rate_limit_prune`/`oauth_cleanup` evidence to `/admin/configuration` | no |
| 9 | hook decisions lack `tool_use_id`, no trace link to events | core `infra/security/src/policy/audit.rs:186-200`; astound `handlers/webhook/governance/handler/mod.rs:130` | record `tool_use_id` (present in the hook payload) on the decision; join events↔decisions on it | data (nullable column exists) |
| 10a | 87 prefix-duplicate indexes | per-crate `schema/*.sql` | drop narrower member of each pair | migration per crate |
| 10b | 37 orphan tables + 3 orphan `evaluation` migration rows | removed extensions | a `web` migration `DROP TABLE IF EXISTS …` + `DELETE FROM extension_migrations WHERE extension_id='evaluation'` | migration |
| 10c | fresh vs upgraded constraint form (UNIQUE constraint vs unique index); `idx_ai_safety_findings_blocked` only via migration; duplicate `event_outbox` CHECK | core `domain/mcp/schema/mcp_tool_executions.sql:28`, astound `05_plugin_usage.sql:27`, core `domain/ai/schema/migrations/019` | add the index to `ai_safety_findings.sql`; drop `event_outbox_actor_id_nonempty`; teach doctor that a unique index satisfies a UNIQUE constraint | migration |
| 10d | `served_provider`, `finish_reason` never set; `upstream_latency_ms` NULL on 29 % | gateway settlement `settle.rs` | populate from the adapter result | no |
| 10e | `mcp_artifacts.data` duplicated in `artifact_payloads` | core `domain/mcp` narrow-waist | null out `data` once `payload_sha256` is set | data |
| 10f | 3 requests stuck `pending` since 09-16/18 | gateway open without settlement | sweep `pending` older than N minutes to `failed('client disconnected')` in `database_cleanup` | no |
| 10g | `mcp_proxy_identities.auth_token` clear JWT; 2 expired rows | core `domain/mcp/src/repository/proxy_identity.rs:51` | encrypt or hash+cache; include in `mcp_session_cleanup` | schema |
| 10h | Gemini 2.5 models still routable after provider retirement (15 failures) | `services/ai/providers.yaml`, vertex_discovery | honour retirement dates in routing, not only in warnings | no |

---

## Appendix — queries

All queries and their raw outputs are in `docs/ops/db-analysis-2026-09-22/` and were run with `psql -X -q -v ON_ERROR_STOP=0` against `sp_prod_analysis`; results are in the matching `*.out` files. The report reproduces every number it cites from those outputs. Files: `a1_sizes.sql`, `a1_counts.sql`, `a2_growth.sql`, `a3_indexes.sql`, `a4_payloads.sql` (+ `a4b_dup`), `b6_*` (python xxh64 comparison), `b7_schema.diff`, `b8_triggers.sql`, `b9_orphans.sql` (+ `b9b`, `b9c`), `c10_sessions.sql`, `c11_tokens.sql`, `c12_facts.sql`, `c13_rollups.sql`, `c14_18.sql` (+ `c14b`, `c14_18b`), `c20_21`, `d22_29.sql`, `d26_logs`, `d27_outbox`, `d28_jobs`, `d29`, `e30_32.sql`, `e32_secrets.sql`, `explain.py`, `f33_logs.sql`.

Key one-liners, for re-running against the live instance:

```sql
-- 1 session counters
SELECT count(*) FILTER (WHERE s.ai_request_count IS DISTINCT FROM r.n) mismatched, sum(r.n) actual
FROM user_sessions s LEFT JOIN (SELECT session_id, count(*) n FROM ai_requests GROUP BY 1) r USING (session_id);

-- 2 context-id split
SELECT g.policy, count(*) FILTER (WHERE g.context_id <> r.context_id) diff_ctx, count(*) n
FROM governance_decisions g JOIN ai_requests r ON r.trace_id = g.trace_id GROUP BY 1;

-- 3 pairing
SELECT (SELECT count(mcp_execution_id) FROM ai_request_tool_calls) paired_calls,
       (SELECT count(ai_tool_call_id) FROM mcp_tool_executions) paired_execs,
       (SELECT sum(tool_calls_executed) FROM conversation_facts) executed_in_facts;

-- 4 access control
SELECT entity_type||'/'||entity_id FROM access_control_entities ORDER BY 1;  -- diff against rules.yaml
SELECT count(*) FROM sync_state;

-- 5 payload duplication
SELECT count(*) FILTER (WHERE offered_tools = request_body->'tools') same, count(DISTINCT md5(offered_tools::text)) tool_sets FROM ai_request_payloads;
SELECT count(*) rows, count(DISTINCT (r.context_id, m.role, md5(m.content))) distinct_rows FROM ai_request_messages m JOIN ai_requests r ON r.id=m.request_id;

-- 6 producer lock + counter drift
SELECT pending_count, (SELECT count(*) FROM feedback_ingestion_outbox WHERE pending) actual FROM analytics_ingestion_producers;

-- 7 logs composition + outbox tombstones
SELECT module = 'systemprompt_api::gateway' gw, count(*) FROM logs GROUP BY 1;
SELECT fact->'data'->>'source', (fact->'data'->>'deleted')::bool, count(*), min(created_at), max(created_at) FROM event_outbox WHERE processed_at IS NULL GROUP BY 1,2;

-- 8 pipeline state
SELECT * FROM analytics_projection_state; SELECT attempts, count(*) FROM feedback_ingestion_outbox GROUP BY 1;
SELECT job_name FROM scheduled_jobs ORDER BY 1;  -- diff against services/scheduler/config.yaml

-- 9 hook decisions
SELECT count(*) FILTER (WHERE tool_use_id IS NULL) no_tool_use_id, count(*) FROM governance_decisions;

-- 10 redundant indexes: see a3_indexes.sql (prefix match on pg_index.indkey)
```
