//! Teams online meetings: find one by its join URL, read, create and delete
//! one.
//!
//! Graph will not enumerate `/me/onlineMeetings`: it needs
//! `$filter=JoinWebUrl eq '…'` (with `ConsistencyLevel: eventual`) and
//! refuses `$top`, `$skip` and `$select` there, so
//! `d365_list_online_meetings` and `d365_get_meeting_by_join_url` both answer
//! the first match, or a null with a message, as the Belgian server did.

use reqwest::Method;
use schemars::JsonSchema;
use serde::Deserialize;
// JSON: meeting bodies are built as the Graph resource they write, and
// Graph's answer passes through as returned.
use serde_json::{Value, json};

use super::graph_tool::graph_tools;
use super::reply::GraphItem;
use super::teams::select;
use super::teams_write::noted;
use crate::graph::sanitize::{sanitize_odata_string, validate_id};
use crate::graph::{GraphClient, GraphError, GraphQuery, GraphRequest};

pub const MEETINGS: &str = "/me/onlineMeetings";
pub const MEETING_DETAIL_FIELDS: &str = "id,subject,startDateTime,endDateTime,joinWebUrl,\
    participants,chatInfo,videoTeleconferenceId";
const NO_MATCH: &str = "No meeting found for the given join URL";

const LIST_ONLINE_MEETINGS: &str = "Look up an online meeting by its Teams join URL. Graph \
    cannot list `/me/onlineMeetings` without that filter, so this takes a `join_url` and \
    returns the matching meeting (or none). To find meetings by subject, participants or \
    time, use d365_list_events or d365_list_calendar_view: scheduled Teams meetings appear \
    there with their `onlineMeeting` property.";
const GET_ONLINE_MEETING: &str = "Get details of a specific online meeting.";
const GET_MEETING_BY_JOIN_URL: &str = "Find an online meeting by its join URL.";
const CREATE_ONLINE_MEETING: &str = "Create a new online (Teams) meeting; the signed-in user \
    organises it and `participants` (email addresses) join as attendees.";
const DELETE_ONLINE_MEETING: &str = "Delete an online meeting.";

graph_tools! {
    "d365_list_online_meetings" => meeting_by_join_url, JoinUrlInput, GraphItem,
        LIST_ONLINE_MEETINGS;
    "d365_get_online_meeting" => get_online_meeting, MeetingIdInput, GraphItem,
        GET_ONLINE_MEETING;
    "d365_get_meeting_by_join_url" => meeting_by_join_url, JoinUrlInput, GraphItem,
        GET_MEETING_BY_JOIN_URL;
    "d365_create_online_meeting" => create_online_meeting, CreateOnlineMeetingInput, GraphItem,
        CREATE_ONLINE_MEETING;
    "d365_delete_online_meeting" => delete_online_meeting, MeetingIdInput, GraphItem,
        DELETE_ONLINE_MEETING;
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct JoinUrlInput {
    #[schemars(
        description = "The Teams meeting join URL (e.g. https://teams.microsoft.com/l/meetup-join/...)."
    )]
    pub join_url: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MeetingIdInput {
    #[schemars(description = "The online meeting ID.")]
    pub meeting_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateOnlineMeetingInput {
    #[schemars(description = "Meeting subject / title.")]
    pub subject: String,
    #[schemars(description = "Start in ISO 8601 format (e.g. '2024-06-15T09:00:00Z').")]
    pub start_datetime: String,
    #[schemars(description = "End in ISO 8601 format (e.g. '2024-06-15T10:00:00Z').")]
    pub end_datetime: String,
    #[schemars(description = "Optional participant email addresses.")]
    #[serde(default)]
    pub participants: Vec<String>,
}

pub(super) fn meeting_path(meeting_id: &str) -> Result<String, GraphError> {
    let meeting = validate_id(meeting_id, "meeting_id")?;
    Ok(format!("{MEETINGS}/{meeting}"))
}

// JSON: Graph's onlineMeeting collection; the first match passes through.
pub async fn meeting_by_join_url(
    graph: &GraphClient,
    input: JoinUrlInput,
) -> Result<GraphItem, GraphError> {
    let filter = format!("JoinWebUrl eq '{}'", sanitize_odata_string(&input.join_url));
    let request = GraphRequest::new(Method::GET, MEETINGS)
        .with_query(GraphQuery::new().with("$filter", filter))
        .with_header("ConsistencyLevel", "eventual");
    let mut found = graph.send(&request).await?.json()?;
    let first = found
        .get_mut("value")
        .and_then(Value::as_array_mut)
        .filter(|meetings| !meetings.is_empty())
        .map(|meetings| meetings.swap_remove(0));
    Ok(first.map_or_else(|| noted(NO_MATCH.to_owned()), GraphItem::new))
}

pub async fn get_online_meeting(
    graph: &GraphClient,
    input: MeetingIdInput,
) -> Result<GraphItem, GraphError> {
    let path = meeting_path(&input.meeting_id)?;
    let data = graph
        .get_json(&path, &select(MEETING_DETAIL_FIELDS))
        .await?;
    Ok(GraphItem::new(data))
}

// Why: Graph makes the caller of `/me/onlineMeetings` its organizer and has
// no `organizer` attendee role, so the caller is not listed (the Belgian
// server added it as one).
pub async fn create_online_meeting(
    graph: &GraphClient,
    input: CreateOnlineMeetingInput,
) -> Result<GraphItem, GraphError> {
    let mut meeting = json!({
        "subject": input.subject,
        "startDateTime": input.start_datetime,
        "endDateTime": input.end_datetime,
    });
    if !input.participants.is_empty() {
        let attendees: Vec<Value> = input
            .participants
            .into_iter()
            .map(|upn| json!({ "upn": upn, "role": "attendee" }))
            .collect();
        meeting["participants"] = json!({ "attendees": attendees });
    }
    let data = graph.post_json(MEETINGS, meeting).await?;
    Ok(GraphItem::new(data))
}

pub async fn delete_online_meeting(
    graph: &GraphClient,
    input: MeetingIdInput,
) -> Result<GraphItem, GraphError> {
    let path = meeting_path(&input.meeting_id)?;
    graph.delete(&path).await?;
    Ok(GraphItem::new(json!({
        "deleted": true,
        "meeting_id": input.meeting_id,
    })))
}
