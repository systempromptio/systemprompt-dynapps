//! Integration coverage for `systemprompt-mcp-shared`'s audit writers against a
//! live Postgres, focused on the fail-closed rejection path
//! (`record_mcp_access_rejected`): a rejection audit row is written only when a
//! reserved anonymous principal exists to attribute it to, and is silently
//! dropped (no row, no error) otherwise.
//!
//! Also covers the knowledge bank: its Bedrock residency guard (always) and
//! live reachability (only when `KNOWLEDGE_BANK_BEDROCK_KEY` is set); its
//! REQ-052 access control, proving the `knowledge_document_grants` join
//! withholds documents from callers without a grant and that revocation takes
//! effect on the next call; the shared-index write path through an in-memory
//! connector with a fake model — per-user grants, one document for two
//! readers, zero re-embeds, grant removal on disappearance, orphan deletion,
//! pruning, incremental skip, archive opt-in, and the degradation contract
//! when the reranker fails; and the pgvector boot check against a plain
//! Postgres (only when `KNOWLEDGE_BANK_PLAIN_DATABASE_URL` is set).
//!
//! Every test runs against its OWN throwaway database created on the live
//! server named by `DATABASE_URL`, so the shared application tables are never
//! read, written, or truncated. The database is dropped on completion.

#[cfg(test)]
mod bedrock_live;
#[cfg(test)]
mod common;
#[cfg(test)]
mod fake_model;
#[cfg(test)]
mod knowledge_bank_acl;
#[cfg(test)]
mod knowledge_benchmark;
#[cfg(test)]
mod knowledge_ingest;
#[cfg(test)]
mod knowledge_live;
#[cfg(test)]
mod record_access;
#[cfg(test)]
mod rejection_fail_closed;
