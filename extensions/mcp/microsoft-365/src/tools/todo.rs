//! Microsoft To Do tools: the user's task lists and the tasks in them —
//! list, read, create, update and delete. Graph v1.0 refuses `$select` on
//! `/me/todo/lists` and on a list's tasks (400 `RequestBroker--ParseUri`),
//! so these reads take Graph's default fields; `$top` and `$skip` work.
//! Due dates travel as `dateTimeTimeZone` in UTC.

use schemars::JsonSchema;
use serde::Deserialize;
// JSON: task and list bodies are built as the Graph resource they write,
// and Graph's answer passes through as returned.
use serde_json::{Value, json};

use super::graph_tool::graph_tools;
use super::reply::{GraphItem, GraphList};
use crate::graph::sanitize::{DEFAULT_MAX_TOP, clamp_skip, clamp_top, validate_id};
use crate::graph::{GraphClient, GraphError, GraphId, GraphQuery, Window};

const LISTS: &str = "/me/todo/lists";

const LIST_TODO_LISTS: &str = "List the signed-in user's To Do task lists. Page with `skip`.";
const LIST_TODO_TASKS: &str = "List the tasks in a To Do list. Page with `top` (1-50) and `skip`.";
const GET_TODO_TASK: &str = "Get the full details of one To Do task.";
const CREATE_TODO_TASK: &str = "Create a task in a To Do list.";
const UPDATE_TODO_TASK: &str = "Update a To Do task; only the fields given change.";
const DELETE_TODO_TASK: &str = "Delete a task from a To Do list.";
const CREATE_TODO_LIST: &str = "Create a To Do task list.";

graph_tools! {
    "d365_list_todo_lists" => list_todo_lists, ListTodoListsInput, GraphList, LIST_TODO_LISTS;
    "d365_list_todo_tasks" => list_todo_tasks, ListTodoTasksInput, GraphList, LIST_TODO_TASKS;
    "d365_get_todo_task" => get_todo_task, TodoTaskIdInput, GraphItem, GET_TODO_TASK;
    "d365_create_todo_task" => create_todo_task, CreateTodoTaskInput, GraphItem,
        CREATE_TODO_TASK;
    "d365_update_todo_task" => update_todo_task, UpdateTodoTaskInput, GraphItem,
        UPDATE_TODO_TASK;
    "d365_delete_todo_task" => delete_todo_task, TodoTaskIdInput, GraphItem, DELETE_TODO_TASK;
    "d365_create_todo_list" => create_todo_list, CreateTodoListInput, GraphItem,
        CREATE_TODO_LIST;
}

const fn default_top() -> i64 {
    25
}

fn default_importance() -> String {
    "normal".to_owned()
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct ListTodoListsInput {
    #[schemars(description = "Items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListTodoTasksInput {
    #[schemars(description = "The To Do list ID.")]
    pub list_id: String,
    #[schemars(description = "Number of tasks to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TodoTaskIdInput {
    #[schemars(description = "The To Do list ID.")]
    pub list_id: String,
    #[schemars(description = "The task ID.")]
    pub task_id: GraphId,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateTodoTaskInput {
    #[schemars(description = "The To Do list ID.")]
    pub list_id: String,
    #[schemars(description = "Task title (required).")]
    pub title: String,
    #[schemars(description = "Optional task body / notes.")]
    #[serde(default)]
    pub body: String,
    #[schemars(description = "Optional due date, ISO 8601 (e.g. \"2024-12-31\").")]
    #[serde(default)]
    pub due_date: String,
    #[schemars(description = "Importance: \"low\", \"normal\" or \"high\" (default normal).")]
    #[serde(default = "default_importance")]
    pub importance: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateTodoTaskInput {
    #[schemars(description = "The To Do list ID.")]
    pub list_id: String,
    #[schemars(description = "The task ID to update.")]
    pub task_id: GraphId,
    #[schemars(description = "New task title.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub title: Option<String>,
    #[schemars(
        description = "New status: \"notStarted\", \"inProgress\", \"completed\", \
        \"waitingOnOthers\" or \"deferred\"."
    )]
    #[serde(default)]
    #[schemars(with = "String")]
    pub status: Option<String>,
    #[schemars(description = "New importance: \"low\", \"normal\" or \"high\".")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub importance: Option<String>,
    #[schemars(description = "New due date, ISO 8601.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateTodoListInput {
    #[schemars(description = "Name of the new list.")]
    pub display_name: String,
}

fn tasks_path(list_id: &str) -> Result<String, GraphError> {
    let list = validate_id(list_id, "list_id")?;
    Ok(format!("{LISTS}/{list}/tasks"))
}

fn task_path(list_id: &str, task: &str) -> Result<String, GraphError> {
    let tasks = tasks_path(list_id)?;
    let task = validate_id(task, "task_id")?;
    Ok(format!("{tasks}/{task}"))
}

fn due(date: &str) -> Value {
    json!({ "dateTime": date, "timeZone": "UTC" })
}

pub async fn list_todo_lists(
    graph: &GraphClient,
    input: ListTodoListsInput,
) -> Result<GraphList, GraphError> {
    let window = Window {
        skip: clamp_skip(input.skip),
        top: None,
    };
    let page = graph
        .get_collection(LISTS, &GraphQuery::new(), window)
        .await?;
    Ok(GraphList::from(page))
}

pub async fn list_todo_tasks(
    graph: &GraphClient,
    input: ListTodoTasksInput,
) -> Result<GraphList, GraphError> {
    let path = tasks_path(&input.list_id)?;
    let window = Window {
        skip: clamp_skip(input.skip),
        top: Some(clamp_top(input.top, DEFAULT_MAX_TOP)),
    };
    let page = graph
        .get_collection(&path, &GraphQuery::new(), window)
        .await?;
    Ok(GraphList::from(page))
}

pub async fn get_todo_task(
    graph: &GraphClient,
    input: TodoTaskIdInput,
) -> Result<GraphItem, GraphError> {
    let path = task_path(&input.list_id, &input.task_id)?;
    let data = graph.get_json(&path, &GraphQuery::new()).await?;
    Ok(GraphItem::new(data))
}

pub async fn create_todo_task(
    graph: &GraphClient,
    input: CreateTodoTaskInput,
) -> Result<GraphItem, GraphError> {
    let path = tasks_path(&input.list_id)?;
    let mut task = json!({ "title": &input.title, "importance": &input.importance });
    if !input.body.is_empty() {
        task["body"] = json!({ "content": &input.body, "contentType": "text" });
    }
    if !input.due_date.is_empty() {
        task["dueDateTime"] = due(&input.due_date);
    }
    let data = graph.post_json(&path, task).await?;
    Ok(GraphItem::new(data))
}

pub async fn update_todo_task(
    graph: &GraphClient,
    input: UpdateTodoTaskInput,
) -> Result<GraphItem, GraphError> {
    let path = task_path(&input.list_id, &input.task_id)?;
    let mut patch = json!({});
    if let Some(title) = &input.title {
        patch["title"] = json!(title);
    }
    if let Some(status) = &input.status {
        patch["status"] = json!(status);
    }
    if let Some(importance) = &input.importance {
        patch["importance"] = json!(importance);
    }
    if let Some(due_date) = &input.due_date {
        patch["dueDateTime"] = due(due_date);
    }
    let data = graph.patch_json(&path, patch).await?;
    Ok(GraphItem::new(data))
}

pub async fn delete_todo_task(
    graph: &GraphClient,
    input: TodoTaskIdInput,
) -> Result<GraphItem, GraphError> {
    let path = task_path(&input.list_id, &input.task_id)?;
    graph.delete(&path).await?;
    Ok(GraphItem::new(json!({
        "deleted": true,
        "list_id": &input.list_id,
        "task_id": &input.task_id,
    })))
}

pub async fn create_todo_list(
    graph: &GraphClient,
    input: CreateTodoListInput,
) -> Result<GraphItem, GraphError> {
    let list = json!({ "displayName": &input.display_name });
    let data = graph.post_json(LISTS, list).await?;
    Ok(GraphItem::new(data))
}
