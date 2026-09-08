//! Knowledge-bank configuration, re-exported from the shared tier so the
//! scheduler's Drive job and this server read one model.

// Why: the module now lives in mcp/shared; this crate keeps its public paths so
// callers and tests need no change.
pub use systemprompt_mcp_shared::knowledge::config::*; // lint-ok: glob-reexport
