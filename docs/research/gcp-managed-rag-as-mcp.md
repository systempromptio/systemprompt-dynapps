# Consuming Google Cloud Managed RAG from Astound

## Executive conclusion

Google Cloud's RAG Engine can be consumed through a Google-hosted remote MCP
server. The relevant endpoint is the regional Agent Platform retrieval toolset,
for example:

```text
https://europe-west3-aiplatform.googleapis.com/mcp/retrieval
```

It currently exposes four retrieval tools, including `retrieve_contexts`, and
Google separately exposes a `/mcp/ragdata` toolset for corpus creation and file
imports.[^1] Frankfurt (`europe-west3`) is a generally available RAG Engine
region, and Google lists a regional MCP retrieval endpoint there.[^2]

This means Astound can outsource parsing, chunking, embeddings, vector storage,
retrieval, and optional reranking. It does **not** mean the Google MCP endpoint
is a drop-in replacement for Astound's knowledge-bank contract.

The production recommendation is:

> Use RAG Engine as the managed retrieval backend, but keep a thin,
> Astound-governed `knowledge` MCP facade that exposes only the stable,
> permission-safe tools Astound wants agents to call.

The facade should be much smaller than the current knowledge engine. It would
not run pgvector, embed documents, parse files, or rank chunks. It would:

1. derive the caller's permitted organisation/project/source scope;
2. select a fixed allowlisted corpus;
3. construct or enforce any metadata filter server-side;
4. call RAG Engine with an Astound workload identity;
5. normalise returned contexts into Astound citations;
6. emit the existing governance/audit/cost trace.

Directly registering Google's MCP endpoint in `services/mcp/` is technically
viable for a proof of concept or for a corpus whose entire readership is the
same. It should not be used for a mixed-permission enterprise corpus. The
Google tool schema lets the caller supply the project, corpus, file IDs,
metadata filter, `topK`, thresholds, and reranker. Astound's current governance
authorises an MCP server/tool, not arbitrary values nested inside that tool's
arguments. A caller able to omit or rewrite a security metadata filter can
bypass a filter that was only suggested in a skill prompt.

## What Google is offering

Several similarly named Google products need to be separated.

| Layer | Google resource | What it provides | MCP relationship |
|---|---|---|---|
| RAG orchestration | RAG Engine (`RagCorpus`, `RagFile`) | ingestion, parsing, chunking, embeddings, retrieval and generation integration | Agent Platform provides first-party retrieval and data-management MCP toolsets |
| Default managed storage | `RagManagedDb` in Spanner mode | Google-managed Spanner for RAG resources and optionally vector search | reached through RAG Engine, not directly as a database MCP server |
| Serverless vector storage | RAG-managed Agent Retrieval / Vector Search 2.0 | managed collection and vector search | RAG Engine uses it; Agent Retrieval also has its own separate read-only MCP endpoint |
| Agent Retrieval | formerly Vector Search 2.0 | general JSON data-object search: vector, semantic, hybrid and text | separate `https://vectorsearch.googleapis.com/mcp`, currently Preview |

RAG Engine's own first-party MCP endpoints are the best match for the question:

```text
https://REGION-aiplatform.googleapis.com/mcp/retrieval
https://REGION-aiplatform.googleapis.com/mcp/ragdata
```

The retrieval toolset advertises:

- `retrieve_contexts` — raw relevant chunks, scores, source URI/display name,
  file/chunk IDs and page spans;
- `augment_prompt` — retrieves contexts and returns a prompt prepared for a
  subsequent generation call;
- `corroborate_content` — checks content against retrieved evidence;
- `ask_contexts` — a higher-level question-over-context operation.

The data toolset advertises:

- `create_rag_corpus`;
- `list_rag_corpora`;
- `import_rag_files`;
- `get_operation`;
- `delete_rag_corpus`.

Google's public MCP reference confirms that `/mcp/retrieval` and `/mcp/ragdata`
are separate toolsets.[^1] This separation is useful: ordinary agents should
receive the retrieval endpoint only. The data endpoint contains mutations,
including permanent corpus deletion, and belongs on an administrator/worker
path if it is exposed at all.

Do not confuse this with the Agent Retrieval remote MCP server at
`https://vectorsearch.googleapis.com/mcp`. That endpoint is currently Preview,
uses a new stateless MCP 2026-07-28 header-routing convention, and works against
Agent Retrieval collections/data objects rather than RAG corpora.[^3] It can be
useful when Astound owns a custom object schema and ingestion pipeline. It is
not the simplest way to consume the managed document RAG pipeline.

## How Astound can consume it

### Option 1 — Direct Google MCP, per-user OAuth

Register the regional retrieval toolset as an external MCP server and let every
Astound user connect a Google account. Astound's current generic connector can
bank and refresh per-user OAuth grants, because it supports a configured client
ID/secret, PKCE, discovery, encrypted grant storage, and an outbound bearer
accessor. Google publishes MCP protected-resource metadata for the regional and
global Agent Platform resources and documents OAuth client credentials as a
supported third-party authentication method.[^4]

An indicative target descriptor is shown below. It is **not deployable unchanged
with the current generic connector**; the compatibility fixes immediately after
the example are required.

```yaml
mcp_servers:
  google_rag:
    type: external
    binary: ""
    package: null
    port: 5051                 # nominal for an external server
    endpoint: "https://europe-west3-aiplatform.googleapis.com/mcp/retrieval"
    enabled: true
    display_in_web: false
    removable: true
    description: "Google Cloud RAG Engine retrieval"
    oauth:
      required: false          # Astound inbound auth is enforced at its gateway
      scopes: [user]
      audience: mcp
      client_id: null
    connector:
      adapter: generic
      scopes:
        - "https://www.googleapis.com/auth/aiplatform"
      authorization_origins:
        - "https://accounts.google.com"
        - "https://oauth2.googleapis.com"
      client_id_secret: google_rag_mcp_client_id
      client_secret: google_rag_mcp_client_secret
```

Two concrete OAuth compatibility gaps exist in the current repo:

1. `generic_discovery.rs` requires the issuer in protected-resource metadata to
   equal the issuer in authorization-server/OpenID metadata as an exact string.
   Google's live MCP resource metadata returns `https://accounts.google.com/`,
   while Google's OpenID metadata uses `https://accounts.google.com`. Those are
   the same origin but fail the current exact comparison.[^21]
2. `generic.rs` does not add Google's `access_type=offline` authorization
   parameter. Google says a server-side application must request offline access
   to receive the refresh token needed after the short-lived access token
   expires.[^22]

Before using per-user Google OAuth, normalise issuer URLs according to OAuth
issuer comparison rules without weakening origin/path validation, add an
allowlisted connector mechanism for provider-specific authorization parameters,
and test initial consent, re-consent and refresh. A small Google-specific
connector adapter is preferable if those parameters cannot be represented
safely in generic configuration.

It would also need to be included by `services/config/config.yaml`, granted in
the access-control configuration, and attached to the appropriate plugin and
marketplace. The Google OAuth client would allowlist:

```text
https://SP-DEV-HOST/api/public/connectors/google_rag/callback
```

This path has an important operational prerequisite: each connecting Google
principal must have both the MCP-call permission and RAG query permission in
the corpus project. Google documents `roles/mcp.toolUser` plus
`roles/aiplatform.user` for the Agent Platform MCP server; the narrower custom
role would need at least `mcp.tools.call` and `aiplatform.ragCorpora.query` if
Google accepts that role combination for this endpoint.[^5]

Advantages:

- virtually no new retrieval code;
- Google attributes and audits calls to each Google identity;
- disconnecting the user's Google grant immediately removes their route;
- Astound's existing MCP scope, secret scan, blocklist, rate limit and trace
  remain in front of the call.

Disadvantages:

- every Astound user needs a Google Cloud identity and IAM grants, which may
  not match the ADFS identity model;
- `roles/aiplatform.user` is broader than a purpose-built retrieval role;
- RAG query access is documented at the owning project level, not as a
  document ACL;
- a direct caller controls corpus and filter arguments;
- Google tool names and output are exposed directly to skills, coupling the
  Astound product surface to a managed API that is changing quickly.

Use this only where every authorised user may read every file in the exposed
project/corpus, or for a tightly controlled technical evaluation.

### Option 2 — Direct Google MCP, shared workload identity

Astound can call Google with a service account, workload identity federation,
or an agent identity. Google recommends a separate agent/workload identity for
production because it can be least-privileged and its actions can be isolated
in logs.[^4]

This is operationally easier for users but unsafe as a direct, broadly exposed
MCP connection: every Astound caller inherits the same Google corpus access.
Astound may restrict which users can call the `google_rag` server, but the
current policy chain does not rewrite or constrain the corpus and metadata
arguments inside `retrieve_contexts`.

The repo also does not currently have a generic service-account/ADC token
accessor for external MCP resources. Static YAML headers are not suitable:
OAuth access tokens expire and credentials must not be committed. Supporting
this option therefore requires a small token broker or adapter anyway.

### Option 3 — Thin Astound MCP facade over RAG Engine (recommended)

Keep the external contract, identity boundary, and audit semantics in Astound,
but delegate retrieval mechanics to Google:

```text
Claude / Codex / other host
        |
        | Astound credential
        v
Systemprompt MCP gateway
  scope -> secret scan -> blocklist -> rate limit -> audit trace
        |
        v
Astound knowledge facade
  caller -> allowed tenant/project/source -> fixed corpus/filter
        |
        | short-lived Google workload token
        v
europe-west3 RAG Engine RetrieveContexts
        |
        v
normalised chunks + source URI + page span + score
```

The facade can call the regional REST `:retrieveContexts` API or Google's
`/mcp/retrieval` endpoint. REST is preferable inside the facade: only one
operation is needed, its request/response contract is explicit, and it avoids
nested MCP session/error translation. Google documents the REST operation and
returns original Cloud Storage or Drive source URIs with retrieved chunks.[^6]

Recommended public tool contract:

```json
{
  "name": "search_knowledge",
  "arguments": {
    "query": "What is the launch message?",
    "project_ids": ["astound-marketing"],
    "source_types": ["google_drive"],
    "limit": 8
  }
}
```

The caller should never provide a GCP project number, location, corpus resource
name, raw metadata filter, ranker model, or arbitrary file ID. Those values are
resolved from server-owned configuration after Astound authorisation.

Suggested server-owned mapping:

```yaml
google_rag:
  location: europe-west3
  project_secret: google_rag_project_id
  workload_identity_audience_secret: google_rag_wif_audience
  corpora:
    astound_marketing:
      corpus_secret: google_rag_astound_marketing_corpus
      allowed_projects: [astound-marketing]
      source_types: [google_drive]
      max_top_k: 20
```

The facade should clamp `topK`, set fixed thresholds/ranking policy, reject
unknown scopes, and return citations such as:

```json
{
  "results": [
    {
      "text": "...",
      "score": 0.14,
      "title": "Launch brief",
      "uri": "https://drive.google.com/...",
      "pages": {"first": 3, "last": 4},
      "source": "google_drive"
    }
  ],
  "retrieval": {
    "backend": "gcp_rag_engine",
    "corpus_alias": "astound_marketing",
    "location": "europe-west3"
  }
}
```

This preserves the skill-facing `search_knowledge` abstraction and makes a
later backend change possible without rewriting every skill.

### Option 4 — Replace the whole knowledge server with raw Google tools

This is the smallest code change but the largest product/security regression.
It should be rejected for production unless the knowledge base has a single
readership and Astound intentionally abandons its existing knowledge contract.

The direct replacement loses or changes:

| Astound knowledge capability | Google retrieval MCP | Consequence |
|---|---|---|
| `search_knowledge` with Astound project/source semantics | `retrieve_contexts` with raw GCP resource names and caller-supplied filters | skills and prompts must become GCP-aware |
| document grants joined before ranking | project/IAM access plus optional caller-supplied metadata filter | no equivalent proof of per-document non-disclosure |
| `list_knowledge_sources` | no equivalent on retrieval toolset | build from server-owned config or RAG file APIs |
| `sync_knowledge` | `import_rag_files` on separate mutation endpoint | do not expose to ordinary agents; schedule through a worker |
| `knowledge_sync_status` | `get_operation` only for known operations | needs durable local job state and list/import-result inspection |
| `knowledge_index_stats` | no direct equivalent | use Cloud Monitoring/Billing plus corpus/file inventory |
| `prune_knowledge` and grant expiry | corpus deletion exists; file lifecycle requires REST/API work | a separate reconciliation job is required |
| stable citations | source URI, display name and page span are returned | good fit, but response normalisation remains useful |
| air-gapped operation | cloud dependency | feature is unavailable when disconnected from GCP |

## Identity and permission model

### Google IAM is necessary but not a document entitlement system

The Google MCP endpoint authenticates with OAuth 2.0 and IAM. Google supports
user identities, service accounts and agent/workload identities.[^4] For
`retrieve_contexts`, Google explicitly instructs callers to grant
`roles/aiplatform.user` on the project that owns the corpus; the permission is
`aiplatform.ragCorpora.query`.[^5]

Google's general Agent Platform IAM documentation says most resources are
controlled at project/folder/organisation level and lists only specific
resource types as supporting resource-level IAM. RAG corpora are not in that
document's resource-level list.[^7] Accordingly, the safe planning assumption
is that IAM is the coarse corpus-project boundary, not a replacement for
Astound's per-document grants.

RAG Engine has schema-based file metadata and a CEL `metadata_filter`, but this
feature is currently `v1beta1`, is unavailable in Serverless mode, and the
filter is supplied in the retrieval request.[^8] Metadata can implement
tenant/project segmentation only when trusted server code constructs the
filter. It is not an access-control boundary when the LLM or end user can edit
or omit it.

### Recommended tenancy patterns

From strongest isolation to lowest operational overhead:

1. **GCP project per hard tenant/security boundary.** Strong IAM, quota,
   billing and blast-radius separation. Highest provisioning overhead.
2. **Corpus per audience in one project, accessed only through the Astound
   facade.** Good operational compromise. The facade allowlists corpus IDs;
   users do not receive direct Google query credentials.
3. **One corpus plus server-enforced metadata filters.** Efficient, but metadata
   search requires Spanner mode and the Preview `v1beta1` feature. Requires
   strong negative tests proving a caller cannot alter the filter.
4. **One corpus plus prompt-instructed filters.** Not an access-control design.

For Astound's current source model, use a corpus per organisation or per
readership cohort for folder-style shared material. Do not copy arbitrary Jira,
Confluence, or Salesforce content into a shared corpus until its source ACL
semantics have a tested mapping. The repo's current design deliberately reads
Atlassian as the caller and treats Salesforce records as live queries because
their permission models are not faithfully represented by flat document
grants.[^9]

## Ingestion and lifecycle

RAG Engine can import local uploads, Cloud Storage, Google Drive, Slack, Jira
and SharePoint. For Drive, Google instructs operators to share the source with
the Google-managed RAG Data Service Agent. It deduplicates unchanged files
using path, filename, content/version information and chunking configuration.[^10]

That removes much of Astound's planned Drive extraction stack, including local
conversion, chunking, embedding and vector persistence. It does not remove the
need for a control-plane worker responsible for:

- source-to-corpus configuration;
- scheduled re-import/reconciliation;
- durable operation IDs and status;
- import result sinks in Cloud Storage or BigQuery;
- detecting and resolving unsupported or silently skipped files;
- deletion/tombstone propagation;
- attaching metadata when server-enforced filtering is used;
- corpus inventory and cost/quality telemetry.

The `/mcp/ragdata` endpoint is deliberately incomplete as a lifecycle API: it
can import files and delete whole corpora, but it does not advertise list/delete
individual file tools. Astound should use the RAG REST/SDK APIs from a scheduled
worker for full lifecycle management, and should not give the mutation MCP
toolset to ordinary agents.

### Parsing and formats

The default parser supports Google Docs/Drawings/Slides, HTML, JSON/JSONL,
Markdown, PPTX, DOCX, PDF and plain text, with documented limits from 10 MB to
50 MB depending on type.[^11] The Document AI layout parser understands tables,
lists and layout-aware chunks, but its RAG import path is limited to 20 MB and
500 PDF pages and carries Document AI pricing.[^12] The LLM parser supports
PDF and common image formats and incurs Gemini token charges.[^13]

This is not feature-equivalent to the repo's documented Drive pipeline, which
also describes CSV/XML, spreadsheet formulas/notes, comments, audio/video and
explicit unsupported-file inventory.[^14] A representative corpus test must
measure the actual formats Astound needs rather than infer coverage from the
word "managed."

## Retrieval quality and citations

RAG Engine supports fixed-size chunking, configurable embedding models,
distance/similarity thresholds, metadata filters and optional reranking. Its
Ranking API reranker is documented as under 100 ms; its LLM reranker is
documented as roughly 1–2 seconds and is charged as model usage.[^15]

The raw retrieval result includes chunk text, a relevance score, the original
source URI/display name, file/chunk IDs and page spans where available.[^6]
That is sufficient for Astound to return evidence with deep links and page
references. It does not by itself establish answer correctness. The existing
repo already identifies a precision@5 target above 70% and says it is blocked
until a real corpus and agreed question set exist.[^9]

The migration should therefore be evaluated as a retriever, not by asking a
model whether answers "look good." Build a labelled test set containing:

- common questions with one canonical answer;
- ambiguous questions requiring multiple sources;
- no-answer questions;
- stale/deleted documents;
- near-duplicate and contradictory versions;
- tables, scans and images;
- queries from two users with different permissions;
- prompt-injection text embedded inside a document.

Measure recall@5/10, precision@5, mean reciprocal rank, citation validity,
permission leakage, no-answer behaviour, p50/p95 latency and cost per verified
answer. Compare identical queries against the current fixture-backed engine or
a fixed baseline. Do not switch production traffic until the permission-leakage
suite is zero-failure and the quality target is met.

## Security, governance and compliance

### What remains governed by Astound

When the endpoint is registered behind Systemprompt, the call still passes
Astound's synchronous scope check, configured secret scan, blocklist and rate
limit, and receives Astound's user/actor/trace attribution. This is valuable,
but it controls the call envelope. It does not automatically constrain GCP
resource identifiers inside the arguments.

The recommended facade adds the missing semantic authorisation boundary and
should log, without logging retrieved confidential text:

- Astound user/actor/trace ID;
- corpus alias (not necessarily the raw secret resource name);
- resolved project/source scope;
- query hash and safe query classification;
- top-k, threshold and reranker policy;
- returned document/chunk IDs and scores;
- Google request ID, latency and error class;
- estimated or reconciled cost.

### Google-side controls

Google documents IAM-based MCP access, central audit logging, optional Model
Armor and IAM deny policies for Google Cloud MCP servers.[^16] Model Armor can
scan prompts and responses, but Google warns that enabling logging can record
entire payloads and expose sensitive information in logs.[^5] Treat it as a
second scanning plane, configure regional compatibility, and make payload
logging an explicit data-protection decision.

RAG Engine documents VPC Service Controls and CMEK support, but also explicitly
states that data-residency and AXT security controls are not supported.[^17]
Serverless mode does not support CMEK; Spanner mode does.[^18] A regional
Frankfurt endpoint is therefore a placement/routing choice, not evidence of a
contractual data-residency control. Legal/security review should resolve that
before regulated client content is ingested.

The managed service also removes Astound's air-gap capability for this feature.
The wider binary can remain self-hosted, but knowledge retrieval now depends on
Google availability, IAM, network egress and quota.

## Capacity, reliability and cost

RAG Engine currently documents 60 data-management requests per minute per
region, 600 synchronous retrieval requests per minute per region, three
concurrent imports, and up to 10,000 files in an import request. Async retrieval
and `ask_contexts` are documented at 60 RPM.[^19] Those are starting quotas,
not an availability guarantee; load tests and quota-increase lead time belong
in the rollout plan.

Billing is component-based:

- access/import orchestration is free, though source transfer can cost money;
- the default parser and fixed-size chunking are free;
- embedding model usage is charged;
- LLM or Document AI parsing is charged when selected;
- vector storage/indexing is charged according to the selected backend/mode;
- optional LLM or Ranking API reranking is charged.[^20]

In Spanner mode, `RagManagedDb` Basic provisions 100 processing units with
backup; Scaled starts at one node/1,000 processing units and can autoscale to
ten nodes. If any corpus uses the RAG-managed database for vector search, the
managed Spanner cost is surfaced to the customer project.[^20] Serverless mode
instead provisions managed Agent Retrieval/Vector Search in the customer
project and offers autoscaling, but it cannot use metadata search or CMEK.[^18]

Do not approve the migration from list prices alone. Run a two-week pilot with
budgets and billing export enabled, then model:

```text
monthly cost = vector storage/index serving
             + initial and changed-document embeddings
             + optional parser calls
             + retrieval query embeddings/search
             + optional reranking
             + logging/monitoring and source egress
```

Compare that total with the real cost of the current dedicated Postgres,
Bedrock embeddings/reranking, extraction services and operational support.

## Recommended implementation plan

### Phase 0 — platform spike (2–3 days)

1. Create a non-production GCP project and budget alerts.
2. Enable Agent Platform/RAG Engine and the relevant embedding/parser APIs.
3. Use `europe-west3` and create one evaluation corpus.
4. Import 50–200 representative, non-sensitive documents from a controlled
   Cloud Storage bucket or Drive folder.
5. Validate REST `RetrieveContexts` and both MCP `tools/list`/tool calls.
6. Capture exact endpoint protocol, OAuth scopes, response schema, latency,
   quotas and billing SKUs observed in the project.

Exit: retrieved chunks contain usable original citations and the chosen parser
handles the representative formats.

### Phase 1 — direct MCP compatibility spike (2–4 days)

1. Fix and test issuer normalisation plus Google's offline-access parameter.
2. Add an evaluation-only external MCP descriptor using Astound's generic
   OAuth connector.
3. Register a Google web OAuth client with the Astound callback.
4. Grant only test identities and a test project.
5. Verify token refresh, disconnect, MCP initialisation, tool discovery and
   `retrieve_contexts` through the Systemprompt gateway.
6. Repeat authenticated protocol negotiation. A public, unauthenticated
   handshake against the live regional endpoint accepted Astound core's pinned
   2026-07-28 version on 14 September 2026, although Google's general Cloud MCP
   documentation currently describes 2025-11-25 support.[^16]

Exit: the connection survives expiry/refresh and every tool call has both an
Astound trace and a Google audit event.

### Phase 2 — permission-safe facade (1–2 weeks)

1. Implement only `search_knowledge` and `list_knowledge_sources` first.
2. Use workload identity federation or attached workload credentials; avoid a
   downloadable service-account key.
3. Resolve caller-to-corpus aliases in trusted code.
4. Clamp retrieval settings and normalise citations.
5. Add retry/backoff for 429/5xx with an overall request deadline.
6. Add leakage tests that attempt raw corpus IDs, unknown projects, omitted
   filters and injected CEL expressions.
7. Keep the Google `/mcp/ragdata` endpoint out of user marketplaces.

Exit: no caller-controlled path can select a corpus or document outside its
Astound entitlement.

### Phase 3 — managed ingestion worker (1–2 weeks)

1. Map approved sources to corpora and parser profiles.
2. Schedule imports using the existing scheduler and explicit real admin owner.
3. Store operation state and import-result locations durably.
4. Implement deletion reconciliation and inventory/status endpoints.
5. Add alerts for failed imports, quota exhaustion, stale sources and cost.

Exit: add/change/delete flows converge correctly and operators can explain
what was indexed, skipped, failed and billed.

### Phase 4 — quality and production decision (1–2 weeks)

1. Run the labelled retrieval suite and permission matrix.
2. Tune chunking, embeddings, thresholds and reranking one variable at a time.
3. Compare cost/latency/quality with the current baseline.
4. Complete data protection, residency, retention and incident reviews.
5. Roll out by corpus/audience with an instant feature-flag fallback.

Exit: zero leakage failures, agreed retrieval target met, p95 within the product
SLO, budget accepted and deletion/revocation behaviour demonstrated.

## Decision matrix

| Criterion | Direct Google MCP | Thin Astound facade + Google RAG | Current/self-hosted knowledge engine |
|---|---:|---:|---:|
| Fastest proof of concept | Excellent | Good | Poor |
| Managed parsing/index/search | Excellent | Excellent | Poor |
| Stable Astound tool contract | Poor | Excellent | Excellent |
| Per-document Astound authorisation | Poor | Good, if enforced by corpus/filter mapping | Excellent by design |
| Per-user Google audit identity | Excellent with user OAuth | Shared workload identity unless impersonation is added | N/A |
| Minimal end-user account setup | Poor with user OAuth | Excellent | Excellent |
| Ingestion/status/prune parity | Poor | Good after worker implementation | Excellent in the documented design |
| Air-gap support | None | None for knowledge | Excellent |
| Vendor portability | Poor | Moderate | High |
| Operational database burden | Low | Low | High |
| Current service maturity risk | Medium: MCP surface is new | Medium, isolated behind facade | Internal engine remains unvalidated on real content |

## Go/no-go recommendation

Proceed with a bounded Google RAG pilot. Do not replace Astound's knowledge MCP
contract with Google's raw retrieval endpoint in production.

Choose the thin facade if the pilot proves all of the following:

- representative formats parse with acceptable fidelity;
- returned source URIs/page spans are valid citations;
- a tested corpus-per-audience or server-enforced metadata design matches the
  actual client permission model;
- Frankfurt processing plus Google's stated control limitations pass the data
  protection review;
- quality meets the agreed benchmark;
- observed cost and p95 latency beat the operational value of the current
  self-hosted path.

Keep the current approach, or narrow the Google corpus to universally readable
material, if document-level source entitlements cannot be represented safely.
No retrieval-quality improvement compensates for cross-client disclosure.

## Sources

[^1]: Google Cloud, “[MCP Reference: aiplatform.googleapis.com](https://docs.cloud.google.com/gemini-enterprise-agent-platform/reference/mcp),” accessed 14 September 2026.
[^2]: Google Cloud, “[RAG Engine overview — supported regions](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/rag-engine/rag-overview),” and “[Google Cloud MCP supported products](https://docs.cloud.google.com/mcp/supported-products),” accessed 14 September 2026.
[^3]: Google Cloud, “[Use the Agent Retrieval remote MCP server](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/vector-search-2/use-agent-retrieval-mcp),” updated 8 September 2026.
[^4]: Google Cloud, “[Configure MCP in an AI application](https://docs.cloud.google.com/mcp/configure-mcp-ai-application),” accessed 14 September 2026.
[^5]: Google Cloud, “[Use the Agent Platform remote MCP server](https://docs.cloud.google.com/gemini-enterprise-agent-platform/reference/use-agent-platform-mcp),” accessed 14 September 2026.
[^6]: Google Cloud, “[MCP tool: retrieve_contexts](https://docs.cloud.google.com/gemini-enterprise-agent-platform/reference/mcp/tools_list/retrieve_contexts),” accessed 14 September 2026.
[^7]: Google Cloud, “[Agent Platform access control with IAM](https://docs.cloud.google.com/gemini-enterprise-agent-platform/machine-learning/general/access-control),” accessed 14 September 2026.
[^8]: Google Cloud, “[Filter with metadata search](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/rag-engine/use-metadata-search),” accessed 14 September 2026.
[^9]: Astound Digital, “[Enterprise Knowledge & RAG](../../services/content/documentation/enterprise-knowledge-rag.md),” updated 11 September 2026.
[^10]: Google Cloud, “[Use data ingestion with RAG Engine](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/rag-engine/use-data-ingestion),” accessed 14 September 2026.
[^11]: Google Cloud, “[Document types for RAG Engine](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/rag-engine/supported-documents),” updated 24 August 2026.
[^12]: Google Cloud, “[Use Document AI layout parser with RAG Engine](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/rag-engine/layout-parser-integration),” accessed 14 September 2026.
[^13]: Google Cloud, “[Use the LLM parser](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/rag-engine/llm-parser),” accessed 14 September 2026.
[^14]: Astound Digital, “[Google Drive to Astound knowledge bank](../install/google-drive-knowledge.md),” accessed 14 September 2026.
[^15]: Google Cloud, “[Reranking for RAG Engine](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/rag-engine/retrieval-and-ranking),” accessed 14 September 2026.
[^16]: Google Cloud, “[Google Cloud MCP servers overview](https://docs.cloud.google.com/mcp/overview),” accessed 14 September 2026.
[^17]: Google Cloud, “[Vector database choices in RAG Engine](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/rag-engine/vector-db-choices),” updated 3 September 2026.
[^18]: Google Cloud, “[Deployment modes in RAG Engine](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/rag-engine/deployment-modes),” accessed 14 September 2026.
[^19]: Google Cloud, “[Gemini Enterprise Agent Platform quotas and limits — RAG Engine](https://docs.cloud.google.com/gemini-enterprise-agent-platform/machine-learning/quotas#rag_engine_quotas),” accessed 14 September 2026.
[^20]: Google Cloud, “[RAG Engine billing](https://docs.cloud.google.com/gemini-enterprise-agent-platform/build/rag-engine/rag-engine-billing),” updated 3 September 2026.
[^21]: Google Cloud, “[Agent Platform MCP protected-resource metadata](https://europe-west3-aiplatform.googleapis.com/.well-known/oauth-protected-resource/mcp/retrieval),” and “[Google OpenID Connect API reference](https://developers.google.com/identity/openid-connect/reference),” accessed 14 September 2026; Astound Digital, [`generic_discovery.rs`](../../extensions/web/admin/src/services/connector_oauth/generic_discovery.rs), accessed 14 September 2026.
[^22]: Google for Developers, “[Using OAuth 2.0 for Web Server Applications](https://developers.google.com/identity/protocols/oauth2/web-server#offline),” accessed 14 September 2026; Astound Digital, [`generic.rs`](../../extensions/web/admin/src/services/connector_oauth/generic.rs), accessed 14 September 2026.
