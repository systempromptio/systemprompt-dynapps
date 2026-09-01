# REQ-047 / REQ-048 — Knowledge-bank MCP server (project-context, Phase 0)

Status: **built end to end as a remote, shared-index server; Fly database
pending.** The serving path, the per-user ingestion pipeline, the per-document
access control (REQ-052), the schema, the prune tooling, and the agent-side
skill are implemented and tested. Users reach it from Claude Code / Cowork at
`https://astound.systemprompt.io/api/v1/mcp/knowledge-bank` with their own
Atlassian credentials as headers; what remains is provisioning the dedicated
pgvector Postgres on Fly (`blockers.md` B5/B8) and the corpus users can read
(B2). Requirements and the as-is baseline: `docs/migration/project-context/`.

The backend is a **migration of the Node.js "project-context" system** at
`github.com/Astound-Digital/sfcc-next-cursor/.project/project-context`, with two
parts that are not a migration at all: that system is a single-user local tool
with no caller and no ACLs, so REQ-052 is net-new; and it refreshed its index
from a Cursor slash command, so the per-user `sync_knowledge` and the admin
`prune_knowledge` tools are ours.

## Where things are

| Concern | Location |
|---|---|
| Binary, tools, server | `extensions/mcp/knowledge-bank/src/{main,tools,server/*}.rs` |
| Per-user Atlassian credentials from headers | `src/downstream.rs` |
| Schema (crate-owned, applied at boot after a pgvector probe that fails the boot) | `extensions/mcp/knowledge-bank/schema/knowledge_bank.sql`, `src/schema.rs` |
| Model provider seam + Bedrock | `src/model/` |
| Retrieval (hybrid, RRF, rerank, grant join) | `src/retrieval/` |
| Ingestion (connectors, chunking, dating, enrichment, caches, per-user grants) | `src/ingest/` |
| Prune + stats (stale users, orphans, caches) | `src/maintenance.rs` |
| Configuration | `services/knowledge-bank/config.yaml` (crate-owned; validated at load) |
| Server registration + ACL | `services/mcp/knowledge-bank.yaml`, `services/access-control/roles.yaml` |
| Agent contract | `services/skills/project_context/SKILL.md`, plugin `services/plugins/project-context` |
| The knowledge database on Fly | `deploy/fly/knowledge-db/`, `docs/install/knowledge-db.md`, `just knowledge-db-*` |
| Tests | `tests/unit/mcp/src/knowledge_*.rs`, `tests/integration/mcp/src/knowledge_*.rs` |

## Bedrock access, verified live 2026-08-28

The key is a long-term Bedrock API key used as `Authorization: Bearer <key>`,
not sig-v4 — the crate uses `reqwest` and needs no `aws-sdk-*` dependency.
Identity: `BedrockAPIKey-0o2w` in AWS account `266359797998`. It is the
instance's key; users never carry one.

**Region is eu-central-1, and that is not a preference.** eu-west-1 carries no
rerank model at all; us-east-1 would send Astound's Confluence and Jira content
to the US. The config validator refuses a non-EU region at load.

| Capability | Result |
|---|---|
| `amazon.titan-embed-text-v2:0` — dims 256/512/1024 | 200 |
| `cohere.rerank-v3-5:0` via `InvokeModel` | 200 |
| `openai.gpt-oss-120b-1:0` via `Converse` | 200 |
| `bedrock-agent` (managed Knowledge Bases) | **403 — denied** |
| Model-invocation logging | **not configured** (REQ-026 reviewers will ask) |

These calls bypass the REQ-037/038 residency mechanism on the AI gateway; the
region guard in `src/model/bedrock` and the config validator are the controls.

## Tool contract

| Node tool | This server | Who | Notes |
|---|---|---|---|
| `search(query, source_type[] REQUIRED, top_k?)` | `search_knowledge(query, source_types?, top_k?)` | any user | Omitting `source_types` searches every *live* category; archive categories only when named — enforced in the retriever. Hits carry `doc_id`, `source_type`, `source`, `title`, `snippet`, `uri`, `lines`, `date`, `date_source`, `chunk_index/total`, `score`. Only documents the caller holds a grant on. |
| `list_sources()` | `list_knowledge_sources()` | any user | Every configured category, with the *caller's* readable document count and the *caller's* last sync. |
| `/project-context:sync` (Cursor command) | `sync_knowledge(source_types?, rebuild?)` | any user, needs `x-atlassian-*` headers | Reads Confluence/Jira as the caller in the background; grants what it writes to `user:<caller>`. `rebuild` ignores the caller's cursor and re-chunks what it fetches. Single-flight per caller. |
| `index_stats()` | `knowledge_index_stats()` | admin | Documents, chunks, grants, distinct syncing users, embedding-cache rows, database size, `last_built`, `version`. |
| — | `prune_knowledge(stale_days?, dry_run?)` | admin | Drops `user:` grants of users with no sync in `stale_days` (default 90) or whose `users` row is gone, deletes documents with zero grants, prunes the date cache (missing docs) and the embedding cache (unreferenced, older than `stale_days`). Returns counts; `dry_run` deletes nothing. |

## Decisions

1. **Vector store: pgvector**, with the grant join *inside* the candidate
   queries (2026-08-28), so an unauthorized row is never read and the result
   count never leaks how many were removed.
2. **Remote server, one shared index on a dedicated pgvector Postgres — a hard
   requirement** (2026-09-01). On Fly the index is its own Postgres app named by
   the `knowledge_database_url` secret; elsewhere the instance database is used.
   No pgvector or no `bedrock_api_key` → the child exits and the platform boot
   fails. See `docs/decisions/2026-09-01-knowledge-bank-remote-shared-index.md`.
3. **Ingestion runs inside the knowledge-bank process, triggered only by
   `sync_knowledge`** — no cron, no CLI (the earlier "core scheduler jobs" idea is
   forbidden by `scripts/lint-layers.sh`; see the ingestion-in-process record).
4. **Archive semantics enforced in the retriever**: a category with
   `archived: true` is searched only when the caller names it.
5. **Only user credentials travel; grants are per user.** Confluence and Jira are
   read as the caller with the credentials on their MCP connection, and every
   document a user syncs is granted `user:<id>`. No service account, no group
   map, no source-configured grants.
6. **Provider seam**: `ModelProvider` trait, `model.provider: bedrock`. A swap is
   configuration plus one implementation; capability logic never names Bedrock.

## Where REQ-052 (P0) is enforced

`KnowledgeRetriever` carries the calling `Principal` on every method that can
surface content. A principal reduces to `(subject_kind, subject_value)` pairs in
the vocabulary `src/principal.rs` defines (`user`, `role`, and the attribute
bands), joined against `knowledge_document_grants` inside both candidate arms
and the source listing. Only `user:<id>` grants are written now, one per user
who synced the document, so the set of people who can find a page is exactly
the set whose own Atlassian account read it. On a complete fetch, the caller's
grants on documents that disappeared upstream are removed; a document left with
zero grants is deleted (chunks cascade) at that sync or by `prune_knowledge`.
The `role` and `adfs_group` subject kinds stay in the schema and the join but
nothing writes them.

## What is still blocked

- **The Fly knowledge database** (`blockers.md` B5) and whether the stock
  `postgres-flex` image ships pgvector (B8).
- **Corpus users can read** (B2): the configured space and project are
  `EAICOMM`; the material lives in spaces and projects to list once users can
  read them.
- **Corpus for parity (M1)** (B1): file sources are gone, so the hand-authored
  trees are indexed only once they are in Confluence.
- **Atlassian OAuth app** for the live Jira/Confluence servers (B4) — and, later,
  federated identity replacing the header token as the credential source.

## Operating it

```bash
# instance secrets (profile store)
systemprompt admin config secret set bedrock_api_key <key>
systemprompt admin config secret set knowledge_database_url \
  'postgres://knowledge:<pw>@sp-a2f658d8bc5f-knowledge.flycast:5432/knowledge' --profile production

# the Fly database
just knowledge-db-create && KNOWLEDGE_DB_PASSWORD=... just knowledge-db-provision
just knowledge-db-report
just knowledge-db-prune            # dry run; DRY_RUN=0 to apply

# health
systemprompt plugins mcp logs knowledge-bank
```

Each user connects with their own headers (`.mcp.json` in the skill):

```json
"headers": { "Authorization": "Bearer ${SYSTEMPROMPT_PAT}",
             "x-atlassian-email": "${ATLASSIAN_EMAIL}",
             "x-atlassian-api-token": "${ATLASSIAN_API_TOKEN}" }
```

then `sync_knowledge` → `list_knowledge_sources` → `search_knowledge`.
Administrators run `prune_knowledge {dry_run: true}` to see what a prune would
remove. Sync state per `(source, user)` is in `knowledge_user_sync_state`; the
embedding and date caches are `knowledge_embedding_cache` and
`knowledge_date_cache`.
