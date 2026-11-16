//! OneDrive writes on drive items: create a folder, create a sharing link,
//! move or rename an item, and delete it. Paths and ids go through the same
//! checks and encoding as the reads in `drive`.

use schemars::JsonSchema;
use serde::Deserialize;
// JSON: request bodies are built as the Graph resource they write, and
// Graph's answer passes through as returned.
use serde_json::json;

use super::drive::{DriveItemInput, item_path, root_path};
use super::reply::GraphItem;
use crate::graph::sanitize::validate_id;
use crate::graph::{GraphClient, GraphError};

pub const CREATE_FOLDER_DESCRIPTION: &str = "Create a folder under a parent path relative to the \
    drive root; a name already taken gets a numbered one.";
pub const SHARE_FILE_DESCRIPTION: &str = "Create a sharing link for a drive item: type 'view' or \
    'edit'. The link works for people in the user's organization unless scope 'anonymous' is \
    asked for explicitly, which makes it public to anyone who has the link.";
pub const MOVE_FILE_DESCRIPTION: &str = "Move a drive item into another folder of the same drive \
    (by the folder's item ID), rename it, or both.";
pub const DELETE_FILE_DESCRIPTION: &str =
    "Delete a file or folder from the drive (it goes to the recycle bin).";

fn default_link_type() -> String {
    "view".to_owned()
}

// Why: Belgium's server defaulted to 'anonymous', a public link; for an end
// customer a link leaves the organization only when someone asks for it.
fn default_link_scope() -> String {
    "organization".to_owned()
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateFolderInput {
    #[schemars(description = "Parent folder path relative to the drive root (e.g. 'Documents').")]
    pub parent_path: String,
    #[schemars(description = "Name of the new folder.")]
    pub folder_name: String,
    #[schemars(description = "A drive ID; omit for the user's own OneDrive.")]
    #[serde(default)]
    pub drive_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ShareFileInput {
    #[schemars(description = "The drive item ID.")]
    pub item_id: String,
    #[schemars(description = "Link type: 'view' or 'edit' (default 'view').")]
    #[serde(rename = "type", default = "default_link_type")]
    pub link_type: String,
    #[schemars(
        description = "Who the link works for: 'organization' (default) is people signed in to \
                       the user's organization; 'anonymous' is anyone with the link, without \
                       signing in. Use 'anonymous' only when a public link is asked for."
    )]
    #[serde(default = "default_link_scope")]
    pub scope: String,
    #[schemars(description = "A drive ID; omit for the user's own OneDrive.")]
    #[serde(default)]
    pub drive_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MoveFileInput {
    #[schemars(description = "The drive item ID.")]
    pub item_id: String,
    #[schemars(description = "Item ID of the folder to move into (same drive); omit to stay.")]
    #[serde(default)]
    pub destination_folder_id: String,
    #[schemars(description = "New name for the item; omit to keep the name.")]
    #[serde(default)]
    pub new_name: String,
    #[schemars(description = "A drive ID; omit for the user's own OneDrive.")]
    #[serde(default)]
    pub drive_id: String,
}

pub async fn create_folder(
    graph: &GraphClient,
    input: CreateFolderInput,
) -> Result<GraphItem, GraphError> {
    let path = root_path(&input.drive_id, &input.parent_path, "/children")?;
    let folder = json!({
        "name": &input.folder_name,
        "folder": {},
        "@microsoft.graph.conflictBehavior": "rename",
    });
    Ok(GraphItem::new(graph.post_json(&path, folder).await?))
}

pub async fn share_file(
    graph: &GraphClient,
    input: ShareFileInput,
) -> Result<GraphItem, GraphError> {
    let path = item_path(&input.drive_id, &input.item_id, "/createLink")?;
    let link = json!({ "type": &input.link_type, "scope": &input.scope });
    Ok(GraphItem::new(graph.post_json(&path, link).await?))
}

pub async fn move_file(graph: &GraphClient, input: MoveFileInput) -> Result<GraphItem, GraphError> {
    let path = item_path(&input.drive_id, &input.item_id, "")?;
    let mut patch = json!({});
    if !input.destination_folder_id.is_empty() {
        let folder = validate_id(&input.destination_folder_id, "destination_folder_id")?;
        patch["parentReference"] = json!({ "id": folder });
    }
    if !input.new_name.is_empty() {
        patch["name"] = json!(&input.new_name);
    }
    Ok(GraphItem::new(graph.patch_json(&path, patch).await?))
}

pub async fn delete_file(
    graph: &GraphClient,
    input: DriveItemInput,
) -> Result<GraphItem, GraphError> {
    let path = item_path(&input.drive_id, &input.item_id, "")?;
    graph.delete(&path).await?;
    Ok(GraphItem::new(
        json!({ "deleted": true, "item_id": &input.item_id }),
    ))
}
