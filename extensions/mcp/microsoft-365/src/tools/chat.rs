//! Teams chat reads: the user's chats with their members, one chat, and a
//! chat's messages and members. `/me/chats` expands `members`, whose
//! `$select` Graph ignores, so each member is cut down here to its id, name,
//! roles and email, as the Belgian server's `_trim_members` did.

use schemars::JsonSchema;
use serde::Deserialize;
// JSON: Graph's chat resources pass through as returned; only the members
// of a listed chat are trimmed.
use serde_json::Value;

use super::graph_tool::graph_tools;
use super::reply::{GraphItem, GraphList};
use super::teams::{MEMBER_FIELDS, list_page, select};
use crate::graph::sanitize::validate_id;
use crate::graph::{GraphClient, GraphError, GraphQuery};

pub const CHAT_FIELDS: &str = "id,topic,chatType,createdDateTime,lastUpdatedDateTime";
pub const CHAT_MESSAGE_FIELDS: &str = "id,body,from,createdDateTime,importance,messageType";
const MEMBER_KEEP: [&str; 4] = ["id", "displayName", "roles", "email"];

const LIST_CHATS: &str = "List the signed-in user's chats with their members, ordered by \
    most recent activity. Page with `top` (1-50) and `skip`.";
const GET_CHAT: &str = "Get details of a specific chat.";
const LIST_CHAT_MESSAGES: &str = "List messages in a chat. Page with `top` (1-50) and `skip`.";
const LIST_CHAT_MEMBERS: &str = "List members of a chat. Page with `skip`.";

graph_tools! {
    "d365_list_chats" => list_chats, ListChatsInput, GraphList, LIST_CHATS;
    "d365_get_chat" => get_chat, ChatIdInput, GraphItem, GET_CHAT;
    "d365_list_chat_messages" => list_chat_messages, ChatMessagesInput, GraphList,
        LIST_CHAT_MESSAGES;
    "d365_list_chat_members" => list_chat_members, ChatMembersInput, GraphList,
        LIST_CHAT_MEMBERS;
}

const fn default_top() -> i64 {
    25
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct ListChatsInput {
    #[schemars(description = "Maximum number of chats to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChatIdInput {
    #[schemars(description = "The chat ID.")]
    pub chat_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChatMessagesInput {
    #[schemars(description = "The chat ID.")]
    pub chat_id: String,
    #[schemars(description = "Maximum number of messages to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChatMembersInput {
    #[schemars(description = "The chat ID.")]
    pub chat_id: String,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

pub(super) fn chat_path(chat_id: &str) -> Result<String, GraphError> {
    let chat = validate_id(chat_id, "chat_id")?;
    Ok(format!("/me/chats/{chat}"))
}

// JSON: a chat resource from Graph; only its `members` array is reshaped.
fn trim_members(chat: &mut Value) {
    let Some(members) = chat.get_mut("members").and_then(Value::as_array_mut) else {
        return;
    };
    for member in members.iter_mut().filter_map(Value::as_object_mut) {
        member.retain(|key, _| MEMBER_KEEP.contains(&key.as_str()));
    }
}

pub async fn list_chats(
    graph: &GraphClient,
    input: ListChatsInput,
) -> Result<GraphList, GraphError> {
    let query = select(CHAT_FIELDS).with("$expand", "members");
    let mut list = list_page(graph, "/me/chats", query, input.skip, Some(input.top)).await?;
    list.data.iter_mut().for_each(trim_members);
    Ok(list)
}

pub async fn get_chat(graph: &GraphClient, input: ChatIdInput) -> Result<GraphItem, GraphError> {
    let path = chat_path(&input.chat_id)?;
    let data = graph.get_json(&path, &GraphQuery::new()).await?;
    Ok(GraphItem::new(data))
}

pub async fn list_chat_messages(
    graph: &GraphClient,
    input: ChatMessagesInput,
) -> Result<GraphList, GraphError> {
    let path = format!("{}/messages", chat_path(&input.chat_id)?);
    let query = select(CHAT_MESSAGE_FIELDS);
    list_page(graph, &path, query, input.skip, Some(input.top)).await
}

pub async fn list_chat_members(
    graph: &GraphClient,
    input: ChatMembersInput,
) -> Result<GraphList, GraphError> {
    let path = format!("{}/members", chat_path(&input.chat_id)?);
    list_page(graph, &path, select(MEMBER_FIELDS), input.skip, None).await
}
