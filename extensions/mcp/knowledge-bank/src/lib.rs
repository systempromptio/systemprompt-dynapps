//! MCP server crate for the Astound knowledge bank — the project-context RAG
//! engine (REQ-046/047/048/052, Phase 0 of the AI Kit migration).
//!
//! The crate is a migration of the Node.js "project-context" system
//! (sfcc-next-cursor/.project/project-context) onto this platform:
//!
//! - [`ingest`] is the write path — the Node `sync`/`index` CLIs, parsers,
//!   normaliser, per-source chunking strategies, enrichment and dating — run
//!   in-process on a schedule, from the `ingest` subcommand, or from the
//!   admin-only `sync_knowledge` tool. It also carries what the Node system had
//!   no notion of: translating each document's source-system permissions into
//!   grants, which is what makes REQ-052 enforceable at query time.
//! - [`retrieval`] is the read path — hybrid search over pgvector with
//!   reciprocal-rank fusion and reranking — implementing the
//!   [`KnowledgeRetriever`] seam the server is built against, with the grant
//!   join inside the candidate queries.
//! - [`model`] is the single boundary to a model provider; Bedrock is the only
//!   implementation, as in the source system, behind a trait so a swap is
//!   configuration plus one implementation.
//! - [`schema`] owns the tables: the binary probes for pgvector and applies its
//!   own idempotent schema, so a database without the extension leaves the
//!   knowledge bank inert instead of failing the platform's boot.
//!
//! [`StubRetriever`] serves whenever the instance cannot retrieve and reports
//! honestly that no backend is configured rather than fabricating results.
//! Full migration map and decisions:
//! `docs/tickets/req-047-knowledge-bank-mcp.md`.

pub mod cli;
pub mod config;
pub mod error;
pub mod ingest;
pub mod model;
pub mod principal;
pub mod retrieval;
pub mod retriever;
pub mod schema;
pub mod server;
pub mod tools;

pub use config::KnowledgeBankConfig;
pub use ingest::{IngestReport, Ingestor, RunOptions};
pub use model::{BedrockProvider, ModelProvider, RerankResult, assert_eu_region};
pub use principal::{Principal, Subject};
pub use retrieval::{PgRetriever, pgvector_literal};
pub use retriever::{
    IndexStats, KnowledgeHit, KnowledgeRetriever, KnowledgeSource, SearchFilter, StubRetriever,
};
pub use server::KnowledgeBankServer;
