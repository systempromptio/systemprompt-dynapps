//! Tool definitions exposed by the knowledge-bank MCP server.
//!
//! The retrieval contract mirrors the Node project-context MCP server being
//! migrated here (`search`, `list_sources`, `index_stats`), renamed into this
//! instance's namespace. `sync_knowledge` is every user's own sync — it reads
//! Atlassian through their connected account — and `prune_knowledge` is the
//! administrator's broom for the shared index.

mod catalog;
mod drive;
use drive::include_shared_default;
pub use drive::{DRIVE_INVENTORY_TOOL, DriveInventoryInput};

use rmcp::model::{MetaObject, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::mcp::{McpOutputSchema, default_tool_visibility, tool_ui_meta};
use systemprompt::models::artifacts::CliArtifact;

pub const SERVER_NAME: &str = "knowledge-bank";
pub const EXPORT_TOOL: &str = "export_knowledge_sources";
pub const SYNC_STATUS_TOOL: &str = "knowledge_sync_status";
pub const SEARCH_TOOL: &str = "search_knowledge";
pub const LIST_SOURCES_TOOL: &str = "list_knowledge_sources";
pub const INDEX_STATS_TOOL: &str = "knowledge_index_stats";
pub const SYNC_TOOL: &str = "sync_knowledge";
pub const PRUNE_TOOL: &str = "prune_knowledge";
pub const ADD_DOCUMENT_TOOL: &str = "add_knowledge_document";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchKnowledgeInput {
    #[serde(default)]
    #[schemars(
        description = "Stable project folder IDs from knowledge_drive_inventory; omitted searches all projects"
    )]
    pub project_ids: Vec<String>,
    #[serde(default = "include_shared_default")]
    pub include_shared: bool,
    #[schemars(description = "Natural-language query to search the knowledge bank with")]
    pub query: String,
    #[schemars(
        description = "Source categories to search (ids from list_knowledge_sources, e.g. \
                       confluence, jira). Omit to search every live category; archive \
                       categories are searched only when named here. Prefer the smallest set \
                       that could hold the answer"
    )]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_types: Option<Vec<String>>,
    #[schemars(
        range(min = 1, max = 20),
        description = "Maximum results to return (1-20). Omit for the configured default"
    )]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[expect(
    clippy::empty_structs_with_brackets,
    reason = "MCP tool arguments arrive as an empty JSON object `{}`; a unit struct would fail \
              to deserialize from it"
)]
pub struct ListKnowledgeSourcesInput {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[expect(
    clippy::empty_structs_with_brackets,
    reason = "MCP tool arguments arrive as an empty JSON object `{}`; a unit struct would fail \
              to deserialize from it"
)]
pub struct IndexStatsInput {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct SyncKnowledgeInput {
    #[serde(default)]
    #[schemars(
        description = "Immediately reconcile the Drive inventory and retry failed files, retaining valid extraction caches"
    )]
    pub force_resync: bool,
    #[schemars(description = "Source categories to sync. Omit to sync every configured category")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_types: Option<Vec<String>>,
    #[schemars(
        description = "Ignore your incremental cursor and re-chunk everything you can read. \
                       Cached embeddings and dates are reused, so a rebuild does not re-embed \
                       unchanged text"
    )]
    #[serde(default)]
    pub rebuild: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct AddKnowledgeDocumentInput {
    #[schemars(
        description = "A files category id from list_knowledge_sources (e.g. meeting_notes, \
                       adrs). Confluence and Jira categories cannot take uploads"
    )]
    pub source_type: String,
    #[schemars(
        description = "Bare file name with its suffix: .md, .txt, .pdf or .docx. No directories"
    )]
    pub file_name: String,
    #[schemars(description = "The document text, for .md and .txt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[schemars(description = "The document bytes, base64-encoded, for .pdf and .docx")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_base64: Option<String>,
    #[schemars(description = "Title to store instead of the one inferred from the text")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[schemars(
        description = "Replace a document of the same name (administrators only). Off by default"
    )]
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema)]
pub struct PruneKnowledgeInput {
    #[schemars(
        range(min = 1, max = 3650),
        description = "A user whose last sync is older than this many days loses their grants \
                       (default 90). Documents nobody holds any more are deleted"
    )]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale_days: Option<u32>,
    #[schemars(description = "Report what would be removed without removing it")]
    #[serde(default)]
    pub dry_run: bool,
}

#[must_use]
// JSON: protocol boundary
pub fn search_input_schema() -> serde_json::Value {
    schemars::schema_for!(SearchKnowledgeInput).to_value()
}

#[must_use]
// JSON: protocol boundary
pub fn list_sources_input_schema() -> serde_json::Value {
    schemars::schema_for!(ListKnowledgeSourcesInput).to_value()
}

#[must_use]
// JSON: protocol boundary
pub fn index_stats_input_schema() -> serde_json::Value {
    schemars::schema_for!(IndexStatsInput).to_value()
}

#[must_use]
// JSON: protocol boundary
pub fn sync_input_schema() -> serde_json::Value {
    schemars::schema_for!(SyncKnowledgeInput).to_value()
}

#[must_use]
// JSON: protocol boundary
pub fn prune_input_schema() -> serde_json::Value {
    schemars::schema_for!(PruneKnowledgeInput).to_value()
}

#[must_use]
// JSON: protocol boundary
pub fn add_document_input_schema() -> serde_json::Value {
    schemars::schema_for!(AddKnowledgeDocumentInput).to_value()
}

#[must_use]
// JSON: protocol boundary
pub fn output_schema() -> serde_json::Value {
    <CliArtifact as McpOutputSchema>::validated_schema()
}

// JSON: protocol boundary
fn create_tool(
    name: &str,
    title: &str,
    description: &str,
    input_schema: &serde_json::Value,
    output: &serde_json::Value,
) -> Tool {
    let input_obj = input_schema
        .as_object()
        .cloned()
        .unwrap_or_else(serde_json::Map::new);
    let output_obj = output
        .as_object()
        .cloned()
        .unwrap_or_else(serde_json::Map::new);

    let mut tool = Tool::default();
    tool.name = name.to_owned().into();
    tool.title = Some(title.to_owned());
    tool.description = Some(description.to_owned().into());
    tool.input_schema = Arc::new(input_obj);
    tool.output_schema = Some(Arc::new(output_obj));
    tool.meta = Some(MetaObject(tool_ui_meta(
        SERVER_NAME,
        &default_tool_visibility(),
    )));
    tool
}

#[must_use]
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceExportInput {
    pub after: Option<String>,
    pub limit: Option<i64>,
}

pub use catalog::list_tools;
