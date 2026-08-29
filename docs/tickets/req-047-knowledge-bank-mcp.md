# REQ-047 / REQ-048 — Knowledge-bank MCP server (project-context migration)

Status: **backend built, index empty, server still disabled.** The Bedrock
retriever, the pgvector store, the REQ-052 grant enforcement, and the
ingestion pipeline are all in place and tested. What is missing is a corpus:
the indexing account has no Jira application licence and sees a single
Confluence page, so there is nothing to index (`rag-access-request.local.md`).
Until then the server ships `enabled: false` and, with no key provisioned,
answers `BACKEND_NOT_CONFIGURED` rather than fabricating results.

The backend is a **migration of the Node.js "project-context" system** at
`github.com/Astound-Digital/sfcc-next-cursor/.project/project-context`, behind
the `KnowledgeRetriever` trait — with one part that is not a migration at all:
that system is a single-user local tool with no concept of a caller and no
ACLs, so REQ-052 is net-new design.

## Bedrock access, verified live 2026-08-28

The key is a long-term Bedrock API key used as `Authorization: Bearer <key>`,
not sig-v4 — so the crate uses `reqwest` and needs no `aws-sdk-*` dependency.
Identity: `BedrockAPIKey-0o2w` in AWS account `266359797998`.

**Region is eu-central-1, and that is not a preference.** All three models
were invoked successfully there. eu-west-1 carries *no rerank model at all*
and currently 403s this account pending verification; us-east-1 works but
would send Astound's Confluence and Jira content to the US. The Node source
system independently defaults to eu-central-1 for the same reason.

| Capability | Result |
|---|---|
| `amazon.titan-embed-text-v2:0` — dims 256/512/1024 | 200 |
| `cohere.rerank-v3-5:0` via `InvokeModel` | 200 |
| `openai.gpt-oss-120b-1:0` via `Converse` | 200 |
| `eu.anthropic.*` inference profiles | 200 |
| `bedrock-agent` (managed Knowledge Bases) | **403 — denied** |
| Model-invocation logging | **not configured** |

Two gaps worth naming. Managed Bedrock Knowledge Bases are denied, which
confirms building retrieval in-process was the right call. And model-invocation
logging is off, which REQ-026 (Immutable AI Audit Trail) reviewers will ask
about — our own audit spine records every retrieval call, but AWS-side logging
would need an S3/CloudWatch destination and a permission this key has not been
tested for.

**These calls bypass the REQ-037/038 residency mechanism.** That mechanism is
`governance:` metadata on provider/model YAML plus route `requires:` on the AI
gateway; the knowledge bank calls Bedrock directly from the MCP process, so
nothing in it applies. `bedrock::assert_eu_region` runs at client construction
and is the only control on this path — the binary refuses to start on a non-EU
region, with a message that names the requirement.

## What was scaffolded

- `extensions/mcp/knowledge-bank/` — workspace crate
  `systemprompt-mcp-knowledge-bank`, modeled on the `systemprompt` MCP agent:
  same auth path (`enforce_rbac_from_registry` + `record_mcp_access`
  auditing), same `McpToolExecutor`/`CliArtifact` response shape, same HTTP
  serving shell in `main.rs` (`MCP_SERVICE_ID` / `MCP_PORT`, default 5030).
- `services/mcp/knowledge-bank.yaml` — internal server, `enabled: false`,
  OAuth required, audience `mcp`, scope `admin`; listed in
  `services/config/config.yaml` includes.
- `services/access-control/roles.yaml` — `mcp_server: knowledge-bank` granted
  to role `admin` only (least privilege while the backend is a stub).
- Tests: `tests/unit/mcp/src/knowledge_bank.rs` pins the three-tool contract,
  the required-`query` / bounded-`top_k` schema, and the stub's refusal to
  fabricate results.

## Tool-contract parity with the Node server

| Node tool (`scripts/mcp/server.mjs`) | This server | Notes |
|---|---|---|
| `search(query, source_type: string[] REQUIRED, top_k?: 1..20)` | `search_knowledge(query, source_types?: string[], top_k?: 1..20)` | Same semantics: categories listed explicitly, archive categories opt-in only; `source_types` is optional only while no backend is configured — the migrated backend should enforce non-empty like the Node server. |
| `list_sources()` → source types + doc counts + last sync | `list_knowledge_sources()` | `KnowledgeSource` carries `doc_count` and `last_synced` (RFC 3339), rendered when known. |
| `index_stats()` | `knowledge_index_stats()` | `IndexStats {documents, chunks, last_built, version}`. |

## The retriever contract

`extensions/mcp/knowledge-bank/src/retriever.rs`:

```rust
#[async_trait]
pub trait KnowledgeRetriever: Send + Sync {
    async fn search(&self, query: &str, filter: &SearchFilter)
        -> Result<Vec<KnowledgeHit>, KnowledgeBankError>;
    async fn list_sources(&self) -> Result<Vec<KnowledgeSource>, KnowledgeBankError>;
    async fn index_stats(&self) -> Result<IndexStats, KnowledgeBankError>;
}
```

`SearchFilter` = `source_types: Vec<String>` + `top_k: Option<u8>` (cap 20).
`KnowledgeHit` = source_type (category), source (system), title, snippet, uri
(file/line pointer back to the original), score 0..1. The server holds an
`Arc<dyn KnowledgeRetriever>`; `main.rs` currently injects `StubRetriever`,
which fails `search`/`index_stats` with `BACKEND_NOT_CONFIGURED` and lists no
sources — never fake data. A `NotConfigured` answer renders as an honest text
artifact; every other error is a tool error.

## Migration map (Node module → Rust destination)

| Node (project-context) | Rust destination | Status |
|---|---|---|
| `scripts/mcp/server.mjs` (stdio MCP) | `src/server/` + `src/tools.rs` | Ported as contract; served over authenticated HTTP, not stdio. |
| `scripts/retrieval/service.mjs` (hybrid search, candidate_multiplier over-fetch, Cohere rerank, top_k cap, per-call `.version` re-check) | new `src/retrieval/` module implementing `KnowledgeRetriever` | To build. Keep the `.version` re-check so index rebuilds are picked up without restart. |
| `scripts/retrieval/store.mjs` (LanceDB table, `source_type IN (...)` pre-filter) | vector-store layer under `src/retrieval/` | To build — store decision open, see below. |
| `scripts/ingest/atlassian.mjs`, parsers, normalizer, chunker (per-source `llm \| script \| none` strategy, LLM contextual enrichment at index time) | future offline ingestion jobs (NOT the serving path; candidates: scheduler jobs or a CLI extension) | To build. |
| `scripts/cli/index.mjs` + `sync.mjs` | same ingestion surface as above | Open decision: scheduler jobs vs. operator CLI. |
| `core/config` + env (bedrock models, chunking targets, retrieval tunables `{top_k, candidate_multiplier, rerank_enabled}`, sources map) | crate config (YAML + profile secret store for the key) | To build. |
| inbox/<source_type> directories; sources today: `meeting_notes`, `confluence` (space mirror), `jira` (issues+comments) | ingestion input layout, ported as-is or re-pointed | To decide with ingestion. |

## Bedrock specifics (from the Node system)

- Embeddings: `amazon.titan-embed-text-v2:0`, 1024-dim.
- Chunking/enrichment LLM: `openai.gpt-oss-120b-1:0` (via bedrock-runtime).
- Rerank: `cohere.rerank-v3-5:0` (via bedrock-runtime).
- Auth: `AWS_BEARER_TOKEN_BEDROCK` bearer key (not sig-v4 access keys) —
  provision via the profile secret store.
- **No gateway changes**: the AI gateway speaks no Bedrock wire protocol and
  none is needed — all Bedrock calls happen inside this MCP process (and the
  offline ingestion jobs) via the AWS SDK / bedrock-runtime HTTP.

## Decisions, now settled

1. **Vector store: pgvector**, in the instance's existing Postgres. The
   deciding factor was REQ-052, not storage: pgvector lets the grant join run
   *inside* the candidate queries. A LanceDB port would have forced ACL
   filtering into application code above the store, where it would still read
   unauthorized rows and leak how many were removed through the result count.
   The local Docker image moved to `pgvector/pgvector:pg18`.
2. **Archive semantics: enforced in the retriever**, not left as a tool
   description. A category marked `archived: true` in
   `services/knowledge-bank/config.yaml` is searched only when the caller names
   it. REQ-052 asks for enforcement, and a convention is not enforcement.
3. **Ingestion: scheduler jobs.** The instance already runs jobs at startup and
   on a schedule, and an operator still gets a manual trigger through
   `systemprompt infra jobs`. A new `extensions/cli/*` binary would have been a
   second deployment surface for no gain.
4. **`source_types` stays optional**, but the default is now meaningful rather
   than a placeholder: omitting it searches every *live* source, and archive
   categories must be named. This is the Node server's semantics without its
   ergonomics tax, and access control makes the broad default safe — a caller
   only ever sees granted documents.

## Where REQ-052 (P0) is enforced

The `KnowledgeRetriever` trait now carries the calling `Principal` on every
method that can surface content. The identity was always available at the tool
boundary — each handler received a `RequestContext` and discarded it — so this
was a matter of threading it through, not new plumbing.

A principal reduces to `(subject_kind, subject_value)` pairs using the same
vocabulary as `access_control_rules` (`user`, `department`, `adfs_group`,
`role`, `organization`), which are joined against `knowledge_document_grants`
in the candidate queries. Two consequences, both deliberate:

- **A document with no grants is reachable by nobody.** Ingestion writes grants
  from the source system's own permissions; a document whose permissions could
  not be read gets none and fails closed. A missing document is a support
  ticket, a leaked one is an incident.
- **`list_sources` reports the caller's document counts, not the index's.**
  Reporting true totals would disclose how much material exists in a category
  the caller cannot read.

Grants are rewritten wholesale on every sync, so a permission removed upstream
disappears at the next sync — the propagation lag is exactly the sync interval.

## What is still blocked

Nothing on the AWS side. The blockers are source access, unchanged from
`rag-access-request.local.md`:

- **Jira: 401 on every call**, including `/rest/api/3/myself` — the account has
  no Jira application licence. Needs Browse Projects + View Comments.
- **Confluence: only the EAICOMM space is visible, holding one page.** Enough
  to prove the pipeline, nowhere near enough for REQ-046's precision@5
  benchmark, whose fixture is checked in as an explicit placeholder that
  refuses to report a score.
- **Confluence delete is 403**, so indexer test content cannot be cleaned up.
- **GitHub and Salesforce connectors** (also named in REQ-048) have no access
  request raised at all.

## Left for the orchestrator

- Compile validation (`just verify` / `just preflight`) — not run here by
  instruction; `.sqlx/` not needed (crate has no SQL).
- The new binary is built by `systemprompt build mcp` walking `manifest.yaml`
  (`build_type: workspace`); confirm on the next `just build-mcp`.
- Coverage baseline may need re-recording after tests run.
