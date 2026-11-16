//! Calendar list tools: the user's calendars, a calendar's events, a
//! calendar view (recurring series expanded) and another user's calendar,
//! with the Belgian server's filters, fields and paging.
//!
//! Graph takes no `$search` beside `$filter` on events, so structured
//! filters (dates, attendee) win and the KQL `query` only applies alone. An
//! attendee filter is an `any(...)` lambda, an advanced query that needs
//! `ConsistencyLevel: eventual`. An event's `start/dateTime` is a string in
//! Graph, so date bounds are quoted literals here, unlike mail's
//! `receivedDateTime`; a view's range travels as `startDateTime` and
//! `endDateTime` instead of a filter.

use schemars::JsonSchema;
use serde::Deserialize;

use super::graph_tool::graph_tools;
use super::reply::GraphList;
use crate::graph::sanitize::{
    DEFAULT_MAX_TOP, clamp_skip, clamp_top, sanitize_odata_string, sanitize_search,
    sanitize_select, validate_id,
};
use crate::graph::{GraphClient, GraphError, GraphQuery, Window};

pub const CALENDAR_FIELDS: &str = "id,name,color,isDefaultCalendar,canEdit,owner";
pub const EVENT_LIST_FIELDS: &str = "id,subject,start,end,location,organizer,isAllDay,\
    isCancelled,responseStatus,onlineMeeting";
pub const CALENDAR_VIEW_FIELDS: &str = "id,subject,start,end,location,organizer,attendees,\
    categories,isAllDay,isCancelled,responseStatus,onlineMeeting,type,seriesMasterId";

const EVENTUAL: [(&str, &str); 1] = [("ConsistencyLevel", "eventual")];
const START_FIELD: &str = "start/dateTime";

const LIST_CALENDARS: &str = "List all calendars of the signed-in user. Page with `skip`.";
const LIST_EVENTS: &str = "List events from a calendar (the default one unless `calendar_id` is \
    given), ordered by start. `start_date`, `end_date` and `attendee` filter server-side and \
    combine; `query` is a KQL search (e.g. \"subject:budget\") that Graph cannot combine with \
    them, so it is ignored when any of them is set. Page with `top` (1-50) and `skip`.";
const LIST_CALENDAR_VIEW: &str = "List a calendar view: every event between two datetimes with \
    recurring series expanded into instances (`type` is singleInstance, occurrence or \
    exception, with `seriesMasterId`). Each event carries `attendees` (address, name, type and \
    response) and `categories`, so one call answers who attends what over a range; bodies are \
    left out (use d365_get_event). `select` trims the fields, e.g. \
    \"id,subject,start,end,categories\" on a busy month. `query` (KQL) is ignored when \
    `attendee` is set. Page with `top` (1-50) and `skip`.";
const LIST_SHARED_CALENDAR_EVENTS: &str = "List events from another user's calendar (needs \
    delegated access to it). With both `start_date` and `end_date` it reads a calendar view, \
    expanding recurring events; otherwise it lists events with the optional date filters. \
    `query` (KQL) is ignored when dates or `attendee` are set. Page with `top` (1-50) and \
    `skip`.";

graph_tools! {
    "d365_list_calendars" => list_calendars, ListCalendarsInput, GraphList, LIST_CALENDARS;
    "d365_list_events" => list_events, ListEventsInput, GraphList, LIST_EVENTS;
    "d365_list_calendar_view" => list_calendar_view, ListCalendarViewInput, GraphList,
        LIST_CALENDAR_VIEW;
    "d365_list_shared_calendar_events" => list_shared_calendar_events,
        ListSharedCalendarEventsInput, GraphList, LIST_SHARED_CALENDAR_EVENTS;
}

const fn default_top() -> i64 {
    25
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct ListCalendarsInput {
    #[schemars(description = "Items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListEventsInput {
    #[schemars(description = "Calendar ID; the default calendar when omitted.")]
    #[serde(default)]
    pub calendar_id: String,
    #[schemars(description = "From this ISO 8601 date or datetime (e.g. \"2024-01-01\").")]
    #[serde(default)]
    pub start_date: String,
    #[schemars(description = "Up to this ISO 8601 date or datetime (e.g. \"2024-12-31\").")]
    #[serde(default)]
    pub end_date: String,
    #[schemars(description = "Only events with this email address among the attendees.")]
    #[serde(default)]
    pub attendee: String,
    #[schemars(description = "KQL search (e.g. \"subject:budget\"); ignored beside filters.")]
    #[serde(default)]
    pub query: String,
    #[schemars(description = "Number of events to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListCalendarViewInput {
    #[schemars(description = "Start of the range, ISO 8601 (e.g. \"2024-06-01T00:00:00Z\").")]
    pub start_datetime: String,
    #[schemars(description = "End of the range, ISO 8601 (e.g. \"2024-06-30T23:59:59Z\").")]
    pub end_datetime: String,
    #[schemars(description = "Calendar ID; the default calendar when omitted.")]
    #[serde(default)]
    pub calendar_id: String,
    #[schemars(description = "Only events with this email address among the attendees.")]
    #[serde(default)]
    pub attendee: String,
    #[schemars(description = "KQL search (e.g. \"subject:budget\"); ignored beside `attendee`.")]
    #[serde(default)]
    pub query: String,
    #[schemars(description = "Comma-separated property names replacing the default fields.")]
    #[serde(default)]
    pub select: String,
    #[schemars(description = "Number of events to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListSharedCalendarEventsInput {
    #[schemars(description = "User ID or UPN of the calendar's owner.")]
    pub user_id: String,
    #[schemars(description = "Events starting on or after this ISO 8601 date or datetime.")]
    #[serde(default)]
    pub start_date: String,
    #[schemars(description = "Events starting on or before this ISO 8601 date or datetime.")]
    #[serde(default)]
    pub end_date: String,
    #[schemars(description = "Only events with this email address among the attendees.")]
    #[serde(default)]
    pub attendee: String,
    #[schemars(description = "KQL search (e.g. \"subject:budget\"); ignored beside filters.")]
    #[serde(default)]
    pub query: String,
    #[schemars(description = "Number of events to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

pub fn event_filters(start_date: &str, end_date: &str, attendee: &str) -> Vec<String> {
    let mut filters = Vec::new();
    if !start_date.is_empty() {
        let start = sanitize_odata_string(start_date);
        filters.push(format!("{START_FIELD} ge '{start}'"));
    }
    if !end_date.is_empty() {
        let end = sanitize_odata_string(end_date);
        filters.push(format!("{START_FIELD} le '{end}'"));
    }
    if !attendee.is_empty() {
        let address = sanitize_odata_string(attendee);
        filters.push(format!(
            "attendees/any(a: a/emailAddress/address eq '{address}')"
        ));
    }
    filters
}

fn search_or_filter(
    query: &mut GraphQuery,
    filters: &[String],
    search: &str,
) -> &'static [(&'static str, &'static str)] {
    if !filters.is_empty() {
        query.set("$filter", filters.join(" and "));
        let lambda = filters.iter().any(|filter| filter.contains("any("));
        return if lambda { &EVENTUAL } else { &[] };
    }
    if !search.is_empty() {
        query.set("$search", format!("\"{}\"", sanitize_search(search)));
    }
    &[]
}

pub(super) fn calendar_path(calendar_id: &str, collection: &str) -> Result<String, GraphError> {
    if calendar_id.is_empty() {
        return Ok(format!("/me/{collection}"));
    }
    let calendar = validate_id(calendar_id, "calendar_id")?;
    Ok(format!("/me/calendars/{calendar}/{collection}"))
}

fn page_window(top: i64, skip: i64) -> Window {
    Window {
        skip: clamp_skip(skip),
        top: Some(clamp_top(top, DEFAULT_MAX_TOP)),
    }
}

async fn list(
    graph: &GraphClient,
    path: &str,
    query: &GraphQuery,
    headers: &[(&str, &str)],
    window: Window,
) -> Result<GraphList, GraphError> {
    let page = graph
        .get_collection_with_headers(path, query, headers, window)
        .await?;
    Ok(GraphList::from(page))
}

pub async fn list_calendars(
    graph: &GraphClient,
    input: ListCalendarsInput,
) -> Result<GraphList, GraphError> {
    let query = GraphQuery::new().with("$select", CALENDAR_FIELDS);
    let window = Window {
        skip: clamp_skip(input.skip),
        top: None,
    };
    list(graph, "/me/calendars", &query, &[], window).await
}

pub async fn list_events(
    graph: &GraphClient,
    input: ListEventsInput,
) -> Result<GraphList, GraphError> {
    let path = calendar_path(&input.calendar_id, "events")?;
    let mut query = GraphQuery::new()
        .with("$select", EVENT_LIST_FIELDS)
        .with("$orderby", START_FIELD);
    let filters = event_filters(&input.start_date, &input.end_date, &input.attendee);
    let headers = search_or_filter(&mut query, &filters, &input.query);
    let window = page_window(input.top, input.skip);
    list(graph, &path, &query, headers, window).await
}

pub async fn list_calendar_view(
    graph: &GraphClient,
    input: ListCalendarViewInput,
) -> Result<GraphList, GraphError> {
    let path = calendar_path(&input.calendar_id, "calendarView")?;
    let fields = if input.select.is_empty() {
        CALENDAR_VIEW_FIELDS.to_owned()
    } else {
        sanitize_select(&input.select, "select")?
    };
    let mut query = GraphQuery::new()
        .with("startDateTime", input.start_datetime.as_str())
        .with("endDateTime", input.end_datetime.as_str())
        .with("$select", fields)
        .with("$orderby", START_FIELD);
    let filters = event_filters("", "", &input.attendee);
    let headers = search_or_filter(&mut query, &filters, &input.query);
    let window = page_window(input.top, input.skip);
    list(graph, &path, &query, headers, window).await
}

pub async fn list_shared_calendar_events(
    graph: &GraphClient,
    input: ListSharedCalendarEventsInput,
) -> Result<GraphList, GraphError> {
    let user = validate_id(&input.user_id, "user_id")?;
    let view = !input.start_date.is_empty() && !input.end_date.is_empty();
    let mut query = GraphQuery::new();
    let (path, filters) = if view {
        query.set("startDateTime", input.start_date.as_str());
        query.set("endDateTime", input.end_date.as_str());
        let filters = event_filters("", "", &input.attendee);
        (format!("/users/{user}/calendarView"), filters)
    } else {
        let filters = event_filters(&input.start_date, &input.end_date, &input.attendee);
        (format!("/users/{user}/events"), filters)
    };
    query.set("$select", EVENT_LIST_FIELDS);
    query.set("$orderby", START_FIELD);
    let headers = search_or_filter(&mut query, &filters, &input.query);
    let window = page_window(input.top, input.skip);
    list(graph, &path, &query, headers, window).await
}
