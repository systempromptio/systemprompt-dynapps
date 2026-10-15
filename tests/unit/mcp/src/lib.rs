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
//! - the `odoo` server: JSON-RPC envelope handling, search domains, the
//!   knowledge, quote-to-cash and work planes' shaping, its tool contract, and
//!   the guarded tools — the required `reason`, demo scope, and the
//!   posted-invoice refusal — exercised against an in-process Odoo

#[cfg(test)]
mod access_policy;
#[cfg(test)]
mod audit_metadata;
#[cfg(test)]
mod filter_hallucinated_args;
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
mod farm_dashboard_shape;
#[cfg(test)]
mod typed_activity;
#[cfg(test)]
mod typed_inputs;
#[cfg(all(test, unix))]
mod typed_users_handler;

#[cfg(test)]
mod odoo_apps;
#[cfg(test)]
mod odoo_attachment_kind;
#[cfg(test)]
mod odoo_attachments;
#[cfg(test)]
mod odoo_authority;
#[cfg(test)]
mod odoo_domains;
#[cfg(test)]
mod odoo_error;
#[cfg(test)]
mod odoo_format;
#[cfg(test)]
mod odoo_guard;
#[cfg(test)]
mod odoo_guarded;
#[cfg(test)]
mod odoo_lead_table;
#[cfg(test)]
mod odoo_mock;
#[cfg(test)]
mod odoo_notes;
#[cfg(test)]
mod odoo_rpc;
#[cfg(test)]
mod odoo_sales;
#[cfg(test)]
mod odoo_text;
#[cfg(test)]
mod odoo_tools;
#[cfg(test)]
mod odoo_work;
