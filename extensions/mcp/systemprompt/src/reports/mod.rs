//! Deterministic reports over this platform's OWN data.
//!
//! The rule this module follows: wrap our own data, never someone else's MCP
//! server. Translating a third party's payloads here means owning its wire
//! shape, and every undocumented envelope or renamed field becomes a wrong
//! number on a dashboard. What stays here is what has no third party in the
//! path: the CLI and the audit tables.
//!
//! The one artifact is `admin-ai-usage`, and it exists because its data is
//! deterministic: our own audit tables, no third party, exact rather than
//! sampled. A dashboard over data we cannot vouch for is worse than no
//! dashboard.

mod handler;
mod platform;
mod renderer;
mod shape;

pub use handler::ReportHandler;
pub use platform::{normalize_cli, with_dollar_siblings};
pub use renderer::admin_artifact_shell;
pub use shape::{ReportInput, ReportKind, ReportOutput, ReportTable, SourceStatus};

pub(crate) use systemprompt_mcp_shared::reply::{failure, invalid};
