//! OneNote: the signed-in user's notebooks, sections and pages. A page's
//! content is the HTML Graph returns, handed back as text like the Belgian
//! server did; a new page is sent as an HTML document whose title is escaped.

use reqwest::Method;
use schemars::JsonSchema;
use serde::Deserialize;
// JSON: notebooks, sections and pages are Graph resources passed through.
use serde_json::json;

use super::graph_tool::graph_tools;
use super::reply::{GraphItem, GraphList};
use super::teams::{list_page, select};
use crate::graph::sanitize::validate_id;
use crate::graph::{GraphBody, GraphClient, GraphError, GraphQuery, GraphRequest};

pub const NOTEBOOK_FIELDS: &str = "id,displayName,createdDateTime,lastModifiedDateTime";
pub const SECTION_FIELDS: &str = "id,displayName,createdDateTime,lastModifiedDateTime";

const LIST_NOTEBOOKS: &str = "List all OneNote notebooks of the signed-in user. Page with `skip`.";
const LIST_NOTEBOOK_SECTIONS: &str =
    "List all sections within a specific notebook. Page with `skip`.";
const LIST_SECTIONS: &str =
    "List all OneNote sections across all notebooks of the signed-in user. Page with `skip`.";
const GET_PAGE_CONTENT: &str = "Get the HTML content of a OneNote page.";
const LIST_SECTION_PAGES: &str =
    "List pages within a OneNote section. Page with `top` (1-50) and `skip`.";
const CREATE_NOTEBOOK: &str = "Create a OneNote notebook.";
const CREATE_SECTION: &str = "Create a section in a OneNote notebook.";
const CREATE_PAGE: &str = "Create a page in a OneNote section from a title and an HTML body.";

graph_tools! {
    "d365_list_notebooks" => list_notebooks, NotebooksInput, GraphList, LIST_NOTEBOOKS;
    "d365_list_notebook_sections" => list_notebook_sections, NotebookSectionsInput, GraphList,
        LIST_NOTEBOOK_SECTIONS;
    "d365_list_sections" => list_sections, NotebooksInput, GraphList, LIST_SECTIONS;
    "d365_get_page_content" => get_page_content, PageIdInput, GraphItem, GET_PAGE_CONTENT;
    "d365_list_section_pages" => list_section_pages, SectionPagesInput, GraphList,
        LIST_SECTION_PAGES;
    "d365_create_notebook" => create_notebook, CreateNotebookInput, GraphItem, CREATE_NOTEBOOK;
    "d365_create_section" => create_section, CreateSectionInput, GraphItem, CREATE_SECTION;
    "d365_create_page" => create_page, CreatePageInput, GraphItem, CREATE_PAGE;
}

const fn default_top() -> i64 {
    25
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct NotebooksInput {
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct NotebookSectionsInput {
    #[schemars(description = "The notebook ID.")]
    pub notebook_id: String,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct PageIdInput {
    #[schemars(description = "The page ID.")]
    pub page_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SectionPagesInput {
    #[schemars(description = "The section ID.")]
    pub section_id: String,
    #[schemars(description = "Maximum number of pages to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateNotebookInput {
    #[schemars(description = "Display name for the new notebook.")]
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateSectionInput {
    #[schemars(description = "The notebook ID.")]
    pub notebook_id: String,
    #[schemars(description = "Display name for the new section.")]
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreatePageInput {
    #[schemars(description = "The section ID to create the page in.")]
    pub section_id: String,
    #[schemars(description = "Page title.")]
    pub title: String,
    #[schemars(description = "HTML body content for the page.")]
    pub html_content: String,
}

// Why: Python's `html.escape` (quote=True) is what the Belgian server applied
// to the page title, so the same five characters are escaped here.
pub fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            other => escaped.push(other),
        }
    }
    escaped
}

fn notebook_path(notebook_id: &str) -> Result<String, GraphError> {
    let notebook = validate_id(notebook_id, "notebook_id")?;
    Ok(format!("/me/onenote/notebooks/{notebook}"))
}

fn section_path(section_id: &str) -> Result<String, GraphError> {
    let section = validate_id(section_id, "section_id")?;
    Ok(format!("/me/onenote/sections/{section}"))
}

pub async fn list_notebooks(
    graph: &GraphClient,
    input: NotebooksInput,
) -> Result<GraphList, GraphError> {
    let query = select(NOTEBOOK_FIELDS);
    list_page(graph, "/me/onenote/notebooks", query, input.skip, None).await
}

pub async fn list_notebook_sections(
    graph: &GraphClient,
    input: NotebookSectionsInput,
) -> Result<GraphList, GraphError> {
    let path = format!("{}/sections", notebook_path(&input.notebook_id)?);
    list_page(graph, &path, select(SECTION_FIELDS), input.skip, None).await
}

pub async fn list_sections(
    graph: &GraphClient,
    input: NotebooksInput,
) -> Result<GraphList, GraphError> {
    let query = select(SECTION_FIELDS);
    list_page(graph, "/me/onenote/sections", query, input.skip, None).await
}

pub async fn get_page_content(
    graph: &GraphClient,
    input: PageIdInput,
) -> Result<GraphItem, GraphError> {
    let page = validate_id(&input.page_id, "page_id")?;
    let path = format!("/me/onenote/pages/{page}/content");
    let response = graph.send(&GraphRequest::new(Method::GET, path)).await?;
    let html = String::from_utf8_lossy(&response.body);
    Ok(GraphItem::new(json!({"html": html, "page_id": page})))
}

pub async fn list_section_pages(
    graph: &GraphClient,
    input: SectionPagesInput,
) -> Result<GraphList, GraphError> {
    let path = format!("{}/pages", section_path(&input.section_id)?);
    list_page(graph, &path, GraphQuery::new(), input.skip, Some(input.top)).await
}

pub async fn create_notebook(
    graph: &GraphClient,
    input: CreateNotebookInput,
) -> Result<GraphItem, GraphError> {
    let body = json!({"displayName": input.display_name});
    let data = graph.post_json("/me/onenote/notebooks", body).await?;
    Ok(GraphItem::new(data))
}

pub async fn create_section(
    graph: &GraphClient,
    input: CreateSectionInput,
) -> Result<GraphItem, GraphError> {
    let path = format!("{}/sections", notebook_path(&input.notebook_id)?);
    let body = json!({"displayName": input.display_name});
    let data = graph.post_json(&path, body).await?;
    Ok(GraphItem::new(data))
}

pub async fn create_page(
    graph: &GraphClient,
    input: CreatePageInput,
) -> Result<GraphItem, GraphError> {
    let path = format!("{}/pages", section_path(&input.section_id)?);
    let title = escape_html(&input.title);
    let html = format!(
        "<html><head><title>{title}</title></head><body>{}</body></html>",
        input.html_content
    );
    let body = GraphBody::Bytes {
        content_type: "text/html".to_owned(),
        data: html.into_bytes(),
    };
    let request = GraphRequest::new(Method::POST, path).with_body(body);
    let data = graph.send(&request).await?.json()?;
    Ok(GraphItem::new(data))
}
