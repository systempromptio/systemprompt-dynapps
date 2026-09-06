# The knowledge database

The knowledge bank (`extensions/mcp/knowledge-bank/`) keeps its index — documents,
chunks with embeddings, per-user grants, sync state and two cost caches — in a
Postgres database with the `vector` extension. This page says where that database
is in each deployment, how to provision it, and how to keep it tidy.

## Requirements, and what happens without them

One thing is required at boot, and the knowledge-bank child exits non-zero
without it. Core treats a dead internal MCP child as a failed boot, so the
**whole platform refuses to start**. There is no inert mode.

| Requirement | Where | Failure message |
|---|---|---|
| pgvector on the index database | the database named by the `knowledge_database_url` secret, or the instance database when that secret is absent | `knowledge-bank: database <host> has no pgvector; it is required (see docs/install/knowledge-db.md)` |
| `bedrock_api_key` in the profile secret store — **only when** `services/knowledge-bank/config.yaml` sets `model.provider: bedrock` | the instance's Bedrock long-term API key (bearer, eu-central-1) | `model.provider is bedrock but the profile secret store has no bedrock_api_key` |

The fix is always the same: give the database the extension (`CREATE EXTENSION
vector;` as a superuser, on an image that ships it) or point the secret at a
database that has it, then start again. The binary applies its own tables
idempotently on every start; nothing else is created by hand.

## The model provider

The shipped configuration selects the `local` provider: feature-hashed n-gram
embeddings and a BM25 reranker computed in-process, deterministic, free, and
needing no credential, so every install — laptop, CI, a customer's first node —
boots with the real tools over real pgvector. It has no chat model, so the three
index-time LLM tasks (LLM chunking of transcripts, contextual enrichment, LLM
dating) take their deterministic fallbacks: script chunking, no enrichment
prefix, frontmatter-or-mtime dating. Swapping to Bedrock is configuration only;
the procedure and what each provider gives you are in
`docs/install/knowledge-model-provider.md`. The boot log's `knowledge-bank
backend ready` line names the provider and embedding model in use.

The image also ships three `files` categories that point at fixtures under
`services/knowledge-bank/fixtures/` — a fictional engagement's functional specs,
runbooks, Salesforce schema, storefront conventions, two sprints of Jira-shaped
tickets and five client briefs — so `sync_knowledge` on a fresh install indexes
something and `search_knowledge` answers with citations before any Atlassian
account is linked. `just knowledge-seed` does that sync against a running local
stack. Replace or delete the categories for a customer deployment.

## Where the database is

**Local development.** Leave `knowledge_database_url` unset. `just db-up` runs
`pgvector/pgvector:pg18`, and the child uses the instance pool. `just prepare`
applies the crate schema to that database so the `.sqlx` offline cache can be
regenerated.

**docker-compose.** The bundled Postgres is the same pgvector image, so the default
(no secret) works. To keep the index apart from the application data, run a
second pgvector Postgres and set the secret to it:

```bash
docker compose exec app systemprompt admin config secret set knowledge_database_url \
  'postgres://knowledge:<pw>@<host>:5432/knowledge'
```

A remote Postgres without the extension fails the boot as described above.

**Fly (production / staging).** The shared `systemprompt-db-prod` cluster has no
`vector` extension and the tenant role cannot create databases, so the index lives
on a dedicated Fly Postgres app, `sp-a2f658d8bc5f-knowledge`. Provisioning is
three recipes and one secret — see `deploy/fly/knowledge-db/README.md`:

```bash
just knowledge-db-create
KNOWLEDGE_DB_PASSWORD='…' just knowledge-db-provision
systemprompt admin config secret set knowledge_database_url \
  'postgres://knowledge:<pw>@sp-a2f658d8bc5f-knowledge.flycast:5432/knowledge' --profile production
just deploy-next
```

## What is in it

| Table | Holds |
|---|---|
| `knowledge_documents` | one row per page or issue: source category, canonical `uri`, date and its provenance, `content_hash` |
| `knowledge_chunks` | the embedded text, `lines` pointer, chunk position; cascades from the document |
| `knowledge_document_grants` | `(doc_id, subject_kind, subject_value, granted_at)`; only `user:<id>` rows are written now — one per user who synced the document |
| `knowledge_user_sync_state` | per `(source_type, user_id)`: incremental cursor, last run, last error, counters |
| `knowledge_index_state` | `version` / `last_built` stamps |
| `knowledge_date_cache` | LLM-derived dates frozen per content hash |
| `knowledge_embedding_cache` | embeddings keyed by model and text hash, so a page synced by a second user costs no model call |

A document with no grants is reachable by nobody and is deleted at the next
complete sync or prune. Search joins the caller's grants inside the candidate
queries, so nothing leaks between users.

## Keeping it tidy

- **`prune_knowledge {stale_days?, dry_run?}`** (admin-only MCP tool) — drops the
  grants of users who have not synced in `stale_days` (default 90) or whose account
  no longer exists, deletes documents left with zero grants, removes date-cache rows
  for missing documents and embedding-cache rows older than `stale_days` that no
  chunk references. `dry_run: true` returns the counts without deleting.
- **`knowledge_index_stats`** (admin-only) — documents, chunks, grants, distinct
  syncing users, cache rows, database size, last build.
- **`just knowledge-db-report`** — the same numbers plus the per-user sync table,
  straight from psql on the Fly database.
- **`just knowledge-db-prune`** — the prune SQL against the Fly database, dry run by
  default (`DRY_RUN=0` commits). The one thing it cannot do is check the instance's
  `users` table; the tool does.
- **`just knowledge-db-psql`** — interactive psql.

The index is a derived cache: any user can rebuild their share of it with
`sync_knowledge {rebuild: true}`, and the embedding cache makes that cheap.
