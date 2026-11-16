//! `SharePoint` tools: find a site by keyword, ID or host and path, list its
//! lists and document libraries, and read, create and update list items.
//!
//! A library's files are then reached with the `OneDrive` tools and the
//! library's `drive_id`. List item `fields` are the list's own columns,
//! passed through as an open object.

use schemars::JsonSchema;
use serde::Deserialize;
// JSON: sites, lists and items are Graph resources passed through as Graph
// returns them; list item fields are the list's own columns.
use serde_json::{Map, Value, json};

use super::drive::{default_top, encode_path};
use super::graph_tool::graph_tools;
use super::reply::{GraphItem, GraphList};
use super::{ToolKind, ToolSpec};
use crate::graph::sanitize::{
    DEFAULT_MAX_TOP, clamp_skip, clamp_top, sanitize_search, validate_hostname, validate_id,
};
use crate::graph::{GraphClient, GraphError, GraphQuery, Window};

pub const SITE_FIELDS: &str = "id,displayName,webUrl,description";
pub const LIST_FIELDS: &str = "id,displayName,description,list";

const SEARCH_SITES: &str = "Search for SharePoint sites by keyword. Page with `skip`.";
const GET_SITE: &str = "Get the details of a SharePoint site by its ID.";
const GET_SITE_BY_PATH: &str = "Get a SharePoint site by host name (e.g. \
    'contoso.sharepoint.com') and site path (e.g. '/sites/marketing').";
const LIST_LISTS: &str = "List the lists of a SharePoint site. Page with `skip`.";
const LIST_ITEMS: &str = "List the items of a SharePoint list, with their column values in \
    `fields` unless expand_fields is false. Page with `top` (1-50) and `skip`.";
const LIST_SITE_DRIVES: &str = "List the document libraries (drives) of a SharePoint site; use \
    a drive's ID as `drive_id` in the OneDrive tools. Page with `skip`.";
const CREATE_LIST_ITEM: &str =
    "Create an item in a SharePoint list from its column values (`fields`).";
const UPDATE_LIST_ITEM: &str =
    "Update the column values of a SharePoint list item; only the fields given change.";

pub const SHAREPOINT_SPECS: &[ToolSpec] = &[
    ToolSpec::new(
        "d365_search_sharepoint_sites",
        "Search SharePoint sites",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_get_sharepoint_site",
        "Get SharePoint site",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_get_sharepoint_site_by_path",
        "Get SharePoint site by path",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_sharepoint_lists",
        "List SharePoint lists",
        ToolKind::Read,
    ),
    ToolSpec::new(
        "d365_list_sharepoint_list_items",
        "List SharePoint list items",
        ToolKind::Read,
    ),
    ToolSpec::new("d365_list_site_drives", "List site drives", ToolKind::Read),
    ToolSpec::new(
        "d365_create_sharepoint_list_item",
        "Create SharePoint list item",
        ToolKind::Write,
    ),
    ToolSpec::new(
        "d365_update_sharepoint_list_item",
        "Update SharePoint list item",
        ToolKind::IdempotentWrite,
    ),
];

graph_tools! {
    "d365_search_sharepoint_sites" => search_sharepoint_sites, SearchSitesInput, GraphList, SEARCH_SITES;
    "d365_get_sharepoint_site" => get_sharepoint_site, SiteIdInput, GraphItem, GET_SITE;
    "d365_get_sharepoint_site_by_path" => get_sharepoint_site_by_path, SiteByPathInput, GraphItem, GET_SITE_BY_PATH;
    "d365_list_sharepoint_lists" => list_sharepoint_lists, SitePageInput, GraphList, LIST_LISTS;
    "d365_list_sharepoint_list_items" => list_sharepoint_list_items, ListItemsInput, GraphList, LIST_ITEMS;
    "d365_list_site_drives" => list_site_drives, SitePageInput, GraphList, LIST_SITE_DRIVES;
    "d365_create_sharepoint_list_item" => create_sharepoint_list_item, CreateListItemInput, GraphItem, CREATE_LIST_ITEM;
    "d365_update_sharepoint_list_item" => update_sharepoint_list_item, UpdateListItemInput, GraphItem, UPDATE_LIST_ITEM;
}

const fn default_expand() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SearchSitesInput {
    #[schemars(description = "Search keywords.")]
    pub query: String,
    #[schemars(description = "Items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SiteIdInput {
    #[schemars(description = "The site ID.")]
    pub site_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SiteByPathInput {
    #[schemars(description = "The SharePoint host (e.g. 'contoso.sharepoint.com').")]
    pub hostname: String,
    #[schemars(description = "The site path (e.g. '/sites/marketing'); '/' for the root site.")]
    pub site_path: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SitePageInput {
    #[schemars(description = "The site ID.")]
    pub site_id: String,
    #[schemars(description = "Items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListItemsInput {
    #[schemars(description = "The site ID.")]
    pub site_id: String,
    #[schemars(description = "The list ID.")]
    pub list_id: String,
    #[schemars(description = "Number of items to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Whether to include each item's column values (default true).")]
    #[serde(default = "default_expand")]
    pub expand_fields: bool,
    #[schemars(description = "Items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateListItemInput {
    #[schemars(description = "The site ID.")]
    pub site_id: String,
    #[schemars(description = "The list ID.")]
    pub list_id: String,
    #[schemars(description = "Column values of the new item, by column name.")]
    pub fields: Map<String, Value>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateListItemInput {
    #[schemars(description = "The site ID.")]
    pub site_id: String,
    #[schemars(description = "The list ID.")]
    pub list_id: String,
    #[schemars(description = "The list item ID.")]
    pub item_id: String,
    #[schemars(description = "Column values to change, by column name.")]
    pub fields: Map<String, Value>,
}

fn site_path(site_id: &str) -> Result<String, GraphError> {
    Ok(format!("/sites/{}", validate_id(site_id, "site_id")?))
}

fn items_path(site_id: &str, list_id: &str) -> Result<String, GraphError> {
    let list = validate_id(list_id, "list_id")?;
    Ok(format!("{}/lists/{list}/items", site_path(site_id)?))
}

fn skip_only(skip: i64) -> Window {
    Window {
        skip: clamp_skip(skip),
        top: None,
    }
}

async fn list(
    graph: &GraphClient,
    path: &str,
    query: GraphQuery,
    window: Window,
) -> Result<GraphList, GraphError> {
    Ok(GraphList::from(
        graph.get_collection(path, &query, window).await?,
    ))
}

pub async fn search_sharepoint_sites(
    graph: &GraphClient,
    input: SearchSitesInput,
) -> Result<GraphList, GraphError> {
    let query = GraphQuery::new()
        .with("search", sanitize_search(&input.query))
        .with("$select", SITE_FIELDS);
    list(graph, "/sites", query, skip_only(input.skip)).await
}

pub async fn get_sharepoint_site(
    graph: &GraphClient,
    input: SiteIdInput,
) -> Result<GraphItem, GraphError> {
    let path = site_path(&input.site_id)?;
    Ok(GraphItem::new(
        graph.get_json(&path, &GraphQuery::new()).await?,
    ))
}

pub async fn get_sharepoint_site_by_path(
    graph: &GraphClient,
    input: SiteByPathInput,
) -> Result<GraphItem, GraphError> {
    let host = validate_hostname(&input.hostname)?;
    let relative = encode_path(&input.site_path)?;
    let path = if relative.is_empty() {
        format!("/sites/{host}")
    } else {
        format!("/sites/{host}:/{relative}")
    };
    Ok(GraphItem::new(
        graph.get_json(&path, &GraphQuery::new()).await?,
    ))
}

pub async fn list_sharepoint_lists(
    graph: &GraphClient,
    input: SitePageInput,
) -> Result<GraphList, GraphError> {
    let path = format!("{}/lists", site_path(&input.site_id)?);
    let query = GraphQuery::new().with("$select", LIST_FIELDS);
    list(graph, &path, query, skip_only(input.skip)).await
}

pub async fn list_sharepoint_list_items(
    graph: &GraphClient,
    input: ListItemsInput,
) -> Result<GraphList, GraphError> {
    let path = items_path(&input.site_id, &input.list_id)?;
    let mut query = GraphQuery::new();
    if input.expand_fields {
        query.set("$expand", "fields");
    }
    let window = Window {
        skip: clamp_skip(input.skip),
        top: Some(clamp_top(input.top, DEFAULT_MAX_TOP)),
    };
    list(graph, &path, query, window).await
}

pub async fn list_site_drives(
    graph: &GraphClient,
    input: SitePageInput,
) -> Result<GraphList, GraphError> {
    let path = format!("{}/drives", site_path(&input.site_id)?);
    list(graph, &path, GraphQuery::new(), skip_only(input.skip)).await
}

pub async fn create_sharepoint_list_item(
    graph: &GraphClient,
    input: CreateListItemInput,
) -> Result<GraphItem, GraphError> {
    let path = items_path(&input.site_id, &input.list_id)?;
    let item = json!({ "fields": input.fields });
    Ok(GraphItem::new(graph.post_json(&path, item).await?))
}

pub async fn update_sharepoint_list_item(
    graph: &GraphClient,
    input: UpdateListItemInput,
) -> Result<GraphItem, GraphError> {
    let item = validate_id(&input.item_id, "item_id")?;
    let path = format!(
        "{}/{item}/fields",
        items_path(&input.site_id, &input.list_id)?
    );
    let fields = Value::Object(input.fields);
    Ok(GraphItem::new(graph.patch_json(&path, fields).await?))
}
