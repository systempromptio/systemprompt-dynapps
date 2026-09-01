# 2026-09-01 — Knowledge-bank ingestion runs inside the MCP binary; one pgvector database

**Status:** decided (Ed, 2026-09-01). Supersedes decision #3 ("ingestion: scheduler
jobs") in `docs/tickets/req-047-knowledge-bank-mcp.md`.

> **Superseded in part, same day** — see
> `2026-09-01-knowledge-bank-remote-shared-index.md`. Still standing: ingestion runs
> inside the MCP binary (decision 2, minus its cron and `ingest` CLI triggers), and
> pgvector is the store. No longer true: the inert-without-pgvector mode (now a hard
> boot failure on a dedicated database), source-configured grants and group mapping
> (decision 3), the service-account indexer (decision 4), and the addendum. Users
> now sync with their own Atlassian credentials and are granted what they synced.

## Context

Phase 0 of the AI Kit migration (`docs/migration/project-context/`) re-platforms the
Node `project-context` RAG engine onto this instance. The serving path already existed
in `extensions/mcp/knowledge-bank/`; ingestion did not, and its schema was deliberately
left uninstalled because the Fly staging deploy's managed Postgres has no pgvector.
Two questions had to be settled before ingestion could be built: where the index lives,
and where the write path runs.

## Decisions

1. **One database.** Container deployments (docker-compose, the Oracle VM installs, local
   development) run the whole instance database on `pgvector/pgvector:pg18` — already the
   image in `docker-compose.yml`, the local compose, and CI. There is no separate
   knowledge database. Fly staging keeps systemprompt's managed Postgres and is out of
   scope for the index: the knowledge-bank binary probes `pg_available_extensions` at
   boot, applies its own idempotent schema where `vector` exists, and otherwise serves
   the honest `BACKEND_NOT_CONFIGURED` stub. Nothing about the knowledge bank can fail
   the platform's boot any more, so the schema no longer needs to hide in
   `migrations-pending/`.

2. **Ingestion runs in the knowledge-bank process.** `scripts/lint-layers.sh` places
   `extensions/web/jobs` in the *app* tier and `extensions/mcp/*` in the *entry* tier;
   a core `Job` in the jobs crate depending on this crate is an upward edge and fails the
   gate, and the only shared-tier crates (`extensions/{web,mcp}/shared`) are linked by
   every server. Rather than split the crate or bloat `mcp-shared`, the write path is a
   library inside the crate with three triggers over one code path: a scheduled tokio
   task (`ingestion.schedule`, off by default), the `systemprompt-mcp-knowledge-bank
   ingest [--rebuild] [--source <id>]` one-shot subcommand for operators and CI, and the
   admin-only `sync_knowledge` MCP tool. Serving reads the database and never waits on
   ingestion (NFR-SELF-1); a run is single-flight; per-document failures are counted,
   not fatal.

3. **Unrestricted content is granted to `role: user`** — everyone signed in — with
   per-space/project overrides in `services/knowledge-bank/config.yaml`. Confluence page
   restrictions narrow further through the linked-account table, e-mail, and the
   configured `atlassian.group_map`; anything that cannot be translated is dropped, so a
   restricted page whose restrictions all fail to map is reachable by nobody.

4. **The indexer is a service identity.** Confluence and Jira are read with the indexing
   account's e-mail and API token from the profile secret store
   (`atlassian_base_url`, `atlassian_email`, `atlassian_api_token`). The per-user OAuth
   grants behind the live Jira/Confluence MCP servers are for reading as the user, not
   for building a shared index — an index built with one user's permissions is that
   user's index.

## Consequences

- `extensions/web/schema/19_knowledge_bank.sql` and `migrations-pending/034_*` are gone;
  the schema is `extensions/mcp/knowledge-bank/schema/knowledge_bank.sql`, applied by
  `src/schema.rs`.
- `services/mcp/knowledge-bank.yaml` ships `enabled: true` and the server grant widens
  to `role: user`; the per-document grant join is what makes that safe.
- Fly staging reports the knowledge bank as not configured, by design. If it is ever
  wanted there, the pattern is the Odoo companion in `systemprompt-internal`
  (`deploy/fly/odoo/`): a companion app with its own role and database on the shared
  managed Postgres, deployed independently.

## Addendum, same day (POC scope)

- **Group restrictions:** `atlassian.unmapped_groups: everyone` — a Confluence page
  restricted to a group not in `group_map` receives the source's default grants for
  the pilot. Switch to `drop` before real restricted content is indexed.
- **Live Jira/Confluence servers:** deferred; no OAuth app for the POC. Opening a cited
  source relies on individuals' own API tokens through the `atlassian` skill.
- **Index location:** local/container Docker Postgres only (pgvector image); never the
  production managed database, where the binary is inert by design.
