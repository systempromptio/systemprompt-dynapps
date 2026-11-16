//! Unit tests for the MCP extension crates' pure helpers:
//! - `systemprompt-mcp-agent`'s `filter_hallucinated_args` (CLI arg scrubbing)
//! - `systemprompt-mcp-shared`'s `truncate_on_char_boundary` (rejection-reason
//!   truncation with UTF-8 safety) and `AuditMetadata`'s stored JSON shape
//! - `systemprompt-mcp-agent`'s `systemprompt` tool contract (CLI and reporting
//!   tools, their input/output schema) and its error type's code / status /
//!   retryability
//! - the admin reporting tools' report and project-catalog shapes
//! - the typed analytics tools' inputs: defaults, clamps, and the exact CLI
//!   argv each builds
//! - the passthrough result bound: the overflow decision, the pointer sent in
//!   place of a stored oversize result, and the trimming fallback
//! - Gemini declarability of every listed tool's input schema
//! - the `agent-farm` server's tool surface, schemas, run authorisation and the
//!   administrator-only `all` scope, and `farm_dashboard`'s JSON shape against
//!   the fields its dashboard artifact reads
//! - the `documents` server's tool surface and its manifest/services yaml
//! - the issue report tools on the `systemprompt` server and their config
//! - `systemprompt-mcp-shared`'s file layer: `FileRef` parsing and schema,
//!   the inline limit, name and MIME checks, and `McpFiles` ownership,
//!   expiry, size limits and audit over an in-memory store
//! - the `microsoft-365` server (`m365_core_*`): value checks ported from the
//!   Belgian server, the Graph client against a simulated Graph (429, 401,
//!   redirects, size limit, paging), `d365_list_mail`, the broker token and
//!   the tool surface against the pinned Belgian fixture

#[cfg(test)]
mod access_policy;
#[cfg(test)]
mod audit_metadata;
#[cfg(test)]
mod files_refs;
#[cfg(test)]
mod files_service;
#[cfg(test)]
mod files_support;
#[cfg(test)]
mod filter_hallucinated_args;
#[cfg(test)]
mod m365_core_graph;
#[cfg(test)]
mod m365_core_mail;
#[cfg(test)]
mod m365_core_paging;
#[cfg(test)]
mod m365_core_sanitize;
#[cfg(test)]
mod m365_core_surface;
#[cfg(test)]
mod m365_support;
#[cfg(test)]
mod gemini_schema;
#[cfg(test)]
mod systemprompt_error;
#[cfg(test)]
mod systemprompt_tools;
#[cfg(test)]
mod truncate_on_char_boundary;

#[cfg(test)]
mod admin_reports;
#[cfg(test)]
mod agent_farm_tools;
#[cfg(test)]
mod bounds;
#[cfg(test)]
mod documents_tools;
#[cfg(test)]
mod farm_dashboard_shape;
#[cfg(test)]
mod issues_tools;
#[cfg(test)]
mod typed_activity;
#[cfg(test)]
mod typed_inputs;
#[cfg(all(test, unix))]
mod typed_users_handler;
