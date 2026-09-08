# 2026-09-01 — Knowledge bank as a remote, shared-index MCP server with per-user Atlassian credentials

**Status:** decided (Ed, 2026-09-01). Supersedes, in
`2026-09-01-knowledge-bank-ingestion-in-process.md`, the scheduled/CLI triggers of
decision 2, decisions 3 and 4 (source-configured grants, service-account indexer)
and the same-day addendum. Decision 1 there (one database, pgvector) is amended:
the database is now dedicated and required, never inert.

## Context

Phase 0 was built for a host that owns everything: an instance-level Atlassian
service account, file trees on the box, a cron, an `ingest` CLI, grant translation
through a group map, and an "inert without pgvector" mode for the Fly deploy. The
deployment that is actually wanted is the final architecture minus federated
identity: users reach the knowledge bank from Claude Code and Cowork at
`https://astound.systemprompt.io/api/v1/mcp/knowledge-bank`, and nobody runs
systemprompt locally. That changes who reads Atlassian, where the index lives, and
what "who may read this document" means.

Two facts settled the shape. The multitenant Fly cluster `systemprompt-db-prod`
(`flyio/postgres-flex 17.7`, 13 databases) has no `vector` extension and its tenant
role cannot `CREATE DATABASE`, so the index cannot live there. And core fails the
whole platform boot when an internal MCP child dies at startup
(`lifecycle/startup.rs`, `reconciliation.rs`), so a missing requirement has to fail
with an unmistakable message rather than degrade quietly.

## Decisions

1. **The knowledge bank is a remote server.** Users connect over HTTP with a PAT or
   bridge JWT; the `/api/v1/mcp/{server}` proxy forwards every header except `host`
   and `x-mcp-proxy-auth` to the child, and rmcp exposes the request's
   `http::request::Parts` on every `tools/call`, which is how per-user headers reach
   the handlers.

2. **One shared index, on a dedicated pgvector Postgres, as a hard requirement.**
   The instance owns the index; the Bedrock key is the instance's. On Fly the index
   is a new Postgres app (`sp-a2f658d8bc5f-knowledge`, `fra`), named by the
   `knowledge_database_url` secret and reached over 6PN. Where the secret is absent
   (local, compose, CI — all pgvector images) the child uses the instance pool. Two
   pools in the child: the instance pool for auth, audit and `users`; the knowledge
   pool for the index. Without pgvector — or without `bedrock_api_key` — the child
   exits non-zero, naming the host and this repo's `docs/install/knowledge-db.md`,
   and the platform boot fails. `StubRetriever` and the inert mode are deleted.

3. **Only the caller's own identity travels with the request; grants are per
   user.** *(Superseded on 2026-09-08 by
   `2026-09-08-knowledge-bank-reads-through-connected-mcp.md`: the header
   credentials are gone; reads go through the caller's connected Atlassian
   account via the gateway. The grant model below is unchanged.)* Each
   caller passed their own Atlassian e-mail and API token as `x-atlassian-email` /
   `x-atlassian-api-token` (optionally `x-atlassian-base-url`) on the MCP
   connection. `sync_knowledge` — open to every signed-in user — reads Confluence
   and Jira **as that caller** and grants every document it writes to
   `user:<caller>`. A page several users can read is embedded once (the content
   hash and the embedding cache see to it) and granted to each of them; on a
   complete fetch the caller's grants on documents that disappeared upstream are
   removed, and a document left with zero grants is deleted. Sync state is per
   `(source_type, user_id)`, so cursors, errors and counters are the caller's own
   and `list_knowledge_sources` reports the caller's counts and *their* last sync.
   Search is unchanged: JWT → `Principal` → grant join inside the candidate
   queries. No group mapping, no service account, no per-user database.

4. **Dropped:** file sources and the PDF/DOCX parsers, `storage/knowledge` and its
   compose volume, the cron schedule, the `ingest` subcommand, and the
   grant-translation machinery (`GrantResolver`, `group_map`, `unmapped_groups`,
   per-source `grants:`). Confluence restriction reads go too — Atlassian enforces
   them for the caller — and Jira security-level issues are indexed normally,
   because the caller could read them.

5. **Manageability is part of the contract.** The admin-only `prune_knowledge`
   tool drops grants of users with no sync in `stale_days` (default 90) or whose
   `users` row is gone, deletes orphaned documents, and prunes the date and
   embedding caches, with a `dry_run`. `knowledge_index_stats` (now admin-only)
   reports syncing users, grants, cache rows and database size. `just
   knowledge-db-{create,provision,psql,report,prune}` and
   `deploy/fly/knowledge-db/` do the same from psql against the Fly database.

## Federated identity

This is the final architecture minus federated identity. What fedid changes later
is only the *source* of the per-user Atlassian credential — an OAuth grant minted
through `external_auth` instead of an API-token header. The shared index, the
per-user grants, the sync semantics and the tool surface stay as they are.

## Consequences

- `services/knowledge-bank/config.yaml` shrinks to `atlassian.base_url`,
  `knowledge_database.secret`, `ingestion.doc_concurrency` and the two Atlassian
  sources; `services/mcp/knowledge-bank.yaml` documents the remote contract; the
  `project_context` skill gains a "Connect" section with the `.mcp.json` headers.
- The `atlassian_base_url` / `atlassian_email` / `atlassian_api_token` secrets are
  gone. The only instance secrets are `bedrock_api_key` and, on Fly,
  `knowledge_database_url`.
- `knowledge_sync_state` is replaced by `knowledge_user_sync_state`;
  `knowledge_document_grants` gains `granted_at`. Documents from the earlier model
  that still carry `role:user` grants remain visible to everyone until pruned or
  re-synced.
- The Fly knowledge database must exist before `just deploy-next`; provisioning it
  and verifying pgvector on the `postgres-flex` image are blockers B5 and B8 in
  `blockers.md`.
