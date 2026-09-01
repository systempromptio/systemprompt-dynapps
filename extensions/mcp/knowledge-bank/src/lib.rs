//! MCP server crate for the Astound knowledge bank — the project-context RAG
//! engine (REQ-046/047/048/052, Phase 0 of the AI Kit migration).
//!
//! The crate is a migration of the Node.js "project-context" system
//! (sfcc-next-cursor/.project/project-context) onto this platform, reshaped
//! for a remote server with one shared index:
//!
//! - [`downstream`] reads the caller's own Atlassian credentials from the
//!   request headers; nothing else authenticates to Atlassian.
//! - [`ingest`] is the write path — the Node `sync`/`index` CLIs, normaliser,
//!   per-source chunking strategies, enrichment and dating — run in-process
//!   when a user calls `sync_knowledge`. It reads Confluence and Jira as that
//!   user and grants every document it stores to them, which is what makes
//!   REQ-052 enforceable at query time without translating anyone's
//!   permissions.
//! - [`retrieval`] is the read path — hybrid search over pgvector with
//!   reciprocal-rank fusion and reranking — implementing the
//!   [`KnowledgeRetriever`] seam the server is built against, with the grant
//!   join inside the candidate queries.
//! - [`maintenance`] keeps the shared index honest: stale users' grants, orphan
//!   documents and unreferenced cache rows are pruned by administrators.
//! - [`model`] is the single boundary to a model provider; Bedrock is the only
//!   implementation, as in the source system, behind a trait so a swap is
//!   configuration plus one implementation.
//! - [`schema`] owns the tables in the knowledge database and requires
//!   pgvector: a database without it fails the boot, by design.
//!
//! Full migration map and decisions:
//! `docs/tickets/req-047-knowledge-bank-mcp.md`,
//! `docs/decisions/2026-09-01-knowledge-bank-remote-shared-index.md`.

pub mod config;
pub mod downstream;
pub mod error;
pub mod ingest;
pub mod maintenance;
pub mod model;
pub mod principal;
pub mod retrieval;
pub mod retriever;
pub mod schema;
pub mod server;
pub mod tools;

pub use config::KnowledgeBankConfig;
pub use downstream::AtlassianCredentials;
pub use ingest::{IngestReport, Ingestor, RunOptions};
pub use maintenance::{PruneOptions, PruneReport, Pruner};
pub use model::{BedrockProvider, ModelProvider, RerankResult, assert_eu_region};
pub use principal::{Principal, Subject};
pub use retrieval::{PgRetriever, pgvector_literal};
pub use retriever::{IndexStats, KnowledgeHit, KnowledgeRetriever, KnowledgeSource, SearchFilter};
pub use server::{KnowledgeBackend, KnowledgeBankServer};
