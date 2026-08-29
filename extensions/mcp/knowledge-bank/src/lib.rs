//! MCP server crate for the Astound knowledge bank (REQ-047/048).
//!
//! Exposes enterprise knowledge retrieval as MCP tools (`search_knowledge`,
//! `list_knowledge_sources`, `knowledge_index_stats`) over a pluggable
//! backend: [`retriever`] defines the [`KnowledgeRetriever`] trait, and the
//! server holds a trait object so the production backend drops in without
//! touching the tool surface. That backend is a MIGRATION of the Node.js
//! "project-context" system (sfcc-next-cursor/.project/project-context),
//! whose modules map onto this crate as follows:
//!
//! - `scripts/mcp/server.mjs` (stdio MCP server) → [`server`] + [`tools`] here
//!   — already ported as the tool contract (search / `list_sources` /
//!   `index_stats`), served over this instance's authenticated HTTP transport
//!   instead of stdio.
//! - `scripts/retrieval/service.mjs` + `store.mjs` (hybrid search: Titan v2
//!   embeddings + text match, `source_type` pre-filter, candidate over-fetch,
//!   Cohere rerank, per-call index `.version` re-check) → [`retrieval`], which
//!   implements [`KnowledgeRetriever`] over pgvector in the instance's own
//!   Postgres. pgvector was chosen over a ported `LanceDB` directory so the
//!   REQ-052 grant join runs inside the candidate queries rather than as a
//!   post-filter above them.
//! - `scripts/ingest/*` (atlassian.mjs, parsers, normalizer, chunker with
//!   per-source llm | script | none strategies) plus
//!   `scripts/cli/{index,sync}.mjs` → [`ingest`]. Ingestion is NOT part of the
//!   serving path and stays out of the trait. It also carries what the source
//!   system has no notion of: reading each document's source-system permissions
//!   and writing them as grants (REQ-048), which is what makes REQ-052
//!   enforceable at query time.
//! - `core/config` + env (bedrock models, chunking targets, retrieval tunables,
//!   sources map) → server-side config for this crate.
//!
//! [`StubRetriever`] remains the backend when no Bedrock key is provisioned:
//! it reports honestly that no backend is configured rather than fabricating
//! results. Errors normalise on [`error::KnowledgeBankError`]; the `main`
//! binary mirrors the `systemprompt` MCP agent's HTTP serving shell. Full
//! migration map: `docs/tickets/req-047-knowledge-bank-mcp.md`.

pub mod bedrock;
pub mod config;
pub mod error;
pub mod ingest;
pub mod principal;
pub mod retrieval;
pub mod retriever;
pub mod server;
pub mod tools;

pub use bedrock::{BedrockClient, assert_eu_region};
pub use config::KnowledgeBankConfig;
pub use principal::{Principal, Subject};
pub use retrieval::{PgRetriever, pgvector_literal};
pub use retriever::{
    IndexStats, KnowledgeHit, KnowledgeRetriever, KnowledgeSource, SearchFilter, StubRetriever,
};
pub use server::KnowledgeBankServer;
