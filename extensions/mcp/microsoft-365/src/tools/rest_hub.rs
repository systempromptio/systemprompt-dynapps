//! The rest of the surface — OneNote, groups, users, Microsoft Search and
//! `d365_batch`: each tool's title and behaviour class from the Belgian
//! `_annotations.py`, and one definition list and dispatch over them. The
//! three `d365_app_*` entry points of the Belgian `apps/` (interactive HTML
//! views) are not served here; the tools they wrap are.

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Tool};
use systemprompt::identifiers::McpServerId;

use super::{ToolCall, ToolKind, ToolSpec};

pub(super) const REST_SPECS: &[ToolSpec] = &[
    ToolSpec::new("d365_list_notebooks", "List notebooks", ToolKind::Read),
    ToolSpec::new(
        "d365_list_notebook_sections",
        "List notebook sections",
        ToolKind::Read,
    ),
    ToolSpec::new("d365_list_sections", "List sections", ToolKind::Read),
    ToolSpec::new("d365_get_page_content", "Get page content", ToolKind::Read),
    ToolSpec::new(
        "d365_list_section_pages",
        "List section pages",
        ToolKind::Read,
    ),
    ToolSpec::new("d365_create_notebook", "Create notebook", ToolKind::Write),
    ToolSpec::new("d365_create_section", "Create section", ToolKind::Write),
    ToolSpec::new("d365_create_page", "Create page", ToolKind::Write),
    ToolSpec::new("d365_list_groups", "List groups", ToolKind::Read),
    ToolSpec::new("d365_get_group", "Get group", ToolKind::Read),
    ToolSpec::new(
        "d365_list_group_members",
        "List group members",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_group_owners",
        "List group owners",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_group_conversations",
        "List group conversations",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_group_threads",
        "List group threads",
        ToolKind::Read,
    ),
    ToolSpec::new("d365_get_my_profile", "Get my profile", ToolKind::Read),
    ToolSpec::new("d365_search_users", "Search users", ToolKind::Read),
    ToolSpec::new("d365_get_user", "Get user", ToolKind::Read),
    ToolSpec::new("d365_get_my_manager", "Get my manager", ToolKind::Read),
    ToolSpec::new(
        "d365_list_direct_reports",
        "List direct reports",
        ToolKind::Read,
    ),
    ToolSpec::new("d365_search", "Search Microsoft 365", ToolKind::Read),
    ToolSpec::new("d365_search_messages", "Search messages", ToolKind::Read),
    ToolSpec::new("d365_batch", "Execute batch request", ToolKind::Write),
];

pub(super) fn definitions(server: &McpServerId) -> Vec<Tool> {
    let mut tools = super::onenote::definitions(server);
    tools.extend(super::groups::definitions(server));
    tools.extend(super::users::definitions(server));
    tools.extend(super::search::definitions(server));
    tools
}

pub(super) async fn dispatch(call: &ToolCall<'_>) -> Option<Result<CallToolResult, McpError>> {
    if let Some(result) = super::onenote::dispatch(call).await {
        return Some(result);
    }
    if let Some(result) = super::groups::dispatch(call).await {
        return Some(result);
    }
    if let Some(result) = super::users::dispatch(call).await {
        return Some(result);
    }
    super::search::dispatch(call).await
}
