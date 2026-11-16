//! Microsoft Search across the user's content (`/search/query`), the same
//! search narrowed to mail, and `d365_batch`: up to 20 Graph requests in one
//! `$batch` call, which Graph runs as the signed-in user, relative to its
//! own version root.

use schemars::JsonSchema;
use serde::Deserialize;
// JSON: search hits and batch responses are Graph resources; a batch request
// is the caller's own Graph request object, passed through.
use serde_json::{Map, Value, json};

use super::graph_tool::graph_tools;
use super::reply::GraphItem;
use super::teams_write::noted;
use crate::graph::sanitize::{DEFAULT_MAX_TOP, clamp_top, sanitize_search};
use crate::graph::{GraphClient, GraphError};

pub const MAX_BATCH_REQUESTS: usize = 20;
const BATCH_METHODS: [&str; 5] = ["GET", "POST", "PATCH", "DELETE", "PUT"];
const DEFAULT_ENTITY_TYPES: [&str; 4] = ["message", "driveItem", "listItem", "site"];

const SEARCH: &str = "Search across Microsoft 365 content (messages, files, sites, list items).";
const SEARCH_MESSAGES: &str = "Search for email messages across Microsoft 365.";
const BATCH: &str = "Run up to 20 Graph requests in one call; much faster than calling tools \
    one by one for read-heavy work. Each request has `id` (correlates the reply), `method` \
    (GET, POST, PATCH, DELETE or PUT), `url` (a Graph path relative to the version root, e.g. \
    \"/me/messages?$top=5\"), and optionally `body` (JSON, for POST and PATCH) and `headers`. \
    Answers each response with its id, status and body, sorted by id.";

graph_tools! {
    "d365_search" => search, SearchInput, GraphItem, SEARCH;
    "d365_search_messages" => search_messages, SearchMessagesInput, GraphItem, SEARCH_MESSAGES;
    "d365_batch" => batch, BatchInput, GraphItem, BATCH;
}

const fn default_search_top() -> i64 {
    10
}

const fn default_messages_top() -> i64 {
    25
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SearchInput {
    #[schemars(description = "The search query string.")]
    pub query: String,
    #[schemars(
        description = "Entity types to search. Empty searches message, driveItem, listItem and site."
    )]
    #[serde(default)]
    pub entity_types: Vec<String>,
    #[schemars(description = "Maximum number of results to return (1-50, default 10).")]
    #[serde(default = "default_search_top")]
    pub top: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SearchMessagesInput {
    #[schemars(description = "The search query string.")]
    pub query: String,
    #[schemars(description = "Maximum number of results to return (1-50, default 25).")]
    #[serde(default = "default_messages_top")]
    pub top: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct BatchInput {
    #[schemars(
        description = "The requests, each an object with id, method, url and optionally body and headers."
    )]
    pub requests: Vec<Map<String, Value>>,
}

async fn query_hits(
    graph: &GraphClient,
    entity_types: Vec<String>,
    query: &str,
    top: i64,
) -> Result<GraphItem, GraphError> {
    let body = json!({"requests": [{
        "entityTypes": entity_types,
        "query": {"queryString": sanitize_search(query)},
        "from": 0,
        "size": clamp_top(top, DEFAULT_MAX_TOP),
    }]});
    let result = graph.post_json("/search/query", body).await?;
    let hits = result
        .pointer("/value/0/hitsContainers/0/hits")
        .filter(|hits| hits.is_array())
        .cloned()
        .unwrap_or_else(|| json!([]));
    Ok(GraphItem::new(hits))
}

pub async fn search(graph: &GraphClient, input: SearchInput) -> Result<GraphItem, GraphError> {
    let types = if input.entity_types.is_empty() {
        DEFAULT_ENTITY_TYPES.map(str::to_owned).to_vec()
    } else {
        input.entity_types
    };
    query_hits(graph, types, &input.query, input.top).await
}

pub async fn search_messages(
    graph: &GraphClient,
    input: SearchMessagesInput,
) -> Result<GraphItem, GraphError> {
    query_hits(graph, vec!["message".to_owned()], &input.query, input.top).await
}

fn text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

// Why: Python truthiness chose the Belgian default id: null, "", 0 and false
// all fall back to `req<index>`.
fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().is_some_and(|n| n.abs() > 0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(fields) => !fields.is_empty(),
    }
}

// Why: Graph's `$batch` needs a `Content-Type` on every request carrying a
// body; JSON unless the caller's headers name another.
fn batch_item(index: usize, request: &Map<String, Value>) -> Result<Value, String> {
    let id = request
        .get("id")
        .filter(|id| truthy(id))
        .map_or_else(|| format!("req{index}"), text);
    let method = request
        .get("method")
        .map_or_else(|| "GET".to_owned(), text)
        .to_uppercase();
    let url = match request.get("url") {
        Some(Value::String(url)) if !url.is_empty() => url.clone(),
        _ => return Err(format!("Request {id} has no 'url'")),
    };
    if !BATCH_METHODS.contains(&method.as_str()) {
        return Err(format!("Request {id} method '{method}' not allowed"));
    }
    let mut item = Map::new();
    item.insert("id".to_owned(), Value::String(id));
    item.insert("method".to_owned(), Value::String(method));
    item.insert("url".to_owned(), Value::String(url));
    let headers = request.get("headers").cloned();
    match request.get("body").filter(|body| !body.is_null()) {
        Some(body) => {
            let mut headers = headers
                .and_then(|headers| headers.as_object().cloned())
                .unwrap_or_default();
            headers
                .entry("Content-Type")
                .or_insert_with(|| json!("application/json"));
            item.insert("body".to_owned(), body.clone());
            item.insert("headers".to_owned(), Value::Object(headers));
        },
        None => {
            if let Some(headers) = headers {
                item.insert("headers".to_owned(), headers);
            }
        },
    }
    Ok(Value::Object(item))
}

pub async fn batch(graph: &GraphClient, input: BatchInput) -> Result<GraphItem, GraphError> {
    let count = input.requests.len();
    if count == 0 {
        return Ok(noted(
            "Provide a non-empty list of request objects".to_owned(),
        ));
    }
    if count > MAX_BATCH_REQUESTS {
        return Ok(noted(format!(
            "Too many requests: {count}. Graph batch max is {MAX_BATCH_REQUESTS}."
        )));
    }
    let mut items = Vec::with_capacity(count);
    for (index, request) in input.requests.iter().enumerate() {
        match batch_item(index, request) {
            Ok(item) => items.push(item),
            Err(refusal) => return Ok(noted(refusal)),
        }
    }
    let data = graph
        .post_json("/$batch", json!({"requests": items}))
        .await?;
    let mut responses = match data.get("responses") {
        Some(Value::Array(responses)) => responses.clone(),
        _ => Vec::new(),
    };
    let id = |response: &Value| {
        response
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    responses.sort_by_key(id);
    Ok(GraphItem::new(Value::Array(responses)))
}
