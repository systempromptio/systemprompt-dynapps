# Retention and backups — operator runbook

Written 2026-09-22 after a self-hosted 0.58.0 could not boot: the analytics
baseline rebuild walked a history nothing had ever pruned and the Postgres
backend ran out of memory. Two things prevent that class of incident: the
tables are kept bounded by a job that actually deletes, and there is a
restorable copy of the database from before any upgrade.

## What the instance keeps, and for how long

One job, `database_cleanup`, deletes rows past their window from the
tables that grow with traffic. The windows live in the profile
(`.systemprompt/profiles/<name>/profile.yaml`), all optional:

```yaml
retention:
  logs_days: 30                 # server log lines
  analytics_events_days: 90     # traffic and behaviour events
  ai_request_messages_days: 30  # stored prompt/completion bodies; omit to
                                # follow services/ai/config.yaml history.retention_days
  mcp_tool_executions_days: 365 # tool call records
  outbox_processed_days: 7      # processed durable events
  ai_request_payload_raw_days: 7   # raw request/response bodies set to NULL;
                                   # excerpts, hashes and sizes stay
  governance_decisions_days: 180   # per-call policy decisions
```

`ai_quota_buckets` older than 62 days (the longest quota window plus its
carry-forward) are pruned by the same job with no setting.

A profile that says nothing gets exactly those defaults. Unknown keys are an
error at load.

The job deletes in batches of 5 000 rows, one statement each, so no long lock
is ever held. A run
stops at 15 minutes and continues the next night; the console marks that with
a `+` after the count. It runs nightly at 04:00 (`services/scheduler/config.yaml`)
and **only deletes with `enforce: true`** on its entry — without it the job
logs what it would remove and removes nothing, which is how a self-hosted
instance grew a 500 000-row `logs` table.

Where to see it: **`/admin/lifecycle`** ("Data lifecycle" in the Platform
nav) is the one page for this — the windows in force with the profile key
behind each one, what the last run deleted, then the size and growth of every
managed table, the archives with download links, and the last health check.
`/admin/configuration` shows the same window table beside the rest of the
instance's configuration. Both are read-only: the profile is the source, so a
change means editing `profile.yaml` and restarting. From the CLI:

```bash
systemprompt infra jobs run database_cleanup      # run it now
systemprompt infra jobs list                      # last run / next run / status
```

Requests themselves (`ai_requests`) are not in this job: the Dynapps
`plugin_usage_retention` job expires them after 90 days together with the
hook events (`expire_raw_evidence`, `extensions/web/schema/32_raw_retention.sql`),
and the stored message bodies cascade with them. The messages window above
only trims bodies sooner than the request rows.

## Measurement, archives and the health check

Three Dynapps jobs are the other half of the policy (`extensions/web/jobs/src/retention/`):

| job | when | what |
|---|---|---|
| `retention_daily_report` | 04:30 daily | size, dead tuples and oldest row of every managed table into `retention_runs`; warns when a table grew more than 25 % week on week or an outbox has more than 100 000 pending rows |
| `retention_export_weekly` | Sunday 02:00 | the previous ISO week of the raw tables (`ai_requests`, messages, payloads, tool calls, safety findings, governance decisions, tool executions, sessions, hook events, logs, analytics and engagement events) to `storage/exports/weekly/<yyyy>-W<ww>/<table>.jsonl.gz` + `manifest.json` |
| `retention_export_monthly` | 1st, 02:30 | last month's rollups (`admin_usage_daily_rollups`, `plugin_usage_daily`, `conversation_facts`, `conversation_skill_facts`, anomalies, content metrics, `retention_runs`) to `storage/exports/monthly/<yyyy>-<mm>/`, then the health check into `retention_health_reports`, then `VACUUM (ANALYZE)` on the managed tables |

Every archive file is `COPY (SELECT row_to_json(t) …) TO STDOUT`, gzipped,
with a SHA-256 in the manifest — restorable with `COPY … FROM`. Weekly
archives are pruned after `keep_weeks` (26) once the monthly archive covering
that week exists; monthly archives are kept (`keep_months: 0`). The rollups
themselves are never expired — `expire_raw_evidence` does not touch them — so
the console, the Versions pages included, keeps its history past the 90-day
raw window.

Where to see it: `/admin/lifecycle` lists the latest measurement per table
with its growth, every archive with a download link, and the last health
check's ranked findings. From the CLI:

```bash
systemprompt infra jobs run retention_daily_report
systemprompt infra jobs run retention_export_weekly -p weeks_back=4   # backfill
systemprompt infra jobs run retention_export_monthly
```

## Dead schema

A crate that is deleted must take its tables with it. `infra db doctor`
exits non-zero on any live table no registered extension declares and on
any `extension_migrations` ledger for an extension that no longer exists;
boot logs the same as a warning after every schema install. In this repo
`scripts/check-dropped-schema.sh` (a `lint-gates` gate) refuses a deleted
`schema/*.sql` whose tables no migration drops. Migration 095 is the drop
for everything found on production in the 2026-09-22 audit.

## Backups

Take a full logical dump before every upgrade and on a rolling schedule. The
database URL is `database_url` in the profile's `secrets.json`.

```bash
# before an upgrade, and nightly from cron
pg_dump "$DATABASE_URL" --format=custom --no-owner --no-acl \
  --file="systemprompt-$(date +%Y%m%d-%H%M).dump"

# keep 14 daily + 8 weekly; anything older is deleted
find /backups -name 'systemprompt-*.dump' -mtime +14 -delete
```

Restore into a fresh database (never over a running one):

```bash
createdb systemprompt_restore
pg_restore --dbname=systemprompt_restore --no-owner --no-acl systemprompt-<stamp>.dump
```

Point a throwaway profile at the restored database to rehearse an upgrade:
run the new binary's `systemprompt infra db migrate`, boot it, and watch
`analytics projection status` move from `rebuild_source` to `initialized:
true`. The baseline rebuild runs in the background after the server binds,
in committed pages of 10 000 rows, so a large history no longer blocks
startup — but it still reads every retained row, which is why the windows
above matter.

## If the analytics baseline is rebuilding on every boot (0.58.0)

Symptom: `Failed to initialize application context … out of memory` after
several minutes, every boot and every CLI command. Fixed from 0.58.1. Until
the upgrade: mark the baseline initialized by hand and restart, then let the
new release backfill it:

```sql
UPDATE analytics_projection_state SET initialized = TRUE WHERE singleton;
```

```bash
systemprompt analytics projection rebuild   # after upgrading
```

## Open: there is no local reset recipe

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

The runbook below stands in for it. A `just db-reset-activity` recipe that
owns this list is the tooling answer, once the list is agreed.

### Runbook — reset local activity data (verified 2026-09-17)

Runs in one transaction against the per-clone Docker Postgres; keeps schema,
seeds, content, marketplaces, sync state, the analytics registration rows and
the `admin` user. Re-materialises access control afterwards through the sync
plane rather than the bootstrap job, and applies it **twice** if the
`gateway_route` catalog was empty at the first pass (the glob expands over the existing catalog).

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
