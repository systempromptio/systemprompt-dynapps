//! Microsoft 365 groups in the directory: list or search them, read one,
//! and list its members, owners, conversations and threads. A search is an
//! advanced directory query, so it rides with `ConsistencyLevel: eventual`.

use schemars::JsonSchema;
use serde::Deserialize;

use super::graph_tool::graph_tools;
use super::reply::{GraphItem, GraphList};
use super::teams::{list_page, select};
use super::users::USER_FIELDS;
use crate::graph::sanitize::{
    DEFAULT_MAX_TOP, clamp_skip, clamp_top, sanitize_search, validate_id,
};
use crate::graph::{GraphClient, GraphError, GraphQuery, Window};

pub const GROUP_FIELDS: &str = "id,displayName,description,mail,groupTypes,visibility";

const LIST_GROUPS: &str = "List groups in the directory, optionally filtered by a display \
    name search. Page with `top` (1-50) and `skip`.";
const GET_GROUP: &str = "Get details of a specific group.";
const LIST_GROUP_MEMBERS: &str = "List members of a group. Page with `skip`.";
const LIST_GROUP_OWNERS: &str = "List owners of a group. Page with `skip`.";
const LIST_GROUP_CONVERSATIONS: &str =
    "List conversations in a group. Page with `top` (1-50) and `skip`.";
const LIST_GROUP_THREADS: &str =
    "List threads within a group conversation. Page with `top` (1-50) and `skip`.";

graph_tools! {
    "d365_list_groups" => list_groups, ListGroupsInput, GraphList, LIST_GROUPS;
    "d365_get_group" => get_group, GroupIdInput, GraphItem, GET_GROUP;
    "d365_list_group_members" => list_group_members, GroupSkipInput, GraphList,
        LIST_GROUP_MEMBERS;
    "d365_list_group_owners" => list_group_owners, GroupSkipInput, GraphList, LIST_GROUP_OWNERS;
    "d365_list_group_conversations" => list_group_conversations, GroupConversationsInput,
        GraphList, LIST_GROUP_CONVERSATIONS;
    "d365_list_group_threads" => list_group_threads, GroupThreadsInput, GraphList,
        LIST_GROUP_THREADS;
}

const fn default_top() -> i64 {
    25
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListGroupsInput {
    #[schemars(description = "Maximum number of groups to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Optional search string matched against the display name.")]
    #[serde(default)]
    pub search: String,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GroupIdInput {
    #[schemars(description = "The group ID.")]
    pub group_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GroupSkipInput {
    #[schemars(description = "The group ID.")]
    pub group_id: String,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GroupConversationsInput {
    #[schemars(description = "The group ID.")]
    pub group_id: String,
    #[schemars(description = "Maximum number of conversations to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GroupThreadsInput {
    #[schemars(description = "The group ID.")]
    pub group_id: String,
    #[schemars(description = "The conversation ID.")]
    pub conversation_id: String,
    #[schemars(description = "Maximum number of threads to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

fn group_path(group_id: &str) -> Result<String, GraphError> {
    let group = validate_id(group_id, "group_id")?;
    Ok(format!("/groups/{group}"))
}

// Why: Graph takes `$search` on directory objects only as an advanced query,
// which needs `ConsistencyLevel: eventual` on the request.
pub(super) async fn search_directory(
    graph: &GraphClient,
    path: &str,
    mut query: GraphQuery,
    term: &str,
    window: Window,
) -> Result<GraphList, GraphError> {
    let term = sanitize_search(term);
    query.set("$search", format!("\"displayName:{term}\""));
    let eventual = [("ConsistencyLevel", "eventual")];
    let page = graph
        .get_collection_with_headers(path, &query, &eventual, window)
        .await?;
    Ok(GraphList::from(page))
}

pub async fn list_groups(
    graph: &GraphClient,
    input: ListGroupsInput,
) -> Result<GraphList, GraphError> {
    if !input.search.is_empty() {
        let window = Window {
            skip: clamp_skip(input.skip),
            top: Some(clamp_top(input.top, DEFAULT_MAX_TOP)),
        };
        let query = select(GROUP_FIELDS);
        return search_directory(graph, "/groups", query, &input.search, window).await;
    }
    list_page(
        graph,
        "/groups",
        select(GROUP_FIELDS),
        input.skip,
        Some(input.top),
    )
    .await
}

pub async fn get_group(graph: &GraphClient, input: GroupIdInput) -> Result<GraphItem, GraphError> {
    let path = group_path(&input.group_id)?;
    let data = graph.get_json(&path, &GraphQuery::new()).await?;
    Ok(GraphItem::new(data))
}

pub async fn list_group_members(
    graph: &GraphClient,
    input: GroupSkipInput,
) -> Result<GraphList, GraphError> {
    let path = format!("{}/members", group_path(&input.group_id)?);
    list_page(graph, &path, select(USER_FIELDS), input.skip, None).await
}

pub async fn list_group_owners(
    graph: &GraphClient,
    input: GroupSkipInput,
) -> Result<GraphList, GraphError> {
    let path = format!("{}/owners", group_path(&input.group_id)?);
    list_page(graph, &path, select(USER_FIELDS), input.skip, None).await
}

pub async fn list_group_conversations(
    graph: &GraphClient,
    input: GroupConversationsInput,
) -> Result<GraphList, GraphError> {
    let path = format!("{}/conversations", group_path(&input.group_id)?);
    list_page(graph, &path, GraphQuery::new(), input.skip, Some(input.top)).await
}

pub async fn list_group_threads(
    graph: &GraphClient,
    input: GroupThreadsInput,
) -> Result<GraphList, GraphError> {
    let group = group_path(&input.group_id)?;
    let conversation = validate_id(&input.conversation_id, "conversation_id")?;
    let path = format!("{group}/conversations/{conversation}/threads");
    list_page(graph, &path, GraphQuery::new(), input.skip, Some(input.top)).await
}
