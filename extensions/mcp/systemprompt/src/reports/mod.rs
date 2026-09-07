//! Deterministic admin reports shared by MCP Apps and Cowork library pages.

mod atlassian;
mod handler;
mod platform;
mod projects;
mod renderer;
mod shape;

pub use atlassian::Atlassian;
pub use handler::{CatalogHandler, ReportHandler};
pub use platform::normalize_cli;
pub use projects::{project_metrics, report as project_report};
pub use renderer::admin_artifact_shell;
pub use shape::{CatalogInput, ReportInput, ReportOutput, ReportTable, SourceStatus};

pub(crate) fn invalid(message: impl Into<String>) -> rmcp::ErrorData {
    rmcp::ErrorData::invalid_params(message.into(), None)
}

pub(crate) fn failure(message: impl Into<String>) -> rmcp::ErrorData {
    rmcp::ErrorData::internal_error(message.into(), None)
}
