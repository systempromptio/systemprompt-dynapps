//! `OneDrive` reads on drive items: the user's drives, a folder's children,
//! search within a drive, and an item's metadata and sharing permissions (the
//! writes are in `drive_write`).
//!
//! Each tool works on the user's own drive (`/me/drive`) unless it is given a
//! `drive_id` (a `SharePoint` library or a drive shared with the user).
//!
//! A location relative to the drive root is addressed as `root:/a/b:` (the
//! root itself as `root`), with empty segments dropped, `..` refused and
//! every segment percent-encoded, so a name with a space, `#` or `%` reaches
//! Graph whole; the Belgian server put the raw path in the URL.

use schemars::JsonSchema;
use serde::Deserialize;

use super::reply::{GraphItem, GraphList};
use crate::graph::sanitize::{
    DEFAULT_MAX_TOP, clamp_skip, clamp_top, sanitize_odata_string, validate_id, validate_path,
};
use crate::graph::{GraphClient, GraphError, GraphQuery, Window};

pub const DRIVE_ITEM_FIELDS: &str =
    "id,name,size,createdDateTime,lastModifiedDateTime,webUrl,file,folder,parentReference";

pub const LIST_DRIVES_DESCRIPTION: &str = "List all drives (OneDrive, SharePoint document \
    libraries) the signed-in user can reach. Page with `skip`.";
pub const LIST_FILES_DESCRIPTION: &str = "List the files and folders in a OneDrive folder, by \
    path relative to the drive root ('/' for the root). Page with `top` (1-50) and `skip`.";
pub const SEARCH_FILES_DESCRIPTION: &str =
    "Search for files by name or content within one drive. Page with `top` (1-50) and `skip`.";
pub const GET_FILE_METADATA_DESCRIPTION: &str =
    "Get the metadata (size, dates, parent, etc.) of one drive item.";
pub const LIST_FILE_PERMISSIONS_DESCRIPTION: &str =
    "List the sharing permissions on a drive item. Page with `skip`.";
pub(super) const fn default_top() -> i64 {
    25
}

fn default_root() -> String {
    "/".to_owned()
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct ListDrivesInput {
    #[schemars(description = "Items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListFilesInput {
    #[schemars(description = "Folder path relative to the drive root (default '/').")]
    #[serde(default = "default_root")]
    pub folder_path: String,
    #[schemars(description = "A drive ID; omit for the user's own OneDrive.")]
    #[serde(default)]
    pub drive_id: String,
    #[schemars(description = "Number of items to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SearchFilesInput {
    #[schemars(description = "Search text, matched against file names and content.")]
    pub query: String,
    #[schemars(description = "A drive ID; omit for the user's own OneDrive.")]
    #[serde(default)]
    pub drive_id: String,
    #[schemars(description = "Number of results to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DriveItemInput {
    #[schemars(description = "The drive item ID.")]
    pub item_id: String,
    #[schemars(description = "A drive ID; omit for the user's own OneDrive.")]
    #[serde(default)]
    pub drive_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListPermissionsInput {
    #[schemars(description = "The drive item ID.")]
    pub item_id: String,
    #[schemars(description = "A drive ID; omit for the user's own OneDrive.")]
    #[serde(default)]
    pub drive_id: String,
    #[schemars(description = "Items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

fn drive_base(drive_id: &str) -> Result<String, GraphError> {
    if drive_id.is_empty() {
        return Ok("/me/drive".to_owned());
    }
    Ok(format!("/drives/{}", validate_id(drive_id, "drive_id")?))
}

pub(super) fn item_path(drive_id: &str, item_id: &str, suffix: &str) -> Result<String, GraphError> {
    let item = validate_id(item_id, "item_id")?;
    Ok(format!("{}/items/{item}{suffix}", drive_base(drive_id)?))
}

pub(super) fn encode_path(relative: &str) -> Result<String, GraphError> {
    let segments: Vec<String> = validate_path(relative)?
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| urlencoding::encode(segment).into_owned())
        .collect();
    Ok(segments.join("/"))
}

pub(super) fn root_path(
    drive_id: &str,
    relative: &str,
    suffix: &str,
) -> Result<String, GraphError> {
    let base = drive_base(drive_id)?;
    let encoded = encode_path(relative)?;
    if encoded.is_empty() {
        return Ok(format!("{base}/root{suffix}"));
    }
    Ok(format!("{base}/root:/{encoded}:{suffix}"))
}

fn page_window(top: i64, skip: i64) -> Window {
    Window {
        skip: clamp_skip(skip),
        top: Some(clamp_top(top, DEFAULT_MAX_TOP)),
    }
}

fn item_fields() -> GraphQuery {
    GraphQuery::new().with("$select", DRIVE_ITEM_FIELDS)
}

pub async fn list_drives(
    graph: &GraphClient,
    input: ListDrivesInput,
) -> Result<GraphList, GraphError> {
    let window = Window {
        skip: clamp_skip(input.skip),
        top: None,
    };
    let page = graph
        .get_collection("/me/drives", &GraphQuery::new(), window)
        .await?;
    Ok(GraphList::from(page))
}

pub async fn list_files(
    graph: &GraphClient,
    input: ListFilesInput,
) -> Result<GraphList, GraphError> {
    let path = root_path(&input.drive_id, &input.folder_path, "/children")?;
    let window = page_window(input.top, input.skip);
    let page = graph.get_collection(&path, &item_fields(), window).await?;
    Ok(GraphList::from(page))
}

pub async fn search_files(
    graph: &GraphClient,
    input: SearchFilesInput,
) -> Result<GraphList, GraphError> {
    let query = urlencoding::encode(&sanitize_odata_string(&input.query)).into_owned();
    let path = root_path(&input.drive_id, "/", &format!("/search(q='{query}')"))?;
    let window = page_window(input.top, input.skip);
    let page = graph.get_collection(&path, &item_fields(), window).await?;
    Ok(GraphList::from(page))
}

pub async fn get_file_metadata(
    graph: &GraphClient,
    input: DriveItemInput,
) -> Result<GraphItem, GraphError> {
    let path = item_path(&input.drive_id, &input.item_id, "")?;
    let data = graph.get_json(&path, &GraphQuery::new()).await?;
    Ok(GraphItem::new(data))
}

pub async fn list_file_permissions(
    graph: &GraphClient,
    input: ListPermissionsInput,
) -> Result<GraphList, GraphError> {
    let path = item_path(&input.drive_id, &input.item_id, "/permissions")?;
    let window = Window {
        skip: clamp_skip(input.skip),
        top: None,
    };
    let page = graph
        .get_collection(&path, &GraphQuery::new(), window)
        .await?;
    Ok(GraphList::from(page))
}
