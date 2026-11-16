//! Microsoft Teams reads: the teams the user has joined, a team's channels
//! and members, and a channel's messages, replies and members. Graph v1.0
//! refuses `$top` and `$skip` on `/me/joinedTeams`, so that list takes no
//! paging; the others page with `skip` (and `top` for messages), walking
//! `@odata.nextLink` where Graph rejects `$skip`, as the Belgian server's
//! `get_collection` did.

use schemars::JsonSchema;
use serde::Deserialize;

use super::graph_tool::graph_tools;
use super::reply::{GraphItem, GraphList};
use crate::graph::sanitize::{DEFAULT_MAX_TOP, clamp_skip, clamp_top, validate_id};
use crate::graph::{GraphClient, GraphError, GraphId, GraphQuery, Window};

pub const TEAM_FIELDS: &str = "id,displayName,description";
pub const CHANNEL_FIELDS: &str = "id,displayName,description,membershipType";
pub const MEMBER_FIELDS: &str =
    "id,displayName,roles,microsoft.graph.aadUserConversationMember/email";

const LIST_TEAMS: &str = "List all Microsoft Teams the signed-in user has joined. Graph \
    returns them all at once: this list takes no `top` or `skip`.";
const LIST_CHANNELS: &str = "List all channels in a team. Page with `skip`.";
const GET_CHANNEL: &str = "Get details of a specific channel.";
const LIST_CHANNEL_MESSAGES: &str =
    "List recent messages in a channel. Page with `top` (1-50) and `skip`.";
const GET_CHANNEL_MESSAGE: &str = "Get a specific message from a channel.";
const LIST_CHANNEL_MESSAGE_REPLIES: &str =
    "List replies to a specific channel message. Page with `skip`.";
const LIST_CHANNEL_MEMBERS: &str = "List members of a specific channel. Page with `skip`.";
const LIST_TEAM_MEMBERS: &str = "List members of a team. Page with `skip`.";

graph_tools! {
    "d365_list_teams" => list_teams, JoinedTeamsInput, GraphList, LIST_TEAMS;
    "d365_list_channels" => list_channels, TeamSkipInput, GraphList, LIST_CHANNELS;
    "d365_get_channel" => get_channel, ChannelIdInput, GraphItem, GET_CHANNEL;
    "d365_list_channel_messages" => list_channel_messages, ChannelMessagesInput, GraphList,
        LIST_CHANNEL_MESSAGES;
    "d365_get_channel_message" => get_channel_message, ChannelMessageIdInput, GraphItem,
        GET_CHANNEL_MESSAGE;
    "d365_list_channel_message_replies" => list_channel_message_replies, ChannelRepliesInput,
        GraphList, LIST_CHANNEL_MESSAGE_REPLIES;
    "d365_list_channel_members" => list_channel_members, ChannelSkipInput, GraphList,
        LIST_CHANNEL_MEMBERS;
    "d365_list_team_members" => list_team_members, TeamSkipInput, GraphList, LIST_TEAM_MEMBERS;
}

const fn default_top() -> i64 {
    25
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct JoinedTeamsInput {}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TeamSkipInput {
    #[schemars(description = "The team ID.")]
    pub team_id: String,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChannelIdInput {
    #[schemars(description = "The team ID.")]
    pub team_id: String,
    #[schemars(description = "The channel ID.")]
    pub channel_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChannelSkipInput {
    #[schemars(description = "The team ID.")]
    pub team_id: String,
    #[schemars(description = "The channel ID.")]
    pub channel_id: String,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChannelMessagesInput {
    #[schemars(description = "The team ID.")]
    pub team_id: String,
    #[schemars(description = "The channel ID.")]
    pub channel_id: String,
    #[schemars(description = "Maximum number of messages to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChannelMessageIdInput {
    #[schemars(description = "The team ID.")]
    pub team_id: String,
    #[schemars(description = "The channel ID.")]
    pub channel_id: String,
    #[schemars(description = "The message ID.")]
    pub message_id: GraphId,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ChannelRepliesInput {
    #[schemars(description = "The team ID.")]
    pub team_id: String,
    #[schemars(description = "The channel ID.")]
    pub channel_id: String,
    #[schemars(description = "The parent message ID.")]
    pub message_id: GraphId,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

fn team_path(team_id: &str) -> Result<String, GraphError> {
    let team = validate_id(team_id, "team_id")?;
    Ok(format!("/teams/{team}"))
}

pub(super) fn channel_path(team_id: &str, channel_id: &str) -> Result<String, GraphError> {
    let team = team_path(team_id)?;
    let channel = validate_id(channel_id, "channel_id")?;
    Ok(format!("{team}/channels/{channel}"))
}

pub(super) fn channel_message_path(
    team_id: &str,
    channel_id: &str,
    message: &str,
) -> Result<String, GraphError> {
    let channel = channel_path(team_id, channel_id)?;
    let message = validate_id(message, "message_id")?;
    Ok(format!("{channel}/messages/{message}"))
}

pub(super) fn select(fields: &str) -> GraphQuery {
    GraphQuery::new().with("$select", fields)
}

pub(super) async fn list_page(
    graph: &GraphClient,
    path: &str,
    query: GraphQuery,
    skip: i64,
    top: Option<i64>,
) -> Result<GraphList, GraphError> {
    let window = Window {
        skip: clamp_skip(skip),
        top: top.map(|top| clamp_top(top, DEFAULT_MAX_TOP)),
    };
    let page = graph.get_collection(path, &query, window).await?;
    Ok(GraphList::from(page))
}

pub async fn list_teams(
    graph: &GraphClient,
    _input: JoinedTeamsInput,
) -> Result<GraphList, GraphError> {
    list_page(graph, "/me/joinedTeams", select(TEAM_FIELDS), 0, None).await
}

pub async fn list_channels(
    graph: &GraphClient,
    input: TeamSkipInput,
) -> Result<GraphList, GraphError> {
    let path = format!("{}/channels", team_path(&input.team_id)?);
    list_page(graph, &path, select(CHANNEL_FIELDS), input.skip, None).await
}

pub async fn get_channel(
    graph: &GraphClient,
    input: ChannelIdInput,
) -> Result<GraphItem, GraphError> {
    let path = channel_path(&input.team_id, &input.channel_id)?;
    let data = graph.get_json(&path, &GraphQuery::new()).await?;
    Ok(GraphItem::new(data))
}

pub async fn list_channel_messages(
    graph: &GraphClient,
    input: ChannelMessagesInput,
) -> Result<GraphList, GraphError> {
    let path = format!(
        "{}/messages",
        channel_path(&input.team_id, &input.channel_id)?
    );
    list_page(graph, &path, GraphQuery::new(), input.skip, Some(input.top)).await
}

pub async fn get_channel_message(
    graph: &GraphClient,
    input: ChannelMessageIdInput,
) -> Result<GraphItem, GraphError> {
    let path = channel_message_path(&input.team_id, &input.channel_id, &input.message_id)?;
    let data = graph.get_json(&path, &GraphQuery::new()).await?;
    Ok(GraphItem::new(data))
}

pub async fn list_channel_message_replies(
    graph: &GraphClient,
    input: ChannelRepliesInput,
) -> Result<GraphList, GraphError> {
    let message = channel_message_path(&input.team_id, &input.channel_id, &input.message_id)?;
    let path = format!("{message}/replies");
    list_page(graph, &path, GraphQuery::new(), input.skip, None).await
}

pub async fn list_channel_members(
    graph: &GraphClient,
    input: ChannelSkipInput,
) -> Result<GraphList, GraphError> {
    let path = format!(
        "{}/members",
        channel_path(&input.team_id, &input.channel_id)?
    );
    list_page(graph, &path, select(MEMBER_FIELDS), input.skip, None).await
}

pub async fn list_team_members(
    graph: &GraphClient,
    input: TeamSkipInput,
) -> Result<GraphList, GraphError> {
    let path = format!("{}/members", team_path(&input.team_id)?);
    list_page(graph, &path, select(MEMBER_FIELDS), input.skip, None).await
}
