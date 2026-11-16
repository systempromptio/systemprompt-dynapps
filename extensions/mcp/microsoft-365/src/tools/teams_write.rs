//! Microsoft Teams writes: post a message to a channel, reply to one, and
//! create or update a channel. A message is plain text unless `is_html`.
//! A write Graph answers with an empty body comes back as the Belgian
//! server's `success_response(None, message)`: `data` null and a message.

use schemars::JsonSchema;
use serde::Deserialize;
// JSON: message and channel bodies are built as the Graph resource they
// write, and Graph's answer passes through as returned.
use serde_json::{Value, json};

use super::graph_tool::graph_tools;
use super::reply::GraphItem;
use super::teams::{channel_message_path, channel_path};
use crate::graph::sanitize::validate_id;
use crate::graph::{GraphClient, GraphError};

const SEND_CHANNEL_MESSAGE: &str =
    "Send a message to a channel, as plain text or (with `is_html`) HTML.";
const REPLY_TO_CHANNEL_MESSAGE: &str =
    "Reply to a specific channel message, as plain text or (with `is_html`) HTML.";
const CREATE_CHANNEL: &str = "Create a new channel in a team (standard, private or shared).";
const UPDATE_CHANNEL: &str =
    "Update a channel's display name or description; only the fields given change.";

graph_tools! {
    "d365_send_channel_message" => send_channel_message, SendChannelMessageInput, GraphItem,
        SEND_CHANNEL_MESSAGE;
    "d365_reply_to_channel_message" => reply_to_channel_message, ReplyChannelMessageInput,
        GraphItem, REPLY_TO_CHANNEL_MESSAGE;
    "d365_create_channel" => create_channel, CreateChannelInput, GraphItem, CREATE_CHANNEL;
    "d365_update_channel" => update_channel, UpdateChannelInput, GraphItem, UPDATE_CHANNEL;
}

fn default_membership() -> String {
    "standard".to_owned()
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SendChannelMessageInput {
    #[schemars(description = "The team ID.")]
    pub team_id: String,
    #[schemars(description = "The channel ID.")]
    pub channel_id: String,
    #[schemars(description = "The message content.")]
    pub body: String,
    #[schemars(description = "If true, send as HTML; otherwise plain text.")]
    #[serde(default)]
    pub is_html: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReplyChannelMessageInput {
    #[schemars(description = "The team ID.")]
    pub team_id: String,
    #[schemars(description = "The channel ID.")]
    pub channel_id: String,
    #[schemars(description = "The parent message ID to reply to.")]
    pub message_id: String,
    #[schemars(description = "The reply content.")]
    pub body: String,
    #[schemars(description = "If true, send as HTML; otherwise plain text.")]
    #[serde(default)]
    pub is_html: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateChannelInput {
    #[schemars(description = "The team ID.")]
    pub team_id: String,
    #[schemars(description = "Display name for the new channel.")]
    pub display_name: String,
    #[schemars(description = "Optional channel description.")]
    #[serde(default)]
    pub description: String,
    #[schemars(description = "Channel type: \"standard\" (default), \"private\" or \"shared\".")]
    #[serde(default = "default_membership")]
    pub membership_type: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateChannelInput {
    #[schemars(description = "The team ID.")]
    pub team_id: String,
    #[schemars(description = "The channel ID.")]
    pub channel_id: String,
    #[schemars(description = "New display name (optional).")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub display_name: Option<String>,
    #[schemars(description = "New description (optional).")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub description: Option<String>,
}

// JSON: a chatMessage as Graph takes it, shared by channel and chat posts.
pub(super) fn message_body(content: String, is_html: bool) -> Value {
    let kind = if is_html { "html" } else { "text" };
    json!({ "body": { "content": content, "contentType": kind } })
}

// JSON: `data` is null, as the Belgian `success_response(None, message)`.
pub(super) const fn noted(message: String) -> GraphItem {
    GraphItem {
        data: Value::Null,
        message,
    }
}

pub async fn send_channel_message(
    graph: &GraphClient,
    input: SendChannelMessageInput,
) -> Result<GraphItem, GraphError> {
    let path = format!(
        "{}/messages",
        channel_path(&input.team_id, &input.channel_id)?
    );
    let data = graph
        .post_json(&path, message_body(input.body, input.is_html))
        .await?;
    Ok(GraphItem::new(data))
}

pub async fn reply_to_channel_message(
    graph: &GraphClient,
    input: ReplyChannelMessageInput,
) -> Result<GraphItem, GraphError> {
    let message = channel_message_path(&input.team_id, &input.channel_id, &input.message_id)?;
    let path = format!("{message}/replies");
    let data = graph
        .post_json(&path, message_body(input.body, input.is_html))
        .await?;
    Ok(GraphItem::new(data))
}

pub async fn create_channel(
    graph: &GraphClient,
    input: CreateChannelInput,
) -> Result<GraphItem, GraphError> {
    let team = validate_id(&input.team_id, "team_id")?;
    let mut channel = json!({
        "displayName": input.display_name,
        "membershipType": input.membership_type,
    });
    if !input.description.is_empty() {
        channel["description"] = json!(input.description);
    }
    let data = graph
        .post_json(&format!("/teams/{team}/channels"), channel)
        .await?;
    Ok(GraphItem::new(data))
}

pub async fn update_channel(
    graph: &GraphClient,
    input: UpdateChannelInput,
) -> Result<GraphItem, GraphError> {
    let path = channel_path(&input.team_id, &input.channel_id)?;
    let mut patch = json!({});
    if let Some(name) = input.display_name {
        patch["displayName"] = json!(name);
    }
    if let Some(description) = input.description {
        patch["description"] = json!(description);
    }
    let data = graph.patch_json(&path, patch).await?;
    Ok(GraphItem::new(data))
}
