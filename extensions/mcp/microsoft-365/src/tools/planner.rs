//! Planner plans, and the write rules every Planner tool shares. Planner
//! versions each resource: a PATCH or DELETE carries the `@odata.etag` from
//! a prior GET as `If-Match`, so every changing tool takes an `etag`, as in
//! the Belgian server. A plan lives in a Microsoft 365 group or in a
//! group-less roster; the tools that name a container take exactly one.

use reqwest::Method;
use schemars::JsonSchema;
use serde::Deserialize;
// JSON: plans and their writes are the Graph resources Planner returns.
use serde_json::{Map, Value, json};

use super::graph_tool::graph_tools;
use super::reply::{GraphItem, GraphList};
use super::teams::{list_page, select};
use crate::graph::sanitize::validate_id;
use crate::graph::{GraphBody, GraphClient, GraphError, GraphId, GraphRequest};

pub const PLANNER_PLAN_FIELDS: &str = "id,title,createdDateTime,owner,createdBy,container";
pub const PLANNER_TASK_FIELDS: &str = "id,title,planId,bucketId,assignments,priority,\
    percentComplete,startDateTime,dueDateTime,createdDateTime,completedDateTime,orderHint";

const LIST_PLANS: &str = "List Planner plans for a group or roster. Exactly one of \
    `group_id` or `roster_id` must be given. Page with `skip`.";
const LIST_MY_TASKS: &str = "List all Planner tasks assigned to the signed-in user across \
    every plan they can reach. Page with `skip`.";
const GET_PLAN: &str = "Get a Planner plan by ID. The reply carries `@odata.etag`, which \
    later updates and deletes of the plan need.";
const CREATE_PLAN: &str = "Create a Planner plan owned by a group or roster. Exactly one of \
    `group_id` or `roster_id` must be given.";
const UPDATE_PLAN: &str = "Update a Planner plan. Needs the `@odata.etag` from a prior get.";
const DELETE_PLAN: &str = "Delete a Planner plan. Needs the `@odata.etag` from a prior get.";

graph_tools! {
    "d365_list_planner_plans" => list_planner_plans, ListPlansInput, GraphList, LIST_PLANS;
    "d365_list_my_planner_tasks" => list_my_planner_tasks, MyTasksInput, GraphList,
        LIST_MY_TASKS;
    "d365_get_planner_plan" => get_planner_plan, PlanIdInput, GraphItem, GET_PLAN;
    "d365_create_planner_plan" => create_planner_plan, CreatePlanInput, GraphItem, CREATE_PLAN;
    "d365_update_planner_plan" => update_planner_plan, UpdatePlanInput, GraphItem, UPDATE_PLAN;
    "d365_delete_planner_plan" => delete_planner_plan, DeletePlanInput, GraphItem, DELETE_PLAN;
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListPlansInput {
    #[schemars(description = "Microsoft 365 group ID that owns the plans.")]
    #[serde(default)]
    pub group_id: GraphId,
    #[schemars(description = "Roster ID (for group-less plans).")]
    #[serde(default)]
    pub roster_id: String,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct MyTasksInput {
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct PlanIdInput {
    #[schemars(description = "The plan ID.")]
    pub plan_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreatePlanInput {
    #[schemars(description = "Plan title.")]
    pub title: String,
    #[schemars(description = "Microsoft 365 group ID to own the plan.")]
    #[serde(default)]
    pub group_id: GraphId,
    #[schemars(description = "Roster ID (for group-less plans).")]
    #[serde(default)]
    pub roster_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdatePlanInput {
    #[schemars(description = "The plan ID.")]
    pub plan_id: String,
    #[schemars(description = "`@odata.etag` from a prior get (sent as If-Match).")]
    pub etag: String,
    #[schemars(description = "New plan title.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DeletePlanInput {
    #[schemars(description = "The plan ID.")]
    pub plan_id: String,
    #[schemars(description = "`@odata.etag` from a prior get (sent as If-Match).")]
    pub etag: String,
}

fn etag(value: &str) -> Result<&str, GraphError> {
    let value = value.trim();
    let printable = value.chars().all(|c| matches!(c, ' '..='~'));
    if value.is_empty() || !printable {
        return Err(GraphError::Invalid(
            "Invalid etag: pass the `@odata.etag` of a prior get".to_owned(),
        ));
    }
    Ok(value)
}

// Why: Planner refuses a PATCH or DELETE without the resource's etag in
// `If-Match`; `Prefer: return=representation` makes it echo the change.
pub(super) async fn patch_versioned(
    graph: &GraphClient,
    path: &str,
    tag: &str,
    patch: Map<String, Value>,
) -> Result<GraphItem, GraphError> {
    let tag = etag(tag)?;
    if patch.is_empty() {
        return Err(GraphError::Invalid("No fields to update".to_owned()));
    }
    let request = GraphRequest::new(Method::PATCH, path)
        .with_body(GraphBody::Json(Value::Object(patch)))
        .with_header("If-Match", tag)
        .with_header("Prefer", "return=representation");
    let data = graph.send(&request).await?.json()?;
    Ok(GraphItem::new(data))
}

pub(super) async fn delete_versioned(
    graph: &GraphClient,
    path: &str,
    tag: &str,
    deleted: Value,
) -> Result<GraphItem, GraphError> {
    let tag = etag(tag)?;
    let request = GraphRequest::new(Method::DELETE, path).with_header("If-Match", tag);
    graph.send(&request).await?;
    Ok(GraphItem::new(deleted))
}

fn container<'a>(
    group_id: &'a str,
    roster_id: &'a str,
) -> Result<(&'static str, &'a str), GraphError> {
    match (group_id.is_empty(), roster_id.is_empty()) {
        (false, true) => Ok(("group", validate_id(group_id, "group_id")?)),
        (true, false) => Ok(("roster", validate_id(roster_id, "roster_id")?)),
        _ => Err(GraphError::Invalid(
            "Provide exactly one of group_id or roster_id".to_owned(),
        )),
    }
}

pub async fn list_planner_plans(
    graph: &GraphClient,
    input: ListPlansInput,
) -> Result<GraphList, GraphError> {
    let path = match container(&input.group_id, &input.roster_id)? {
        ("group", group) => format!("/groups/{group}/planner/plans"),
        (_, roster) => format!("/planner/rosters/{roster}/plans"),
    };
    list_page(graph, &path, select(PLANNER_PLAN_FIELDS), input.skip, None).await
}

pub async fn list_my_planner_tasks(
    graph: &GraphClient,
    input: MyTasksInput,
) -> Result<GraphList, GraphError> {
    let query = select(PLANNER_TASK_FIELDS);
    list_page(graph, "/me/planner/tasks", query, input.skip, None).await
}

pub async fn get_planner_plan(
    graph: &GraphClient,
    input: PlanIdInput,
) -> Result<GraphItem, GraphError> {
    let plan = validate_id(&input.plan_id, "plan_id")?;
    let path = format!("/planner/plans/{plan}");
    let data = graph.get_json(&path, &select(PLANNER_PLAN_FIELDS)).await?;
    Ok(GraphItem::new(data))
}

pub async fn create_planner_plan(
    graph: &GraphClient,
    input: CreatePlanInput,
) -> Result<GraphItem, GraphError> {
    let (kind, id) = container(&input.group_id, &input.roster_id)?;
    let body = json!({
        "title": input.title,
        "container": {"containerId": id, "type": kind},
    });
    let data = graph.post_json("/planner/plans", body).await?;
    Ok(GraphItem::new(data))
}

pub async fn update_planner_plan(
    graph: &GraphClient,
    input: UpdatePlanInput,
) -> Result<GraphItem, GraphError> {
    let plan = validate_id(&input.plan_id, "plan_id")?;
    let mut patch = Map::new();
    if let Some(title) = input.title {
        patch.insert("title".to_owned(), Value::String(title));
    }
    let path = format!("/planner/plans/{plan}");
    patch_versioned(graph, &path, &input.etag, patch).await
}

pub async fn delete_planner_plan(
    graph: &GraphClient,
    input: DeletePlanInput,
) -> Result<GraphItem, GraphError> {
    let plan = validate_id(&input.plan_id, "plan_id")?;
    let path = format!("/planner/plans/{plan}");
    let deleted = json!({"deleted": true, "plan_id": plan});
    delete_versioned(graph, &path, &input.etag, deleted).await
}
