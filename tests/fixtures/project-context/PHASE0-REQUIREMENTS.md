# Knowledge retrieval test contracts

| ID | Requirement | Rationale |
|---|---|---|
| **BR-1** | project-context SHALL run as a first-class capability of the systemprompt platform, not as an external side-car. | Single platform, one operational surface. |
| **BR-2** | Every retrieval and administrative action SHALL be attributable to an identity and recorded in the platform audit trail. | Governance/compliance; the legacy tool had none. |
| **BR-3** | The capability SHALL be operable fully self-hosted and air-gapped, depending only on the platform's datastore and a model provider. | Data control, enterprise/air-gap deployments. |
| **BR-4** | The solution SHALL NOT lock the Customer into a single model provider. | Cost/negotiation leverage, resilience. |
| **BR-5** | Migration SHALL preserve retrieval usefulness (no regression in answer quality or citations) as experienced by agents/users. | Business continuity of "project memory". |
| **BR-6** | Total cost of operation SHALL be controlled — unchanged content must not incur repeated model spend. | Predictable running cost at corpus scale. |
| **BR-7** | Phase 0 SHALL be independently deliverable and provide value before later phases begin. | De-risked, incremental program. |
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
| ID | Requirement (SHALL) | Acceptance |
|---|---|---|
| **FR-PROC-1** | Normalize heterogeneous inputs into a common text form for indexing. | Binary/HTML sources are indexed as extracted text. |
| **FR-PROC-2** | Chunk documents using strategies appropriate to the source type (e.g. semantic segmentation for transcripts, structure-aware splitting for documents, record-wise splitting for tickets). | Chunk boundaries respect the source shape per category. |
| **FR-PROC-3** | Enforce chunk size bounds — no chunk exceeds the configured maximum, and tiny fragments are merged. | Automated check: zero chunks above the ceiling; no sub-minimum fragments. |
| **FR-PROC-4** | Enrich chunks so each is self-contained for retrieval (context added without altering the cited source). | Enriched chunks retain a correct pointer to the original text span. |
| **FR-PROC-5** | Assign each document an effective **date** with recorded provenance (authored vs derived). | Every result carries a date and a date-source. |
| **FR-PROC-6** | Preserve a precise **citation pointer** (source file + line range) for every chunk. | Each hit can be opened at the exact span it came from. |
| **FR-PROC-7** | Indexing SHALL be **incremental and idempotent** — only changed content is reprocessed. | Re-indexing an unchanged corpus performs no model calls (see NFR-COST-1). |
| ID | Requirement (SHALL) | Acceptance |
|---|---|---|
| **FR-MODEL-1** | Generate vector embeddings for chunks and for queries. | Query and chunks share an embedding space; dimension is configurable. |
| **FR-MODEL-2** | Provide LLM access for index-time tasks (chunk segmentation, enrichment, dating). | These tasks succeed against the configured provider. |
| **FR-MODEL-3** | Provide **reranking** of candidate results. | Reranking can be enabled/disabled by configuration. |
| **FR-MODEL-4** | Model access SHALL be **provider-agnostic** — the provider is configurable and replaceable without changing capability logic. | A provider/model swap is a configuration change (Bedrock is the current reference). |
| **FR-MODEL-5** | Model calls SHALL be resilient — transient failures are retried, and a persistent failure degrades gracefully rather than failing the whole run. | Injected transient errors are retried; a hard failure isolates to the affected item. |
| ID | Requirement (SHALL) | Acceptance |
|---|---|---|
| **FR-RET-1** | Provide **hybrid search** combining semantic (vector) and lexical (full-text) retrieval, fused into one ranked list. | Both signals contribute; a lexical-only match and a semantic-only match can both surface. |
| **FR-RET-2** | Rerank the fused candidate pool before returning results (when enabled). | With rerank on, ordering reflects the reranker; with it off, the fusion order. |
| **FR-RET-3** | Require the caller to **scope** a query to one or more source categories; broad search means naming categories explicitly. | A query without a scope is rejected or defaulted per policy (see open item O-2). |
| **FR-RET-4** | Treat **archive/historical categories as opt-in** — searched only when explicitly named. | Archive content does not appear unless requested. |
| **FR-RET-5** | Return **citations** for each hit: category, source system, title, snippet, source pointer (file+line), date, and a relevance score. | Every returned record contains these fields. |
| **FR-RET-6** | Return a **bounded** number of results (configurable default, capped maximum). | Result count never exceeds the cap. |
| **FR-RET-7** | Pick up a rebuilt index **without a restart or downtime** (hot reload). | After a rebuild, the next query serves the new index with no service restart. |
| ID | Requirement (SHALL) | Acceptance |
|---|---|---|
| **FR-ACC-1** | Expose retrieval to agents/clients through a standard tool interface (search, list sources, index stats). | The three capabilities are callable by a compliant client. |
| **FR-ACC-2** | Gate all access through **authorization** — authenticated identity, roles/scopes; unauthorized calls are denied. | An unauthenticated or under-scoped call is denied with a clear reason. |
| **FR-ACC-3** | **Audit** every call (identity, action, parameters, outcome, timing). | Each call yields an audit record retrievable by an operator. |
| **FR-ACC-4** | Expose **health/introspection** — list of source categories with counts and last-sync time, and index statistics. | `list sources` and `index stats` return live figures. |
| **FR-ACC-5** | Errors SHALL be returned as informative results, never crash the service or leak internals. | A backend error returns a safe message; the service stays up. |
| **FR-ACC-6** | Supply the metadata the agent usage policy relies on (category, date, date-source) so authority/temporal ranking can be applied by the consumer. | Every hit carries the fields the policy in `c9` requires. |
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
| ID | Question |
|---|---|
