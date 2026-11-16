//! The Planner family's surface — plans, buckets, tasks and rosters: each
//! tool's title and behaviour class from the Belgian `_annotations.py`, and
//! one definition list and dispatch over the family's domain tables.

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Tool};
use systemprompt::identifiers::McpServerId;

use super::{ToolCall, ToolKind, ToolSpec};

pub(super) const PLANNER_SPECS: &[ToolSpec] = &[
    ToolSpec::new(
        "d365_list_planner_plans",
        "List planner plans",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_my_planner_tasks",
        "List my planner tasks",
        ToolKind::Read,
    ),
    ToolSpec::new("d365_get_planner_plan", "Get planner plan", ToolKind::Read),
    ToolSpec::new(
        "d365_create_planner_plan",
        "Create planner plan",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_update_planner_plan",
        "Update planner plan",
        ToolKind::IdempotentWrite,
    ),
    ToolSpec::new(
        "d365_delete_planner_plan",
        "Delete planner plan",
        ToolKind::Destructive,
    ),
    ToolSpec::new(
        "d365_list_planner_buckets",
        "List planner buckets",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_get_planner_bucket",
        "Get planner bucket",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_create_planner_bucket",
        "Create planner bucket",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_update_planner_bucket",
        "Update planner bucket",
        ToolKind::IdempotentWrite,
    ),
    ToolSpec::new(
        "d365_delete_planner_bucket",
        "Delete planner bucket",
        ToolKind::Destructive,
    ),
    ToolSpec::new(
        "d365_list_planner_tasks",
        "List planner tasks",
        ToolKind::Read,
    ),
    ToolSpec::new("d365_get_planner_task", "Get planner task", ToolKind::Read),
    ToolSpec::new(
        "d365_create_planner_task",
        "Create planner task",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_update_planner_task",
        "Update planner task",
        ToolKind::IdempotentWrite,
    ),
    ToolSpec::new(
        "d365_delete_planner_task",
        "Delete planner task",
        ToolKind::Destructive,
    ),
    ToolSpec::new(
        "d365_get_planner_task_details",
        "Get planner task details",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_update_planner_task_details",
        "Update planner task details",
        ToolKind::IdempotentWrite,
    ),
    ToolSpec::new(
        "d365_create_planner_roster",
        "Create planner roster",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_delete_planner_roster",
        "Delete planner roster",
        ToolKind::Destructive,
    ),
    ToolSpec::new(
        "d365_list_planner_roster_members",
        "List planner roster members",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_add_planner_roster_member",
        "Add planner roster member",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_delete_planner_roster_member",
        "Delete planner roster member",
        ToolKind::Destructive,
    ),
];

pub(super) fn definitions(server: &McpServerId) -> Vec<Tool> {
    let mut tools = super::planner::definitions(server);
    tools.extend(super::planner_bucket::definitions(server));
    tools.extend(super::planner_task::definitions(server));
    tools.extend(super::planner_details::definitions(server));
    tools.extend(super::planner_roster::definitions(server));
    tools
}

pub(super) async fn dispatch(call: &ToolCall<'_>) -> Option<Result<CallToolResult, McpError>> {
    if let Some(result) = super::planner::dispatch(call).await {
        return Some(result);
    }
    if let Some(result) = super::planner_bucket::dispatch(call).await {
        return Some(result);
    }
    if let Some(result) = super::planner_task::dispatch(call).await {
        return Some(result);
    }
    if let Some(result) = super::planner_details::dispatch(call).await {
        return Some(result);
    }
    super::planner_roster::dispatch(call).await
}
