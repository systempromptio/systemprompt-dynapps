# Tech debt — data lifecycle and the artifact chain

Recorded 2026-09-17 from a local exercise: wipe every activity row from the
local instance (keep the `admin` user and the seeds), then drive three real
Claude Code flows through the gateway and read them back on the console.
Everything below was observed, not inferred. Items are ordered by how much
they undermine the end-to-end story the console is meant to tell.

## 1. In-process MCP tool calls never join the ledger

Every tool an internal MCP server executes for a Claude Code session lands as
**two** artifacts that cannot be reconciled:

| vantage | `tool_name` | `session_id` | `ai_tool_call_id` | ledger state |
|---|---|---|---|---|
| `in_process` (the real result) | `admin_report` | `sess_8af4…` (MCP session) | *(empty)* | `unattested` |
| `hook_claude_code` | `mcp__plugin_astound-super-admin_systemprompt__admin_report` | `d46d06e1…` (Claude Code session) | `toolu_017n…` | `executed` |

`ArtifactIngest` resolves identity by `_meta` execution id → minted id →
`tool_use_id` via `mcp_tool_executions.ai_tool_call_id` → fingerprint
(session + tool + sha256 within 30 s). For a plugin-routed MCP call none of
the four can match: the in-process context has no `tool_use_id`, the two
sessions are different identifiers, the tool names differ by the
`mcp__plugin_<marketplace>_<server>__` prefix, and the bodies differ (one is
the payload, the other the client's error text). Net effect on
`/admin/artifacts`: the row with the data is `unattested` and the row with the
provenance is `executed` — the inverse of the design. Builtin tools
(`Write`/`Edit`/`Skill`) join correctly because only the hook sees them.

Fix direction: carry the Claude Code `tool_use_id` and session through the
proxy into the in-process `RequestContext` (the bridge already injects
identity headers), and normalise the plugin prefix off the hook tool name
before fingerprinting.

## 2. `systemprompt` MCP tools fail Claude Code's output-schema validation

`admin_report`, `usage_by_user` and `systemprompt` all came back to the client
as *"Structured content does not match the tool's output schema: data must
have required property 'report', 'title', 'checked_at', 'period', 'complete',
'sources', 'metrics', 'tables'"* while the server-side artifact for the same
call is `is_error=false` and contains those keys under `structured_content`.
The declared `outputSchema` and the envelope actually sent disagree; Claude
Code 2.1.x validates and treats the call as failed, so the model gives up on
the admin tools. Also `services/artifacts/admin-ai-usage/config.yaml` binds
`mcp_tools: [mcp__systemprompt__admin_report]`, which is not the name a
plugin-installed server presents (`mcp__plugin_astound-super-admin_systemprompt__admin_report`).

## 3. Access-control sync silently declares nothing for `gateway_route/*`

`build_declared_from_doc` expands the glob over the `gateway_route` rows
already in `access_control_entities` (`repositories/access_control/declared.rs:203`).
When that catalog is empty — as it is after any truncate, and before
`governance_bootstrap` step 1 has run — the glob matches nothing, the plane
writes **zero route rules**, and the drift report says **0** because declared
and stored agree on nothing. The only signal is a `tracing::warn!`. The
symptom is a 403 on every `/v1/messages`: *"gateway_route:claude-star-4203d1:
not assigned … (no allow rule; default_included = false)"*. Re-applying the
plane once the catalog exists fixed it (60 → 115 rules). The plane should
refuse to apply, or report drift, when a glob expands to nothing.

## 4. There is no local reset; a raw wipe has to know the schema's secrets

CLAUDE.md is correct that no reset recipe exists. Doing it by hand needed:

- the FK graph, because `managed_*` and `analytics_*` reference `users` with
  `NO ACTION`, so a test user still referenced there blocks the delete;
- knowledge that `tool_call_ledger`, `conversation_*`, `reporting_source_*`
  and `v_*` are views (TRUNCATE rejects them);
- the user-privacy protocol (`begin_user_privacy()` / `finish_user_privacy()`
  and the `reporting_capture` trigger on `users`) — bypassing it is harmless
  today (the trigger still emits the delete facts into `event_outbox`) but
  that is luck, not contract;
- the analytics pipeline singletons (`analytics_fact_consumers`,
  `analytics_fact_checkpoints`, `analytics_ingestion_producers`,
  `analytics_projection_state`, `analytics_snapshot_state`) which are
  registration state and must survive a wipe.

A runbook is at the end of this file. A `just db-reset-activity` recipe that
owns this list is the tooling answer, once the list is agreed.

## 5. Identity columns without foreign keys (orphan sources)

44 tables carry a `user_id` with no FK to `users`; 34 carry a `session_id`
with no FK to `user_sessions`; 17 a `context_id`; 13 a `trace_id`. The
activity tables that matter for the console are all in that set:
`ai_requests`, `ai_request_scopes`, `mcp_artifacts`, `mcp_tool_executions`,
`governance_decisions`, `plugin_usage_events`, `plugin_session_summaries`,
`logs`, `session_*`, `conversation_analyses`, `evaluation_*`,
`user_settings`, `user_encryption_keys`, `files`.

Two things make an FK impossible as the columns stand:

- `session_id` is overloaded — gateway session, MCP session (`sess_…`) and
  the client's Claude Code session share one column name and never one
  parent table (`ai_requests.client_session_id` is the bridge).
- sentinel principals live in `users`-typed columns: `logs.user_id = 'system'`,
  `event_outbox.user_id ∈ {'system','reporting-source'}`.

Post-wipe orphan scan found nothing else; the schema is clean *today* only
because the wipe was exhaustive. `admin users delete` will leave every one of
those rows behind.

## 6. FK shape oddities

- `user_manual_roles`, `group_members`, `project_members` each hold two FKs
  to `users`: `user_id` (CASCADE) and `granted_by` (SET NULL). Deleting an
  admin rewrites grant history rather than preserving who granted.
- `user_sessions.user_id`, `mcp_sessions.user_id` are SET NULL: deleting a
  user keeps their sessions as anonymous rows (and `admin users delete`
  deletes `user_sessions` explicitly to compensate).
- `ai_requests.session_id`, `user_contexts.session_id`,
  `analytics_events.session_id` are SET NULL to `user_sessions`, so a session
  purge leaves requests with no session but a live `client_session_id`.

## 7. Tooling and environment frictions met on the way

- `infra db tables` reports `pg_class.reltuples` as "Rows": it showed
  `users = 0` while 46 rows existed and `extension_migrations = 16` for 186.
  Label it as an estimate or count.
- `infra db query` cannot render `regclass` or `"char"` columns (empty cells);
  `::text` casts are needed for any catalog query.
- `just bridge-build` refuses whenever `../systemprompt-core` is ahead of
  `bridge/CORE_REF`, which on `next` is the normal state between pins.
  A local client build needed `cd bridge && cargo build --release` directly.
- `deploy/clean-client/bootstrap.sh` runs `astound-bridge install --apply
  --apply-schedule` under `set -e`; without systemd the bridge returns non-zero
  on "partially completed" and the bootstrap dies before the proxy and sync
  ever run. `--apply` alone exits 0.
- `~/.local/bin/docker` on this host is a `docker.exe` shim that predates the
  Docker Desktop WSL integration now at `/usr/bin/docker`; bind mounts through
  the shim arrive as empty directories, which is why the first `just claude`
  saw no bridge and no bootstrap script. Remove the shim.
- Skill ids are `snake_case` in `services/skills/<id>/` but Claude Code exposes
  them dashed (`/who_am_i` → "not available", `/who-am-i` works).
- The stale-binary state of a shared clone: `.hbs` templates mid-edit
  referenced `kpis.scored` and `observability_url`, so
  `/admin/analysis/conversations` and `/admin/sync` 500 until a rebuild;
  `services/evaluation/config.yaml` gained `report:` and every
  `session_evaluation` tick and `governance_bootstrap` run fails on the old
  binary. Neither is a schema issue, but both hide real ones.

## Runbook — reset local activity data (verified 2026-09-17)

Runs in one transaction against the per-clone Docker Postgres; keeps schema,
seeds, content, marketplaces, sync state, the analytics registration rows and
the `admin` user. Re-materialises access control afterwards through the sync
plane rather than the bootstrap job, and applies it **twice** if the
`gateway_route` catalog was empty at the first pass (see §3).

```sql
\set ON_ERROR_STOP on
BEGIN;
CREATE TEMP TABLE keep(t text) ON COMMIT DROP;
INSERT INTO keep VALUES
 ('extension_migrations'),('services'),('service_owned_ids'),('service_sources'),('sync_state'),
 ('scheduled_jobs'),('gateway_routes'),('ai_gateway_policies'),('governance_chain_settings'),('anomaly_thresholds'),
 ('access_control_entities'),('access_control_rules'),('access_control_rule_validity'),
 ('files'),('content_files'),('markdown_categories'),('markdown_content'),('markdown_content_enrichment'),('markdown_fts'),
 ('marketplace_versions'),('campaign_links'),('funnels'),('funnel_steps'),
 ('groups'),('group_ad_mappings'),('group_members'),('projects'),('project_ad_mappings'),('project_members'),
 ('users'),('user_encryption_keys'),('user_settings'),('user_scope_defaults'),('user_manual_roles'),('user_profile_ext'),
 ('federated_identities'),('salesforce_user_identities'),('webauthn_credentials'),('user_device_certs'),('user_device_cert_validity'),
 ('bridge_user_host_prefs'),('bridge_user_host_model_prefs'),('device_app_links'),
 ('oauth_clients'),('oauth_client_contacts'),('oauth_client_grant_types'),('oauth_client_redirect_uris'),('oauth_client_response_types'),('oauth_client_scopes'),
 ('mcp_connector_accounts'),('mcp_connector_credentials'),
 ('analytics_fact_consumers'),('analytics_fact_checkpoints'),('analytics_ingestion_producers'),('analytics_projection_state'),('analytics_snapshot_state'),('analytics_snapshot_dimensions');
SELECT string_agg(quote_ident(tablename), ', ' ORDER BY tablename) AS wipe_list
FROM pg_tables WHERE schemaname='public' AND tablename NOT IN (SELECT t FROM keep) AND tablename NOT LIKE 'managed\_%' \gset
TRUNCATE :wipe_list CASCADE;
DELETE FROM users WHERE name <> 'admin';
COMMIT;
```

```bash
docker exec -i <pg-container> psql -U systemprompt -d systemprompt < wipe.sql
# then, with an admin session token:
curl -X POST -H "Authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"mode":"overwrite"}' http://localhost:8080/api/public/admin/sync/planes/access_control/apply
# confirm every gateway_route entity has rules; if not, POST once more
```

Attribution needs the user in a group and a project (`POST
/api/public/admin/groups/{g}/members`, `…/projects/{p}/members` with
`{"user_id": …}`); each call recomputes `user_scope_defaults`, and the
`request_scope_stamp_ai_requests` trigger stamps every request from then on.
