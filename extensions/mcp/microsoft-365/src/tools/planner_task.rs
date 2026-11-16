//! Planner tasks: list them by plan or bucket, read, create, update (with
//! assignments added and removed in one patch) and delete.

use schemars::JsonSchema;
use serde::Deserialize;
// JSON: tasks are Graph resources; their bodies are built as Planner writes
// them.
use serde_json::{Map, Value, json};

use super::graph_tool::graph_tools;
use super::planner::{PLANNER_TASK_FIELDS, delete_versioned, patch_versioned};
use super::reply::{GraphItem, GraphList};
use super::teams::{list_page, select};
use crate::graph::sanitize::validate_id;
use crate::graph::{GraphClient, GraphError};

const LIST_TASKS: &str = "List Planner tasks in a plan or bucket. Exactly one of `plan_id` \
    or `bucket_id` must be given. Page with `top` (1-50) and `skip`.";
const GET_TASK: &str = "Get a Planner task by ID.";
const CREATE_TASK: &str = "Create a Planner task, optionally in a bucket and assigned to users.";
const UPDATE_TASK: &str = "Update a Planner task; only the fields given change. \
    `assignees_to_add` assigns users and `assignees_to_remove` unassigns them. Needs the \
    task's `@odata.etag` from a prior get.";
const DELETE_TASK: &str = "Delete a Planner task. Needs the `@odata.etag` from a prior get.";
graph_tools! {
    "d365_list_planner_tasks" => list_planner_tasks, ListTasksInput, GraphList, LIST_TASKS;
    "d365_get_planner_task" => get_planner_task, TaskIdInput, GraphItem, GET_TASK;
    "d365_create_planner_task" => create_planner_task, CreateTaskInput, GraphItem, CREATE_TASK;
    "d365_update_planner_task" => update_planner_task, UpdateTaskInput, GraphItem, UPDATE_TASK;
    "d365_delete_planner_task" => delete_planner_task, DeleteTaskInput, GraphItem, DELETE_TASK;
}

const fn default_top() -> i64 {
    25
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListTasksInput {
    #[schemars(description = "List all tasks in this plan.")]
    #[serde(default)]
    pub plan_id: String,
    #[schemars(description = "List the tasks in this single bucket.")]
    #[serde(default)]
    pub bucket_id: String,
    #[schemars(description = "Maximum number of tasks to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TaskIdInput {
    #[schemars(description = "The task ID.")]
    pub task_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateTaskInput {
    #[schemars(description = "Plan that owns the task.")]
    pub plan_id: String,
    #[schemars(description = "Task title.")]
    pub title: String,
    #[schemars(description = "Bucket to place the task in; optional.")]
    #[serde(default)]
    pub bucket_id: String,
    #[schemars(description = "User IDs (Entra object IDs) to assign the task to.")]
    #[serde(default)]
    pub assignee_user_ids: Vec<String>,
    #[schemars(description = "0 (urgent) to 10 (low); usual values 1, 3, 5, 9.")]
    #[serde(default)]
    #[schemars(with = "i64")]
    pub priority: Option<i64>,
    #[schemars(description = "Completion, 0-100.")]
    #[serde(default)]
    #[schemars(with = "i64")]
    pub percent_complete: Option<i64>,
    #[schemars(description = "ISO 8601 due date, e.g. 2026-05-01T00:00:00Z.")]
    #[serde(default)]
    pub due_date: String,
    #[schemars(description = "ISO 8601 start date.")]
    #[serde(default)]
    pub start_date: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateTaskInput {
    #[schemars(description = "The task ID.")]
    pub task_id: String,
    #[schemars(description = "The task's `@odata.etag` from a prior get (sent as If-Match).")]
    pub etag: String,
    #[schemars(description = "New title.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub title: Option<String>,
    #[schemars(description = "Move the task to this bucket.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub bucket_id: Option<String>,
    #[schemars(description = "Priority, 0-10.")]
    #[serde(default)]
    #[schemars(with = "i64")]
    pub priority: Option<i64>,
    #[schemars(description = "Completion, 0-100.")]
    #[serde(default)]
    #[schemars(with = "i64")]
    pub percent_complete: Option<i64>,
    #[schemars(description = "New ISO 8601 due date.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub due_date: Option<String>,
    #[schemars(description = "New ISO 8601 start date.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub start_date: Option<String>,
    #[schemars(description = "User IDs to assign.")]
    #[serde(default)]
    pub assignees_to_add: Vec<String>,
    #[schemars(description = "User IDs to unassign.")]
    #[serde(default)]
    pub assignees_to_remove: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DeleteTaskInput {
    #[schemars(description = "The task ID.")]
    pub task_id: String,
    #[schemars(description = "The task's `@odata.etag` from a prior get (sent as If-Match).")]
    pub etag: String,
}

// Why: Planner reads an assignment orderHint of " !" as "append after the
// others".
fn assignments(user_ids: &[String]) -> Map<String, Value> {
    let assigned = json!({"@odata.type": "#microsoft.planner.assignment", "orderHint": " !"});
    user_ids
        .iter()
        .map(|user| (user.clone(), assigned.clone()))
        .collect()
}

pub(super) fn task_path(task_id: &str) -> Result<String, GraphError> {
    let task = validate_id(task_id, "task_id")?;
    Ok(format!("/planner/tasks/{task}"))
}

pub(super) fn put(patch: &mut Map<String, Value>, key: &str, value: Option<impl Into<Value>>) {
    if let Some(value) = value {
        patch.insert(key.to_owned(), value.into());
    }
}

pub async fn list_planner_tasks(
    graph: &GraphClient,
    input: ListTasksInput,
) -> Result<GraphList, GraphError> {
    let path = match (input.plan_id.is_empty(), input.bucket_id.is_empty()) {
        (false, true) => format!(
            "/planner/plans/{}/tasks",
            validate_id(&input.plan_id, "plan_id")?
        ),
        (true, false) => format!(
            "/planner/buckets/{}/tasks",
            validate_id(&input.bucket_id, "bucket_id")?
        ),
        _ => {
            return Err(GraphError::Invalid(
                "Provide exactly one of plan_id or bucket_id".to_owned(),
            ));
        },
    };
    let query = select(PLANNER_TASK_FIELDS);
    list_page(graph, &path, query, input.skip, Some(input.top)).await
}

pub async fn get_planner_task(
    graph: &GraphClient,
    input: TaskIdInput,
) -> Result<GraphItem, GraphError> {
    let path = task_path(&input.task_id)?;
    let data = graph.get_json(&path, &select(PLANNER_TASK_FIELDS)).await?;
    Ok(GraphItem::new(data))
}

pub async fn create_planner_task(
    graph: &GraphClient,
    input: CreateTaskInput,
) -> Result<GraphItem, GraphError> {
    let plan = validate_id(&input.plan_id, "plan_id")?;
    let mut body = Map::new();
    body.insert("planId".to_owned(), Value::from(plan));
    body.insert("title".to_owned(), Value::String(input.title));
    if !input.bucket_id.is_empty() {
        let bucket = validate_id(&input.bucket_id, "bucket_id")?;
        body.insert("bucketId".to_owned(), Value::from(bucket));
    }
    if !input.assignee_user_ids.is_empty() {
        let assigned = assignments(&input.assignee_user_ids);
        body.insert("assignments".to_owned(), Value::Object(assigned));
    }
    put(&mut body, "priority", input.priority);
    put(&mut body, "percentComplete", input.percent_complete);
    let due = Some(input.due_date).filter(|date| !date.is_empty());
    put(&mut body, "dueDateTime", due);
    let start = Some(input.start_date).filter(|date| !date.is_empty());
    put(&mut body, "startDateTime", start);
    let data = graph
        .post_json("/planner/tasks", Value::Object(body))
        .await?;
    Ok(GraphItem::new(data))
}

pub async fn update_planner_task(
    graph: &GraphClient,
    input: UpdateTaskInput,
) -> Result<GraphItem, GraphError> {
    let path = task_path(&input.task_id)?;
    let mut patch = Map::new();
    put(&mut patch, "title", input.title);
    if let Some(bucket) = input.bucket_id {
        let bucket = validate_id(&bucket, "bucket_id")?;
        patch.insert("bucketId".to_owned(), Value::from(bucket));
    }
    put(&mut patch, "priority", input.priority);
    put(&mut patch, "percentComplete", input.percent_complete);
    put(&mut patch, "dueDateTime", input.due_date);
    put(&mut patch, "startDateTime", input.start_date);
    if !input.assignees_to_add.is_empty() || !input.assignees_to_remove.is_empty() {
        let mut assigned = assignments(&input.assignees_to_add);
        for user in input.assignees_to_remove {
            assigned.insert(user, Value::Null);
        }
        patch.insert("assignments".to_owned(), Value::Object(assigned));
    }
    patch_versioned(graph, &path, &input.etag, patch).await
}

pub async fn delete_planner_task(
    graph: &GraphClient,
    input: DeleteTaskInput,
) -> Result<GraphItem, GraphError> {
    let path = task_path(&input.task_id)?;
    let deleted = json!({"deleted": true, "task_id": input.task_id});
    delete_versioned(graph, &path, &input.etag, deleted).await
}
