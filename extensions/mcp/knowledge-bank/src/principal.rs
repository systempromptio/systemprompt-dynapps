//! The calling identity and grant subjects, shared with the scheduler.

// Why: the module now lives in mcp/shared; this crate keeps its public paths so
// callers and tests need no change.
pub use systemprompt_mcp_shared::knowledge::principal::*; // lint-ok: glob-reexport
