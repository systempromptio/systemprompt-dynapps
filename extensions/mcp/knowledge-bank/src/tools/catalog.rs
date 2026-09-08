//! MCP tool catalog, grouped by retrieval and index maintenance.

use super::{
    ADD_DOCUMENT_TOOL, EXPORT_TOOL, INDEX_STATS_TOOL, LIST_SOURCES_TOOL, PRUNE_TOOL, SEARCH_TOOL,
    SYNC_STATUS_TOOL, SYNC_TOOL, SourceExportInput, add_document_input_schema, create_tool, drive,
    index_stats_input_schema, list_sources_input_schema, output_schema, prune_input_schema,
    search_input_schema, sync_input_schema,
};
use rmcp::model::Tool;

pub fn list_tools() -> Vec<Tool> {
    let output = output_schema();
    let mut tools = vec![
        drive::inventory_tool(),
        create_tool(
            EXPORT_TOOL,
            "Export complete sources",
            "Export caller-readable Jira and Confluence source text in doc-id order. Use source_type:source_ref of the last row as after; an empty page ends the export. Rebuild after upgrading to backfill source text.",
            &serde_json::to_value(schemars::schema_for!(SourceExportInput)).unwrap_or_default(),
            &output,
        ),
        create_tool(
            SYNC_STATUS_TOOL,
            "Knowledge sync status",
            "Return your latest persisted sync run and source outcomes. A failed or running run is not complete.",
            &list_sources_input_schema(),
            &output,
        ),
        create_tool(
            SEARCH_TOOL,
            "Search Knowledge",
            "Search the project knowledge bank (team notes, ADRs and specs, plus the Confluence \
             and Jira you have synced). Pass a \
             natural-language query; scope it with source_types (ids from \
             list_knowledge_sources; omit for every live category, name archive categories \
             explicitly); cap results with top_k (1-20). Every hit carries a citation: uri, \
             lines, date and date_source — read those before trusting a hit, because results \
             are ranked by similarity, not authority. You only ever see documents your own \
             Atlassian account could read.\n\n\
             Example: {\"query\": \"why was the cart rounding changed\", \"source_types\": \
             [\"confluence\", \"jira\"], \"top_k\": 5}",
            &search_input_schema(),
            &output,
        ),
        create_tool(
            LIST_SOURCES_TOOL,
            "List Knowledge Sources",
            "List the knowledge source categories this instance can search, with the number of \
             documents you may read in each and when you last synced it. Takes no arguments.",
            &list_sources_input_schema(),
            &output,
        ),
        create_tool(
            INDEX_STATS_TOOL,
            "Knowledge Index Stats",
            "Administrators only. Report shared-index statistics: document, chunk and grant \
             totals, how many users have synced, embedding-cache rows, database size, last \
             build time and the index version stamp. Takes no arguments.",
            &index_stats_input_schema(),
            &output,
        ),
    ];
    tools.extend(maintenance_tools(&output));
    tools
}

// JSON: MCP tool schemas are serialized at the transport boundary
fn maintenance_tools(output: &serde_json::Value) -> Vec<Tool> {
    vec![
        create_tool(
            SYNC_TOOL,
            "Sync Knowledge",
            "Index the files categories and what YOUR Atlassian account can read. Confluence \
             and Jira are read through the Atlassian account you connected on \
             /admin/profile#connected-accounts; if it is not connected or Atlassian is \
             unreachable, only the files categories are synced and the reply says why. \
             Runs in the background: reads each category, chunks and \
             embeds what changed, grants Atlassian documents to you and files documents to \
             their category's readers, and removes what disappeared. Pass source_types to \
             limit the run and rebuild to ignore your cursor. Check progress with \
             list_knowledge_sources.",
            &sync_input_schema(),
            output,
        ),
        create_tool(
            ADD_DOCUMENT_TOOL,
            "Add Knowledge Document",
            "Add one hand-authored document (meeting note, ADR, spec) to a files category and \
             index it immediately — no sync or rebuild needed. Pass source_type, a bare \
             file_name ending in .md, .txt, .pdf or .docx, and either content (text) or \
             content_base64 (binary). The document becomes readable by the category's \
             configured readers, everyone signed in by default. overwrite replaces an \
             existing name and is for administrators.\n\n\
             Example: {\"source_type\": \"meeting_notes\", \"file_name\": \
             \"2026-09-02_checkout-sync.md\", \"content\": \"# Checkout sync\\n...\"}",
            &add_document_input_schema(),
            output,
        ),
        create_tool(
            PRUNE_TOOL,
            "Prune Knowledge",
            "Administrators only. Remove the grants of users whose account is gone or whose \
             last sync is older than stale_days (default 90), delete documents nobody holds \
             any more, and clear unreferenced cache rows. dry_run reports without deleting.",
            &prune_input_schema(),
            output,
        ),
    ]
}
