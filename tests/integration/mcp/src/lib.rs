//! Integration coverage for `systemprompt-mcp-shared`'s audit writers against a
//! live Postgres, focused on the fail-closed rejection path
//! (`record_mcp_access_rejected`): a rejection audit row is written only when a
//! reserved anonymous principal exists to attribute it to, and is silently
//! dropped (no row, no error) otherwise.
//!
//! Also covers the knowledge bank's Bedrock residency guard (always) and its
//! live reachability (only when `KNOWLEDGE_BANK_BEDROCK_KEY` is set), plus its
//! REQ-052 access control: that the
//! `knowledge_document_grants` join actually withholds documents from callers
//! with no grant, and that revoking one takes effect on the next call.
//!
//! Every test runs against its OWN throwaway database created on the live
//! server named by `DATABASE_URL`, so the shared application tables are never
//! read, written, or truncated. The database is dropped on completion.

#[cfg(test)]
mod bedrock_live;
#[cfg(test)]
mod common;
#[cfg(test)]
mod knowledge_bank_acl;
#[cfg(test)]
mod knowledge_benchmark;
#[cfg(test)]
mod record_access;
#[cfg(test)]
mod rejection_fail_closed;
