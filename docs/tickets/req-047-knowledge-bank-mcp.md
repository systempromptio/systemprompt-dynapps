# REQ-047 / REQ-048 — Knowledge-bank MCP server (project-context, Phase 0)

Status: **built end to end; corpus pending.** The serving path, the ingestion
pipeline, the per-document access control (REQ-052), the schema, and the
agent-side skill are implemented and tested. What is missing is a corpus: the
Atlassian indexing account has no Jira application licence and sees one
Confluence space, and the hand-authored `meeting_notes` / `adrs` / `specs`
trees have not been landed under `storage/knowledge/`. Requirements and the
as-is baseline: `docs/migration/project-context/`.

The backend is a **migration of the Node.js "project-context" system** at
`github.com/Astound-Digital/sfcc-next-cursor/.project/project-context`, with two
parts that are not a migration at all: that system is a single-user local tool
with no caller and no ACLs, so REQ-052 is net-new; and it refreshed its index
from a Cursor slash command, so the in-process scheduler, the `ingest`
subcommand and the `sync_knowledge` tool are ours.

## Where things are

| Concern | Location |
|---|---|
| Binary, tools, server | `extensions/mcp/knowledge-bank/src/{main,cli,tools,server/*}.rs` |
| Schema (crate-owned, applied at boot after a pgvector probe) | `extensions/mcp/knowledge-bank/schema/knowledge_bank.sql`, `src/schema.rs` |
| Model provider seam + Bedrock | `src/model/` |
| Retrieval (hybrid, RRF, rerank, grant join) | `src/retrieval/` |
| Ingestion (connectors, chunking, dating, enrichment, caches, scheduler) | `src/ingest/` |
| Configuration | `services/knowledge-bank/config.yaml` (crate-owned; validated at load) |
| Server registration + ACL | `services/mcp/knowledge-bank.yaml`, `services/access-control/roles.yaml` |
| Agent contract | `services/skills/project_context/SKILL.md`, plugin `services/plugins/project-context` |
| Tests | `tests/unit/mcp/src/knowledge_*.rs`, `tests/integration/mcp/src/knowledge_*.rs` |

## Bedrock access, verified live 2026-08-28

The key is a long-term Bedrock API key used as `Authorization: Bearer <key>`,
not sig-v4 — the crate uses `reqwest` and needs no `aws-sdk-*` dependency.
Identity: `BedrockAPIKey-0o2w` in AWS account `266359797998`.

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

| Node tool | This server | Notes |
|---|---|---|
| `search(query, source_type[] REQUIRED, top_k?)` | `search_knowledge(query, source_types?, top_k?)` | Omitting `source_types` searches every *live* category; archive categories are searched only when named — enforced in the retriever, not just described. Hits carry `doc_id`, `source_type`, `source`, `title`, `snippet`, `uri`, `lines`, `date`, `date_source`, `chunk_index/total`, `score`. |
| `list_sources()` | `list_knowledge_sources()` | Every configured category, with the *caller's* readable document count and last sync. |
| `index_stats()` | `knowledge_index_stats()` | Totals, `last_built`, `version` (max `indexed_at`, stamped last). |
| `/project-context:sync` (Cursor command) | `sync_knowledge(source_types?, rebuild?)` | Admin role required; starts a background run. Same library path as the schedule and the `ingest` subcommand. |

## Decisions

1. **Vector store: pgvector**, in the instance's own Postgres (2026-08-28). The
   deciding factor was REQ-052: the grant join runs *inside* the candidate
   queries, so an unauthorized row is never read and the result count never
   leaks how many were removed.
2. **One database, container deployments run pgvector; Fly staging is inert by
   design** (2026-09-01). The binary probes `pg_available_extensions` and applies
   its own idempotent schema; without `vector` it serves the stub. See
   `docs/decisions/2026-09-01-knowledge-bank-ingestion-in-process.md`.
3. **Ingestion runs inside the knowledge-bank process** (2026-09-01) — supersedes
   the earlier "core scheduler jobs" decision, which `scripts/lint-layers.sh`
   forbids (jobs is app-tier, MCP crates are entry-tier). Triggers: cron task
   (`ingestion.enabled`), `ingest [--rebuild] [--source <id>]`, `sync_knowledge`.
4. **Archive semantics enforced in the retriever**: a category with
   `archived: true` is searched only when the caller names it.
5. **Unrestricted content → `role: user`** (everyone signed in), overridable per
   source with `grants:`; restrictions narrow through linked accounts, e-mail and
   `atlassian.group_map`; unmappable restrictions fail closed.
6. **Indexer identity is a service account** (`atlassian_*` secrets). Per-user
   OAuth grants stay with the live Jira/Confluence MCP servers.
7. **Provider seam**: `ModelProvider` trait, `model.provider: bedrock`. A swap is
   configuration plus one implementation; capability logic never names Bedrock.

## Where REQ-052 (P0) is enforced

`KnowledgeRetriever` carries the calling `Principal` on every method that can
surface content. A principal reduces to `(subject_kind, subject_value)` pairs in
the vocabulary `src/principal.rs` defines (`user`, `role`, and the attribute
bands), joined against `knowledge_document_grants` inside both candidate arms
and the source listing. A document with no grants is reachable by nobody;
grants are rewritten wholesale on every sync, and for unchanged Confluence
pages the restrictions are still re-read, so a permission dropped upstream
disappears at the next sync.

## What is still blocked

- **Atlassian service account**: Jira 401 on every call (no application
  licence — needs Browse Projects + View Comments); Confluence sees only one
  space. Until then `confluence`/`jira` runs report an error and index nothing.
- **Corpus for parity (M1)**: the `meeting_notes`, `adrs`, `specs` and
  `openspec_archive` trees from the Cursor repo are not landed under
  `storage/knowledge/`; the REQ-046 precision@5 fixture
  (`tests/integration/mcp/fixtures/knowledge-benchmark.json`) is still a
  placeholder and refuses to score.
- **Atlassian OAuth app** for the live Jira/Confluence servers
  (`services/web/config/atlassian.yaml`), and the unverified
  `mcp.atlassian.com` audience question — unrelated to the index but part of
  the "open the cited source as yourself" story.
- **Are Atlassian groups AD-synced?** Decides whether `atlassian.group_map` can
  be an identity mapping or must be hand-kept.
- **Fly staging**: no pgvector on the managed database; out of scope by
  decision.

## Operating it

```bash
# secrets (profile store)
systemprompt admin config secret set bedrock_api_key <key>
systemprompt admin config secret set atlassian_base_url https://<site>.atlassian.net
systemprompt admin config secret set atlassian_email <service account>
systemprompt admin config secret set atlassian_api_token <token>

# one-shot runs
systemprompt-mcp-knowledge-bank ingest
systemprompt-mcp-knowledge-bank ingest --rebuild --source adrs

# health
systemprompt plugins mcp logs knowledge-bank
```

Sync state per source is in `knowledge_sync_state` (cursor, last error,
counters); the embedding and date caches are `knowledge_embedding_cache` and
`knowledge_date_cache`.
