//! Drive inventory input and project-search defaults.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub(super) const fn include_shared_default() -> bool {
    true
}


pub const DRIVE_INVENTORY_TOOL: &str = "knowledge_drive_inventory";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DriveInventoryInput {
    pub source_type: String,
    #[serde(default)]
    pub after: String,
    #[serde(default = "inventory_limit")]
    pub limit: i64,
}
const fn inventory_limit() -> i64 {
    100
}


pub(super) fn inventory_tool() -> rmcp::model::Tool {
    super::create_tool(
        DRIVE_INVENTORY_TOOL,
        "Drive inventory",
        "List Drive files and project IDs with extraction status; paginate using the last file_id as after.",
        &serde_json::to_value(schemars::schema_for!(DriveInventoryInput)).unwrap_or_default(),
        &super::output_schema(),
    )
}
