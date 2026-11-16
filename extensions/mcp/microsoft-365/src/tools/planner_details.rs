//! The details object of a Planner task: description, checklist,
//! references and preview type. It carries its own `@odata.etag`, separate
//! from the task's, and an update sends that one.

use schemars::JsonSchema;
use serde::Deserialize;
// JSON: details are Graph resources; checklist and references are Planner's
// keyed objects, passed through as given.
use serde_json::{Map, Value};

use super::graph_tool::graph_tools;
use super::planner::patch_versioned;
use super::planner_task::{TaskIdInput, put, task_path};
use super::reply::GraphItem;
use crate::graph::{GraphClient, GraphError, GraphId, GraphQuery};

const GET_DETAILS: &str = "Get the details of a Planner task (description, checklist, \
    references, preview type). The reply carries the details' own `@odata.etag`.";
const UPDATE_DETAILS: &str = "Update the details of a Planner task. Checklist and references \
    use Planner's keyed format: checklist `{\"<guid>\": {\"@odata.type\": \
    \"microsoft.graph.plannerChecklistItem\", \"title\": \"...\", \"isChecked\": false}}` (null \
    deletes an item); references `{\"<url with dots as %2E>\": {\"@odata.type\": \
    \"microsoft.graph.plannerExternalReference\", \"alias\": \"...\", \"type\": \"Other\"}}`. \
    Needs the details' `@odata.etag`, not the task's.";

graph_tools! {
    "d365_get_planner_task_details" => get_planner_task_details, TaskIdInput, GraphItem,
        GET_DETAILS;
    "d365_update_planner_task_details" => update_planner_task_details, UpdateDetailsInput,
        GraphItem, UPDATE_DETAILS;
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateDetailsInput {
    #[schemars(description = "The task ID.")]
    pub task_id: GraphId,
    #[schemars(description = "`@odata.etag` of the details object (not of the task).")]
    pub etag: String,
    #[schemars(description = "New task description (free text).")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub description: Option<String>,
    #[schemars(description = "automatic, noPreview, checklist, description or reference.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub preview_type: Option<String>,
    #[schemars(description = "Full or partial checklist, in Planner's keyed format.")]
    #[serde(default)]
    #[schemars(with = "Map<String, Value>")]
    pub checklist: Option<Map<String, Value>>,
    #[schemars(description = "Full or partial references, in Planner's keyed format.")]
    #[serde(default)]
    #[schemars(with = "Map<String, Value>")]
    pub references: Option<Map<String, Value>>,
}

pub async fn get_planner_task_details(
    graph: &GraphClient,
    input: TaskIdInput,
) -> Result<GraphItem, GraphError> {
    let path = format!("{}/details", task_path(&input.task_id)?);
    let data = graph.get_json(&path, &GraphQuery::new()).await?;
    Ok(GraphItem::new(data))
}

pub async fn update_planner_task_details(
    graph: &GraphClient,
    input: UpdateDetailsInput,
) -> Result<GraphItem, GraphError> {
    let path = format!("{}/details", task_path(&input.task_id)?);
    let mut patch = Map::new();
    put(&mut patch, "description", input.description);
    put(&mut patch, "previewType", input.preview_type);
    put(&mut patch, "checklist", input.checklist.map(Value::Object));
    put(
        &mut patch,
        "references",
        input.references.map(Value::Object),
    );
    patch_versioned(graph, &path, &input.etag, patch).await
}
