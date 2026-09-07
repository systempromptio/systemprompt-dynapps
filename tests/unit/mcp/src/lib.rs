//! Unit tests for the MCP extension crates' pure helpers:
//! - `systemprompt-mcp-agent`'s `filter_hallucinated_args` (CLI arg scrubbing)
//! - `systemprompt-mcp-shared`'s `truncate_on_char_boundary` (rejection-reason
//!   truncation with UTF-8 safety) and `AuditMetadata`'s stored JSON shape
//! - `systemprompt-mcp-agent`'s `systemprompt` tool contract (CLI and reporting
//!   tools, their input/output schema) and its error type's code / status /
//!   retryability
//! - `systemprompt-mcp-knowledge-bank`: the tool contract (five tools,
//!   schemas), the REQ-052 principal-to-subjects mapping, the chunk strategies
//!   and size discipline, line-pointer and title inference, dating precedence,
//!   the per-user Atlassian header contract, config validation, the local model
//!   provider, the file parsers and files connector, the upload-argument
//!   validation, and the Phase 0 requirement traceability gate

#[cfg(test)]
mod atlassian_mock_http;
#[cfg(test)]
mod atlassian_sources;
#[cfg(test)]
mod audit_metadata;
#[cfg(test)]
mod bedrock_http;
#[cfg(test)]
mod filter_hallucinated_args;
#[cfg(test)]
mod knowledge_bank;
#[cfg(test)]
mod knowledge_chunk_strategies;
#[cfg(test)]
mod knowledge_chunker;
#[cfg(test)]
mod knowledge_config;
#[cfg(test)]
mod knowledge_dating;
#[cfg(test)]
mod knowledge_downstream;
#[cfg(test)]
mod knowledge_enrich;
#[cfg(test)]
mod knowledge_fake_model;
#[cfg(test)]
mod knowledge_files_source;
#[cfg(test)]
mod knowledge_local_model;
#[cfg(test)]
mod knowledge_parsers;
#[cfg(test)]
mod knowledge_upload_input;
#[cfg(test)]
mod phase0_traceability;
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
