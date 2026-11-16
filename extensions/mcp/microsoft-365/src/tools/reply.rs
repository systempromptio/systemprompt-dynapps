//! The reply shapes of the tools. A list tool answers like the Belgian
//! server's `paginated_response`: the items, and when more exist `has_more`,
//! how many came back and a hint telling the model to page with `skip`. A
//! single-item tool answers like its `success_response`: `data` and
//! `message`.

use schemars::JsonSchema;
use serde::Serialize;
use systemprompt::mcp::McpOutputSchema;

use crate::graph::Page;

pub const MORE_HINT: &str =
    "More results available. Call this tool again with a higher `skip` value to get the next page.";

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct GraphList {
    // JSON: Graph resources pass through as Graph returns them; their schema is Graph's.
    pub data: Vec<serde_json::Value>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_returned: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl From<Page> for GraphList {
    fn from(page: Page) -> Self {
        let more = page.has_more;
        let count = page.items.len();
        Self {
            data: page.items,
            message: "success".to_owned(),
            has_more: more.then_some(true),
            total_returned: more.then_some(count),
            hint: more.then(|| MORE_HINT.to_owned()),
        }
    }
}

impl GraphList {
    pub fn summary(&self) -> String {
        let count = self.data.len();
        if self.has_more == Some(true) {
            format!("{count} items returned; more available")
        } else {
            format!("{count} items returned")
        }
    }
}

impl McpOutputSchema for GraphList {
    fn artifact_type() -> &'static str {
        "microsoft_365_list"
    }
}

/// The reply of a single-item tool, the Belgian server's
/// `success_response`: the Graph resource (or the tool's own summary of a
/// write Graph answers with 204) under `data`.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct GraphItem {
    // JSON: a Graph resource passes through as Graph returns it; its schema is Graph's.
    pub data: serde_json::Value,
    pub message: String,
}

impl GraphItem {
    pub fn new(data: serde_json::Value) -> Self {
        Self {
            data,
            message: "success".to_owned(),
        }
    }

    pub fn summary(&self) -> String {
        match self.data.get("id").and_then(serde_json::Value::as_str) {
            Some(_) => "1 item returned".to_owned(),
            None => "done".to_owned(),
        }
    }
}

impl McpOutputSchema for GraphItem {
    fn artifact_type() -> &'static str {
        "microsoft_365_item"
    }
}
