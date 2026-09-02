# project-context — architecture & Phase 0 requirements pack

This pack describes the **as-is architecture** of the Node `project-context`
RAG engine (part of the Customer's **"AI Kit"** solution) and states the
**Phase 0 requirements** for re-platforming that capability onto systemprompt.

## Start here

- **[`PHASE0-REQUIREMENTS.md`](PHASE0-REQUIREMENTS.md)** — customer→vendor
  requirements for Phase 0 (project-context on systemprompt). The primary
  document; the architecture docs below are its substantiating baseline.
- **[`00-system-overview.md`](00-system-overview.md)** — whole-system narrative
  and dataflow diagrams. Read this first for the technical picture.

## Component reference (as-is)

Nine component docs, foundation-first then write-path → read-path:

| # | Doc | Component |
|---|---|---|
| 1 | [`c1-store.md`](c1-store.md) | Store (vector + full-text index) |
| 2 | [`c2-model.md`](c2-model.md) | Model access (provider connector) |
| 3 | [`c3-platform.md`](c3-platform.md) | Platform / core (config, env, preflight) |
| 4 | [`c4-ingestion.md`](c4-ingestion.md) | Ingestion & sources (Atlassian, parsers, normalizer) |
| 5 | [`c5-chunking.md`](c5-chunking.md) | Chunking strategies |
| 6 | [`c6-indexer.md`](c6-indexer.md) | Indexer / orchestration (incl. dating) |
| 7 | [`c7-retrieval.md`](c7-retrieval.md) | Retrieval (hybrid search + rerank) |
| 8 | [`c8-mcp.md`](c8-mcp.md) | Tool interface (MCP) |
| 9 | [`c9-policy.md`](c9-policy.md) | Agent usage policy (consumer) |

## Traceability

Requirement groups in `PHASE0-REQUIREMENTS.md` map to the baseline docs:
FR-ING→c4/c6 · FR-PROC→c5/c6 · FR-MODEL→c2 · FR-RET→c7 · FR-ACC→c8/c9 ·
NFR-SELF/PERF→c7 · NFR-COST→c6.

Requirement → implementation → test, for the delivered crate:
[`TRACEABILITY.md`](TRACEABILITY.md). A unit test
(`tests/unit/mcp/src/phase0_traceability.rs`) fails `just test-unit` when a
requirement id in `PHASE0-REQUIREMENTS.md` has no row, or a row cites a test
that does not exist.

## Reference copy of the Node engine

The as-is source and its inbox corpus are vendored, gitignored, at
`.reference/project-context/` (from
`github.com/Astound-Digital/sfcc-next-cursor/.project/project-context`). The
inbox holds customer meeting transcripts, which is why it is not committed.
