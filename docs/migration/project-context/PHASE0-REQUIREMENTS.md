# Phase 0 — project-context on systemprompt · Requirements (DRAFT)

*Customer requirements for re-platforming the `project-context` knowledge/RAG capability onto systemprompt. Written from the Customer's perspective for the Vendor delivering systemprompt: it states **what** is required and how delivery is judged, not **how** to build it. Technology-neutral — where a concrete technology is named (Bedrock, LanceDB, MCP), it is the **current reference implementation** the Vendor may match or replace with an equivalent.*

Status: DRAFT for review. Companion to the as-is architecture pack
([`00-system-overview.md`](00-system-overview.md), [`c1`](c1-store.md)–[`c9`](c9-policy.md)),
which is the authoritative description of the proven baseline behind every
requirement below.

---

## 1. Context & goal

The Customer is consolidating its bespoke tooling — the **"AI Kit"** solution — onto
**systemprompt**, a single, self-hosted, governed AI platform — to gain one
identity/authorization model, one audit trail, provider flexibility, and an
air-gap-capable deployment, and to retire a scattered set of standalone Node
scripts. This is a multi-phase program. ("AI Kit" is the Customer's name for the
overall solution being migrated; project-context is one capability within it.)

**This document covers Phase 0 only: the `project-context` capability.** It is
addressed first because it is **business-critical** (it is the project memory
agents rely on to ground decisions, scope, and planning) yet **architecturally
independent** — it has a clean boundary (an index on disk + a tool interface) and
does not depend on the rest of the migration to deliver value. Later phases
(re-platforming the rest of the "AI Kit" solution) exist but are **out of scope
here** and are named only where they affect Phase 0 boundaries.

`project-context` today is a local, file-based RAG engine: documents from several
sources are synced, normalized, chunked, embedded, and stored, then served to an
agent through a small tool interface via hybrid search with reranking. The full
as-is behavior is documented in the companion pack; Phase 0 must reproduce that
capability on systemprompt and, in addition, gain the platform's authorization
and audit guarantees.

---

## 2. Scope & phasing

**In scope (Phase 0):** the complete project-context capability — ingestion of
its current source types, index-time processing (chunking, enrichment, dating),
embeddings and model access, hybrid retrieval with reranking, the agent-facing
tool interface, and the operational qualities (incremental indexing, hot reload,
air-gap, self-sufficiency) — delivered on systemprompt with authorization and
audit added.

**Explicitly out of scope (Phase 0):**
- Re-platforming any other part of the "AI Kit" solution (later phases).
- Changing the *content* of the corpus or the source systems themselves.
- The agent-side usage policy (authority/temporal ranking) as engine logic — it
  remains a consumer contract (see [`c9-policy.md`](c9-policy.md)); Phase 0 must
  only supply the metadata that policy relies on.

**Independence requirement:** Phase 0 SHALL be deliverable and operable without
any later phase, and its serving path SHALL NOT depend on any other migrated
component at runtime (see NFR-SELF-1).

**Suggested delivery milestones (outcome gates, non-prescriptive):**
- **M1 — Retrieval parity:** serving path answers queries with parity to the
  current system over a fixed evaluation set, against an index populated from the
  existing corpus.
- **M2 — Ingestion self-sufficiency:** systemprompt produces and refreshes the
  index itself (sync + processing + embedding), with no dependency on the legacy
  Node engine.
- **M3 — Governance & cutover:** authorization + audit enforced; legacy engine
  decommissioned.

---

## 3. Business requirements

| ID | Requirement | Rationale |
|---|---|---|
| **BR-1** | project-context SHALL run as a first-class capability of the systemprompt platform, not as an external side-car. | Single platform, one operational surface. |
| **BR-2** | Every retrieval and administrative action SHALL be attributable to an identity and recorded in the platform audit trail. | Governance/compliance; the legacy tool had none. |
| **BR-3** | The capability SHALL be operable fully self-hosted and air-gapped, depending only on the platform's datastore and a model provider. | Data control, enterprise/air-gap deployments. |
| **BR-4** | The solution SHALL NOT lock the Customer into a single model provider. | Cost/negotiation leverage, resilience. |
| **BR-5** | Migration SHALL preserve retrieval usefulness (no regression in answer quality or citations) as experienced by agents/users. | Business continuity of "project memory". |
| **BR-6** | Total cost of operation SHALL be controlled — unchanged content must not incur repeated model spend. | Predictable running cost at corpus scale. |
| **BR-7** | Phase 0 SHALL be independently deliverable and provide value before later phases begin. | De-risked, incremental program. |

---

## 4. Functional requirements

Each requirement: a `SHALL` statement, the reference baseline (companion doc),
and acceptance criteria. "Reference" points at the proven behavior; it is
descriptive, not a prescribed implementation.

### 4.1 Sources & content intake (FR-ING)

*Reference: [`c4-ingestion.md`](c4-ingestion.md), [`c6-indexer.md`](c6-indexer.md).*

| ID | Requirement (SHALL) | Acceptance |
|---|---|---|
| **FR-ING-1** | Ingest hand-authored documents (meeting notes/transcripts, ADRs) supplied by the team. | A newly added note becomes searchable after a refresh. |
| **FR-ING-2** | Sync from Atlassian **Confluence** (configured spaces, including page comments) into the corpus, incrementally. | Added/updated/deleted pages are reflected after a sync; deletions removed. |
| **FR-ING-3** | Sync from Atlassian **Jira** (configured projects/filters, including comments and status) into the corpus, incrementally. | Ticket changes reflected after a sync. |
| **FR-ING-4** | Ingest structured spec sources (current specs and archived/historical change material) as distinct categories. | Each configured category is listable and searchable. |
| **FR-ING-5** | Accept documents in multiple formats — at minimum Markdown, plain text, PDF, DOCX — extracting readable text. | A PDF and a DOCX yield searchable text; unsupported formats are ignored, not fatal. |
| **FR-ING-6** | Allow **uploading/adding new documents** into the corpus without a full rebuild. | A new file is indexed incrementally on the next refresh. |
| **FR-ING-7** | Keep the index a faithful mirror of the sources — removed source content SHALL be pruned. | A deleted source document produces no further hits. |
| **FR-ING-8** | Support adding new source categories by configuration. | A new category can be introduced without code changes to core logic. |

### 4.2 Index-time processing (FR-PROC)

*Reference: [`c5-chunking.md`](c5-chunking.md), [`c6-indexer.md`](c6-indexer.md).*

| ID | Requirement (SHALL) | Acceptance |
|---|---|---|
| **FR-PROC-1** | Normalize heterogeneous inputs into a common text form for indexing. | Binary/HTML sources are indexed as extracted text. |
| **FR-PROC-2** | Chunk documents using strategies appropriate to the source type (e.g. semantic segmentation for transcripts, structure-aware splitting for documents, record-wise splitting for tickets). | Chunk boundaries respect the source shape per category. |
| **FR-PROC-3** | Enforce chunk size bounds — no chunk exceeds the configured maximum, and tiny fragments are merged. | Automated check: zero chunks above the ceiling; no sub-minimum fragments. |
| **FR-PROC-4** | Enrich chunks so each is self-contained for retrieval (context added without altering the cited source). | Enriched chunks retain a correct pointer to the original text span. |
| **FR-PROC-5** | Assign each document an effective **date** with recorded provenance (authored vs derived). | Every result carries a date and a date-source. |
| **FR-PROC-6** | Preserve a precise **citation pointer** (source file + line range) for every chunk. | Each hit can be opened at the exact span it came from. |
| **FR-PROC-7** | Indexing SHALL be **incremental and idempotent** — only changed content is reprocessed. | Re-indexing an unchanged corpus performs no model calls (see NFR-COST-1). |

### 4.3 Model access (FR-MODEL)

*Reference: [`c2-model.md`](c2-model.md).*

| ID | Requirement (SHALL) | Acceptance |
|---|---|---|
| **FR-MODEL-1** | Generate vector embeddings for chunks and for queries. | Query and chunks share an embedding space; dimension is configurable. |
| **FR-MODEL-2** | Provide LLM access for index-time tasks (chunk segmentation, enrichment, dating). | These tasks succeed against the configured provider. |
| **FR-MODEL-3** | Provide **reranking** of candidate results. | Reranking can be enabled/disabled by configuration. |
| **FR-MODEL-4** | Model access SHALL be **provider-agnostic** — the provider is configurable and replaceable without changing capability logic. | A provider/model swap is a configuration change (Bedrock is the current reference). |
| **FR-MODEL-5** | Model calls SHALL be resilient — transient failures are retried, and a persistent failure degrades gracefully rather than failing the whole run. | Injected transient errors are retried; a hard failure isolates to the affected item. |

### 4.4 Retrieval (FR-RET)

*Reference: [`c7-retrieval.md`](c7-retrieval.md).*

| ID | Requirement (SHALL) | Acceptance |
|---|---|---|
| **FR-RET-1** | Provide **hybrid search** combining semantic (vector) and lexical (full-text) retrieval, fused into one ranked list. | Both signals contribute; a lexical-only match and a semantic-only match can both surface. |
| **FR-RET-2** | Rerank the fused candidate pool before returning results (when enabled). | With rerank on, ordering reflects the reranker; with it off, the fusion order. |
| **FR-RET-3** | Require the caller to **scope** a query to one or more source categories; broad search means naming categories explicitly. | A query without a scope is rejected or defaulted per policy (see open item O-2). |
| **FR-RET-4** | Treat **archive/historical categories as opt-in** — searched only when explicitly named. | Archive content does not appear unless requested. |
| **FR-RET-5** | Return **citations** for each hit: category, source system, title, snippet, source pointer (file+line), date, and a relevance score. | Every returned record contains these fields. |
| **FR-RET-6** | Return a **bounded** number of results (configurable default, capped maximum). | Result count never exceeds the cap. |
| **FR-RET-7** | Pick up a rebuilt index **without a restart or downtime** (hot reload). | After a rebuild, the next query serves the new index with no service restart. |

### 4.5 Access & interface (FR-ACC)

*Reference: [`c8-mcp.md`](c8-mcp.md), [`c9-policy.md`](c9-policy.md), platform governance.*

| ID | Requirement (SHALL) | Acceptance |
|---|---|---|
| **FR-ACC-1** | Expose retrieval to agents/clients through a standard tool interface (search, list sources, index stats). | The three capabilities are callable by a compliant client. |
| **FR-ACC-2** | Gate all access through **authorization** — authenticated identity, roles/scopes; unauthorized calls are denied. | An unauthenticated or under-scoped call is denied with a clear reason. |
| **FR-ACC-3** | **Audit** every call (identity, action, parameters, outcome, timing). | Each call yields an audit record retrievable by an operator. |
| **FR-ACC-4** | Expose **health/introspection** — list of source categories with counts and last-sync time, and index statistics. | `list sources` and `index stats` return live figures. |
| **FR-ACC-5** | Errors SHALL be returned as informative results, never crash the service or leak internals. | A backend error returns a safe message; the service stays up. |
| **FR-ACC-6** | Supply the metadata the agent usage policy relies on (category, date, date-source) so authority/temporal ranking can be applied by the consumer. | Every hit carries the fields the policy in `c9` requires. |

---

## 5. Non-functional requirements (NFR)

| ID | Requirement (SHALL) | Acceptance |
|---|---|---|
| **NFR-SELF-1** | Serving SHALL be independent of ingestion at runtime — the only coupling is the produced index + a version marker. | Ingestion can be stopped while serving continues on the last index. |
| **NFR-AIRGAP-1** | Operate air-gapped, depending only on the platform datastore and a reachable model provider endpoint. | No outbound dependency beyond the model provider is required to serve. |
| **NFR-AUTHZ-1** | Authorization SHALL use the platform's identity/role model (least privilege by default). | Access is denied unless explicitly granted. |
| **NFR-AUDIT-1** | Audit records SHALL be durable and queryable via the platform. | Audit rows survive restart and are queryable. |
| **NFR-PERF-1** | The serving process SHALL start quickly (heavy resources initialized lazily) and answer typical queries within an agreed latency target. | Cold start and p95 query latency meet agreed thresholds (to set — O-4). |
| **NFR-COST-1** | Unchanged content SHALL NOT incur repeated embedding/LLM spend across runs; non-deterministic results (e.g. LLM-derived dates) are cached and shareable. | A no-change re-index reports zero model calls. |
| **NFR-PROV-1** | No hard dependency on a single model provider (see FR-MODEL-4). | Provider is configuration; documented swap procedure exists. |
| **NFR-OBS-1** | Provide operational logging/metrics without corrupting the tool protocol channel. | Logs are separated from protocol output; key metrics are exposed. |
| **NFR-DEGRADE-1** | Degrade gracefully — absence/failure of the LLM, reranker, or full-text index reduces quality but does not stop retrieval or indexing. | With each dependency disabled in turn, the system still returns results. |
| **NFR-SEC-1** | Provider and Atlassian credentials SHALL be held in the platform secret store, never in code or logs. | Secrets are referenced by name; none appear in output. |
| **NFR-SCALE-1** | Handle the current corpus size with headroom for growth (order-of-magnitude), without redesign. | Meets latency/cost targets at the agreed target corpus size. |

---

## 6. Constraints & assumptions

- **Datastore:** the platform runs PostgreSQL per instance; the solution should
  fit that operational story (the current reference uses a local file store —
  Vendor may propose an equivalent).
- **Model provider (current):** AWS Bedrock — embeddings (Titan v2, 1024-dim),
  an LLM for chunking/enrichment/dating, and a reranker (Cohere), authenticated by
  a bearer key. This is the reference, subject to FR-MODEL-4/NFR-PROV-1.
- **Sources (current):** meeting notes, ADRs, Confluence, Jira, current specs,
  archived changes. New categories are configuration.
- **No changes to the AI gateway** are assumed necessary — model calls happen
  inside the capability's own process/jobs.
- **Corpus content authority** (which source outranks which) is a *consumer*
  concern (see [`c9-policy.md`](c9-policy.md)); Phase 0 supplies the metadata, not
  the ranking policy.

---

## 7. Acceptance criteria (Phase 0 rollup)

Phase 0 is accepted when:

1. **Functional parity** — all FR-* pass their acceptance checks; a fixed
   evaluation query set returns results at parity with the current system
   (same top hits and citations within an agreed tolerance).
2. **Ingestion self-sufficiency** — the platform performs sync + processing +
   embedding + indexing itself; the legacy Node engine is not required to produce
   or refresh the index.
3. **Governance** — every access is authenticated, authorized, and audited
   (FR-ACC-2/3, NFR-AUTHZ-1, NFR-AUDIT-1).
4. **Operational** — hot reload (FR-RET-7), incremental/idempotent indexing
   (FR-PROC-7/NFR-COST-1), graceful degradation (NFR-DEGRADE-1), and air-gap
   (NFR-AIRGAP-1) demonstrated.
5. **Independence** — Phase 0 operates with no runtime dependency on later phases
   (NFR-SELF-1, BR-7).

---

## 8. Out of scope (Phase 0)

- Any later migration phase (the rest of the "AI Kit" solution).
- Modifying source-of-truth systems (Confluence/Jira) or corpus content.
- Encoding the authority/temporal ranking policy into the engine.
- New retrieval features beyond current capability (semantic filters, cross-doc
  synthesis, etc.) — candidates for a later phase.

---

## 9. Open items (to resolve during review)

| ID | Question |
|---|---|
| **O-1** | Target datastore for vectors + full-text (platform Postgres vs a ported file store) — Vendor to propose against NFRs. **Resolved 2026-09-02:** platform Postgres with pgvector (vector + `tsvector` in one `knowledge_chunks` table); see `docs/decisions/2026-08-28-rag-pgvector-and-eu-central-1.md`. |
| **O-2** | Is query scoping (FR-RET-3) strictly required-non-empty, or allowed to default to "all live categories"? **Resolved 2026-09-02:** allowed to default — an omitted `source_types` searches every live category; archive categories still need naming. |
| **O-3** | Is archive opt-in (FR-RET-4) a tool-contract convention or an enforced filter? **Resolved 2026-09-02:** an enforced filter in the retriever, not a description-only convention. |
| **O-4** | Concrete latency (NFR-PERF-1) and target corpus size (NFR-SCALE-1) numbers. |
| **O-5** | Ingestion trigger surface — scheduled jobs vs operator-run command — and refresh cadence. **Resolved 2026-09-02:** user-triggered — `sync_knowledge` (per user, own credentials) and `add_knowledge_document`; no schedule. |
| **O-6** | Retrieval-quality parity metric + evaluation set definition for acceptance §7.1. |

---

## 10. References (as-is baseline)

Requirement-to-test traceability for the delivered implementation:
[`TRACEABILITY.md`](TRACEABILITY.md), enforced by `tests/unit/mcp/src/phase0_traceability.rs`.


The proven behavior behind these requirements:

- [`00-system-overview.md`](00-system-overview.md) — whole-system narrative + dataflows.
- [`c1-store.md`](c1-store.md) — index/store model. · [`c2-model.md`](c2-model.md) — model access.
- [`c3-platform.md`](c3-platform.md) — config/env/platform. · [`c4-ingestion.md`](c4-ingestion.md) — sources & intake.
- [`c5-chunking.md`](c5-chunking.md) — chunking strategies. · [`c6-indexer.md`](c6-indexer.md) — indexer + dating.
- [`c7-retrieval.md`](c7-retrieval.md) — hybrid search + rerank. · [`c8-mcp.md`](c8-mcp.md) — tool interface.
- [`c9-policy.md`](c9-policy.md) — agent usage policy (consumer).

**Traceability (requirement group → baseline):** FR-ING→c4/c6 · FR-PROC→c5/c6 ·
FR-MODEL→c2 · FR-RET→c7 · FR-ACC→c8/c9 · NFR-SELF/PERF→c7 · NFR-COST→c6.
