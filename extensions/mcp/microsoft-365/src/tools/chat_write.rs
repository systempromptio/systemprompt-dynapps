//! Teams chat writes: send a message, start a one-on-one or group chat, pin
//! and unpin a message, and react to one.
//!
//! A new chat lists the caller too (Graph requires every member, the caller
//! included), so its Graph id is read from `/me` first; members and pinned
//! messages are bound by URL under the configured Graph root, which is
//! Microsoft's in production.

use schemars::JsonSchema;
use serde::Deserialize;
// JSON: chat, member and pin bodies are built as the Graph resource they
// write, and Graph's answer passes through as returned.
use serde_json::{Value, json};

use super::chat::chat_path;
use super::graph_tool::graph_tools;
use super::reply::GraphItem;
use super::teams::select;
use super::teams_write::{message_body, noted};
use crate::graph::sanitize::validate_id;
use crate::graph::{GraphClient, GraphError, GraphId};

const NO_CALLER: &str = "Could not determine caller id from /me";

const SEND_CHAT_MESSAGE: &str =
    "Send a message in an existing chat, as plain text or (with `is_html`) HTML.";
const CREATE_ONE_ON_ONE_CHAT: &str = "Create a 1:1 chat with another user.";
const CREATE_GROUP_CHAT: &str =
    "Create a group chat with several users (the caller is added automatically).";
const PIN_CHAT_MESSAGE: &str = "Pin a message in a chat.";
const UNPIN_CHAT_MESSAGE: &str = "Unpin a pinned message from a chat (by its pinned-message ID).";
const SET_CHAT_MESSAGE_REACTION: &str =
    "Set a reaction on a chat message: like, angry, sad, laugh, heart or surprised.";

graph_tools! {
    "d365_send_chat_message" => send_chat_message, SendChatMessageInput, GraphItem,
        SEND_CHAT_MESSAGE;
    "d365_create_one_on_one_chat" => create_one_on_one_chat, OneOnOneChatInput, GraphItem,
        CREATE_ONE_ON_ONE_CHAT;
    "d365_create_group_chat" => create_group_chat, GroupChatInput, GraphItem, CREATE_GROUP_CHAT;
    "d365_pin_chat_message" => pin_chat_message, ChatMessageIdInput, GraphItem, PIN_CHAT_MESSAGE;
    "d365_unpin_chat_message" => unpin_chat_message, UnpinChatMessageInput, GraphItem,
        UNPIN_CHAT_MESSAGE;
    "d365_set_chat_message_reaction" => set_chat_message_reaction, ChatReactionInput, GraphItem,
        SET_CHAT_MESSAGE_REACTION;
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SendChatMessageInput {
    #[schemars(description = "The chat ID.")]
    pub chat_id: String,
    #[schemars(description = "The message content.")]
    pub body: String,
    #[schemars(description = "If true, send as HTML; otherwise plain text.")]
    #[serde(default)]
    pub is_html: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct OneOnOneChatInput {
    #[schemars(description = "The Azure AD user ID of the other participant.")]
    pub user_id: GraphId,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GroupChatInput {
    #[schemars(description = "Azure AD user IDs to include (the caller is added automatically).")]
    pub member_ids: Vec<String>,
    #[schemars(description = "Optional display topic for the group chat.")]
    #[serde(default)]
    pub topic: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChatMessageIdInput {
    #[schemars(description = "The chat ID.")]
    pub chat_id: String,
    #[schemars(description = "The ID of the message.")]
    pub message_id: GraphId,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UnpinChatMessageInput {
    #[schemars(description = "The chat ID.")]
    pub chat_id: String,
    #[schemars(description = "The pinned-message resource ID (not the original message ID).")]
    pub pinned_message_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChatReactionInput {
    #[schemars(description = "The chat ID.")]
    pub chat_id: String,
    #[schemars(description = "The message ID.")]
    pub message_id: GraphId,
    #[schemars(description = "Reaction to set: like, angry, sad, laugh, heart or surprised.")]
    pub reaction_type: String,
}

fn graph_root(graph: &GraphClient) -> &str {
    graph.endpoint().base().as_str().trim_end_matches('/')
}

// JSON: `/me` is read for one field of Graph's user resource.
async fn caller_id(graph: &GraphClient) -> Result<Option<String>, GraphError> {
    let me = graph.get_json("/me", &select("id")).await?;
    Ok(me
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .map(str::to_owned))
}

// JSON: an aadUserConversationMember as Graph takes it.
fn member(root: &str, user: &str) -> Value {
    json!({
        "@odata.type": "#microsoft.graph.aadUserConversationMember",
        "roles": ["owner"],
        "user@odata.bind": format!("{root}/users('{user}')"),
    })
}

pub async fn send_chat_message(
    graph: &GraphClient,
    input: SendChatMessageInput,
) -> Result<GraphItem, GraphError> {
    let path = format!("{}/messages", chat_path(&input.chat_id)?);
    let data = graph
        .post_json(&path, message_body(&input.body, input.is_html))
        .await?;
    Ok(GraphItem::new(data))
}

pub async fn create_one_on_one_chat(
    graph: &GraphClient,
    input: OneOnOneChatInput,
) -> Result<GraphItem, GraphError> {
    let user = validate_id(&input.user_id, "user_id")?;
    let Some(caller) = caller_id(graph).await? else {
        return Ok(noted(NO_CALLER.to_owned()));
    };
    let root = graph_root(graph);
    let chat = json!({
        "chatType": "oneOnOne",
        "members": [member(root, &caller), member(root, user)],
    });
    let data = graph.post_json("/chats", chat).await?;
    Ok(GraphItem::new(data))
}

pub async fn create_group_chat(
    graph: &GraphClient,
    input: GroupChatInput,
) -> Result<GraphItem, GraphError> {
    let Some(caller) = caller_id(graph).await? else {
        return Ok(noted(NO_CALLER.to_owned()));
    };
    let others = input
        .member_ids
        .iter()
        .filter(|id| **id != caller)
        .map(|id| validate_id(id, "user_id"))
        .collect::<Result<Vec<_>, _>>()?;
    let root = graph_root(graph);
    let members: Vec<Value> = std::iter::once(caller.as_str())
        .chain(others)
        .map(|id| member(root, id))
        .collect();
    let mut chat = json!({ "chatType": "group", "members": members });
    if !input.topic.is_empty() {
        chat["topic"] = json!(input.topic);
    }
    let data = graph.post_json("/chats", chat).await?;
    Ok(GraphItem::new(data))
}

pub async fn pin_chat_message(
    graph: &GraphClient,
    input: ChatMessageIdInput,
) -> Result<GraphItem, GraphError> {
    let chat = validate_id(&input.chat_id, "chat_id")?;
    let message = validate_id(&input.message_id, "message_id")?;
    let bind = format!("{}/chats/{chat}/messages/{message}", graph_root(graph));
    let pin = json!({ "message@odata.bind": bind });
    let path = format!("{}/pinnedMessages", chat_path(chat)?);
    let data = graph.post_json(&path, pin).await?;
    Ok(GraphItem::new(data))
}

pub async fn unpin_chat_message(
    graph: &GraphClient,
    input: UnpinChatMessageInput,
) -> Result<GraphItem, GraphError> {
    let pinned = validate_id(&input.pinned_message_id, "pinned_message_id")?;
    let path = format!("{}/pinnedMessages/{pinned}", chat_path(&input.chat_id)?);
    graph.delete(&path).await?;
    Ok(noted("Message unpinned".to_owned()))
}

pub async fn set_chat_message_reaction(
    graph: &GraphClient,
    input: ChatReactionInput,
) -> Result<GraphItem, GraphError> {
    let message = validate_id(&input.message_id, "message_id")?;
    let path = format!(
        "{}/messages/{message}/setReaction",
        chat_path(&input.chat_id)?
    );
    let message = format!("Reaction '{}' set", input.reaction_type);
    graph
        .post_json(&path, json!({ "reactionType": input.reaction_type }))
        .await?;
    Ok(noted(message))
}
