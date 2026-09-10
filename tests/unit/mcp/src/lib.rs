//! Unit tests for the MCP extension crates' pure helpers:
//! - `systemprompt-mcp-agent`'s `filter_hallucinated_args` (CLI arg scrubbing)
//! - `systemprompt-mcp-shared`'s `truncate_on_char_boundary` (rejection-reason
//!   truncation with UTF-8 safety) and `AuditMetadata`'s stored JSON shape
//! - `systemprompt-mcp-agent`'s `systemprompt` tool contract (CLI and reporting
//!   tools, their input/output schema) and its error type's code / status /
//!   retryability
//! - the admin reporting tools' report and project-catalog shapes

#[cfg(test)]
mod audit_metadata;
#[cfg(test)]
mod filter_hallucinated_args;
#[cfg(test)]
mod systemprompt_error;
#[cfg(test)]
mod systemprompt_tools;
#[cfg(test)]
mod truncate_on_char_boundary;

#[cfg(test)]
mod admin_reports;

#[cfg(test)]
mod admin_project_reports;
