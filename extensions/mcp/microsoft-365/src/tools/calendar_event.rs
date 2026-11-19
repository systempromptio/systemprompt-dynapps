//! Single-event calendar tools: read one event, create, update and delete
//! events, and answer an invitation (accept, decline, tentatively accept).
//!
//! Times travel as Graph's `dateTimeTimeZone` in UTC. An all-day event
//! starts at midnight of its first day and, per Graph, ends at midnight of
//! the day after its last; the Belgian server sent a `date` property here,
//! which `dateTimeTimeZone` does not have. Graph replaces `attendees` and
//! `categories` whole on update, and the tools do the same rather than merge.

use schemars::JsonSchema;
use serde::Deserialize;
// JSON: event bodies are built as the Graph resource they write, and Graph's
// answer passes through as returned.
use serde_json::{Value, json};

use super::calendar::calendar_path;
use super::graph_tool::graph_tools;
use super::reply::GraphItem;
use crate::graph::sanitize::validate_id;
use crate::graph::{GraphClient, GraphError, GraphQuery};

pub const EVENT_DETAIL_FIELDS: &str = "id,subject,body,start,end,location,organizer,attendees,\
    isAllDay,isCancelled,recurrence,responseStatus,onlineMeeting,categories";

const GET_EVENT: &str = "Get the full details of one calendar event, body and attendees included.";
const CREATE_EVENT: &str = "Create a calendar event (in the default calendar unless \
    `calendar_id` is given). Times are UTC, ISO 8601 without a trailing Z. An all-day event \
    uses only the dates, and its end is the day after its last day. `is_online` makes it a \
    Teams meeting.";
const UPDATE_EVENT: &str = "Update a calendar event; only the fields given change. \
    `attendees` and `categories` replace the whole list (pass [] to clear categories): to add \
    one, read the event with d365_get_event and pass the merged list. On one occurrence of a \
    recurring series the change applies to that occurrence only.";
const DELETE_EVENT: &str = "Delete a calendar event.";
const ACCEPT_EVENT: &str = "Accept a calendar event invitation.";
const DECLINE_EVENT: &str = "Decline a calendar event invitation.";
const TENTATIVELY_ACCEPT_EVENT: &str = "Tentatively accept a calendar event invitation.";

graph_tools! {
    "d365_get_event" => get_event, EventIdInput, GraphItem, GET_EVENT;
    "d365_create_event" => create_event, CreateEventInput, GraphItem, CREATE_EVENT;
    "d365_update_event" => update_event, UpdateEventInput, GraphItem, UPDATE_EVENT;
    "d365_delete_event" => delete_event, EventIdInput, GraphItem, DELETE_EVENT;
    "d365_accept_event" => accept_event, RespondEventInput, GraphItem, ACCEPT_EVENT;
    "d365_decline_event" => decline_event, RespondEventInput, GraphItem, DECLINE_EVENT;
    "d365_tentatively_accept_event" => tentatively_accept_event, RespondEventInput, GraphItem,
        TENTATIVELY_ACCEPT_EVENT;
}

const fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct EventIdInput {
    #[schemars(description = "The event ID.")]
    pub event_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateEventInput {
    #[schemars(description = "Event subject / title.")]
    pub subject: String,
    #[schemars(description = "Start, ISO 8601 without a trailing Z (e.g. \
        \"2024-06-15T09:00:00\"); only the date counts for an all-day event.")]
    pub start_datetime: String,
    #[schemars(
        description = "End, ISO 8601 (e.g. \"2024-06-15T10:00:00\"); for an all-day \
        event, the day after its last day."
    )]
    pub end_datetime: String,
    #[schemars(description = "Optional HTML or plain-text body.")]
    #[serde(default)]
    pub body: String,
    #[schemars(description = "Optional location display name.")]
    #[serde(default)]
    pub location: String,
    #[schemars(description = "Optional attendee email addresses.")]
    #[serde(default)]
    pub attendees: Vec<String>,
    #[schemars(description = "Whether this is an all-day event (default false).")]
    #[serde(default)]
    pub is_all_day: bool,
    #[schemars(description = "Whether to create a Teams online meeting (default false).")]
    #[serde(default)]
    pub is_online: bool,
    #[schemars(description = "Calendar ID; the default calendar when omitted.")]
    #[serde(default)]
    pub calendar_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateEventInput {
    #[schemars(description = "The event ID to update.")]
    pub event_id: String,
    #[schemars(description = "New subject / title.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub subject: Option<String>,
    #[schemars(description = "New start, ISO 8601 (UTC).")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub start_datetime: Option<String>,
    #[schemars(description = "New end, ISO 8601 (UTC).")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub end_datetime: Option<String>,
    #[schemars(description = "New HTML or plain-text body.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub body: Option<String>,
    #[schemars(description = "New location display name.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub location: Option<String>,
    #[schemars(description = "New attendee email addresses; replaces the existing ones.")]
    #[serde(default)]
    #[schemars(with = "Vec<String>")]
    pub attendees: Option<Vec<String>>,
    #[schemars(
        description = "New category list; replaces the existing one ([] clears it). \
        A category outside the user's master list still applies, without a colour."
    )]
    #[serde(default)]
    #[schemars(with = "Vec<String>")]
    pub categories: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RespondEventInput {
    #[schemars(description = "The event ID to answer.")]
    pub event_id: String,
    #[schemars(description = "Optional comment sent with the response.")]
    #[serde(default)]
    pub comment: String,
    #[schemars(description = "Whether to send the response to the organiser (default true).")]
    #[serde(default = "default_true")]
    pub send_response: bool,
}

fn utc(datetime: &str) -> Value {
    json!({ "dateTime": datetime, "timeZone": "UTC" })
}

fn all_day(datetime: &str) -> Value {
    let date: String = datetime.chars().take(10).collect();
    utc(&format!("{date}T00:00:00"))
}

fn html(content: &str) -> Value {
    json!({ "contentType": "HTML", "content": content })
}

fn required_attendees(addresses: &[String]) -> Value {
    addresses
        .iter()
        .map(|address| json!({ "emailAddress": { "address": address }, "type": "required" }))
        .collect()
}

pub async fn get_event(graph: &GraphClient, input: EventIdInput) -> Result<GraphItem, GraphError> {
    let event = validate_id(&input.event_id, "event_id")?;
    let query = GraphQuery::new().with("$select", EVENT_DETAIL_FIELDS);
    let data = graph
        .get_json(&format!("/me/events/{event}"), &query)
        .await?;
    Ok(GraphItem::new(data))
}

pub async fn create_event(
    graph: &GraphClient,
    input: CreateEventInput,
) -> Result<GraphItem, GraphError> {
    let path = calendar_path(&input.calendar_id, "events")?;
    let when: fn(&str) -> Value = if input.is_all_day { all_day } else { utc };
    let mut event = json!({
        "subject": &input.subject,
        "start": when(&input.start_datetime),
        "end": when(&input.end_datetime),
        "isAllDay": input.is_all_day,
    });
    if !input.body.is_empty() {
        event["body"] = html(&input.body);
    }
    if !input.location.is_empty() {
        event["location"] = json!({ "displayName": &input.location });
    }
    if !input.attendees.is_empty() {
        event["attendees"] = required_attendees(&input.attendees);
    }
    if input.is_online {
        event["isOnlineMeeting"] = Value::Bool(true);
        event["onlineMeetingProvider"] = json!("teamsForBusiness");
    }
    Ok(GraphItem::new(graph.post_json(&path, event).await?))
}

pub async fn update_event(
    graph: &GraphClient,
    input: UpdateEventInput,
) -> Result<GraphItem, GraphError> {
    let event_id = validate_id(&input.event_id, "event_id")?;
    let mut changes = json!({});
    if let Some(subject) = &input.subject {
        changes["subject"] = json!(subject);
    }
    if let Some(start) = &input.start_datetime {
        changes["start"] = utc(start);
    }
    if let Some(end) = &input.end_datetime {
        changes["end"] = utc(end);
    }
    if let Some(body) = &input.body {
        changes["body"] = html(body);
    }
    if let Some(location) = &input.location {
        changes["location"] = json!({ "displayName": location });
    }
    if let Some(attendees) = &input.attendees {
        changes["attendees"] = required_attendees(attendees);
    }
    if let Some(categories) = &input.categories {
        changes["categories"] = json!(categories);
    }
    let path = format!("/me/events/{event_id}");
    Ok(GraphItem::new(graph.patch_json(&path, changes).await?))
}

pub async fn delete_event(
    graph: &GraphClient,
    input: EventIdInput,
) -> Result<GraphItem, GraphError> {
    let event = validate_id(&input.event_id, "event_id")?;
    graph.delete(&format!("/me/events/{event}")).await?;
    Ok(GraphItem::new(
        json!({ "deleted": true, "event_id": event }),
    ))
}

async fn respond(
    graph: &GraphClient,
    input: RespondEventInput,
    action: &str,
    outcome: &str,
) -> Result<GraphItem, GraphError> {
    let event = validate_id(&input.event_id, "event_id")?;
    let mut answer = json!({ "sendResponse": input.send_response });
    if !input.comment.is_empty() {
        answer["comment"] = json!(&input.comment);
    }
    let path = format!("/me/events/{event}/{action}");
    graph.post_json(&path, answer).await?;
    let mut done = json!({ "event_id": event });
    done[outcome] = Value::Bool(true);
    Ok(GraphItem::new(done))
}

pub async fn accept_event(
    graph: &GraphClient,
    input: RespondEventInput,
) -> Result<GraphItem, GraphError> {
    respond(graph, input, "accept", "accepted").await
}

pub async fn decline_event(
    graph: &GraphClient,
    input: RespondEventInput,
) -> Result<GraphItem, GraphError> {
    respond(graph, input, "decline", "declined").await
}

pub async fn tentatively_accept_event(
    graph: &GraphClient,
    input: RespondEventInput,
) -> Result<GraphItem, GraphError> {
    respond(graph, input, "tentativelyAccept", "tentatively_accepted").await
}
