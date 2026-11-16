//! The reply shape of a list tool, the Belgian server's
//! `paginated_response`: the items, and when more exist `has_more`, how many
//! came back and a hint telling the model to page with `skip`.

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
