//! Shared-source processing status, re-exported from the shared tier.

// Why: the module now lives in mcp/shared; this crate keeps its public paths so
// callers and tests need no change.
pub use systemprompt_mcp_shared::knowledge::drive::status::*; // lint-ok: glob-reexport
