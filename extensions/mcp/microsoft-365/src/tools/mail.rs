//! Mail tools. `d365_list_mail` lists a folder with the Belgian server's
//! filters, ordering and paging. Graph takes neither `$filter` nor
//! `$orderby` beside `$search` on messages (Graph's query-parameter
//! rules), so a search drops both; the Belgian server still sent `$orderby`.

use schemars::JsonSchema;
use serde::Deserialize;

use crate::graph::sanitize::{
    DEFAULT_MAX_TOP, clamp_skip, clamp_top, sanitize_odata_datetime, sanitize_odata_string,
    sanitize_search, validate_folder,
};
use crate::graph::{GraphClient, GraphError, GraphQuery, Page, Window};

pub const MAIL_LIST_FIELDS: &str =
    "id,subject,from,toRecipients,receivedDateTime,bodyPreview,isRead,hasAttachments,importance";

pub const LIST_MAIL_DESCRIPTION: &str = "List emails from a mail folder, newest first. Filters \
    (is_read, from_address, subject_contains, has_attachments, importance, received_after, \
    received_before) combine with AND; `search` is a KQL keyword search over subject, body and \
    sender and is ignored when any filter is set. Page with `top` (1-50) and `skip`.";

const fn default_top() -> i64 {
    25
}

fn default_folder() -> String {
    "inbox".to_owned()
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListMailInput {
    #[schemars(
        description = "Mail folder: \"inbox\", \"sentitems\", \"drafts\", \"deleteditems\", \"junkemail\", or a folder ID."
    )]
    #[serde(default = "default_folder")]
    pub folder: String,
    #[schemars(description = "Number of messages to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(
        description = "Filter by read status (true = read, false = unread, omitted = all)."
    )]
    #[serde(default)]
    #[schemars(with = "bool")]
    pub is_read: Option<bool>,
    #[schemars(description = "Filter by sender email address (exact match).")]
    #[serde(default)]
    pub from_address: String,
    #[schemars(description = "Filter by subject containing this text.")]
    #[serde(default)]
    pub subject_contains: String,
    #[schemars(description = "Filter by attachment presence (true/false, omitted = all).")]
    #[serde(default)]
    #[schemars(with = "bool")]
    pub has_attachments: Option<bool>,
    #[schemars(description = "Filter by importance: \"low\", \"normal\" or \"high\".")]
    #[serde(default)]
    pub importance: String,
    #[schemars(
        description = "Received on or after this ISO 8601 datetime (e.g. \"2024-01-01T00:00:00Z\")."
    )]
    #[serde(default)]
    pub received_after: String,
    #[schemars(
        description = "Received on or before this ISO 8601 datetime (e.g. \"2024-12-31T23:59:59Z\")."
    )]
    #[serde(default)]
    pub received_before: String,
    #[schemars(
        description = "KQL keyword search over subject, body and sender; not combined with filters."
    )]
    #[serde(default)]
    pub search: String,
    #[schemars(
        description = "Items to skip for paging (default 0); increase by `top` for the next page."
    )]
    #[serde(default)]
    pub skip: i64,
}

pub fn mail_filters(input: &ListMailInput) -> Result<Vec<String>, GraphError> {
    let mut filters = Vec::new();
    if let Some(read) = input.is_read {
        filters.push(format!("isRead eq {read}"));
    }
    if !input.from_address.is_empty() {
        let address = sanitize_odata_string(&input.from_address);
        filters.push(format!("from/emailAddress/address eq '{address}'"));
    }
    if !input.subject_contains.is_empty() {
        let subject = sanitize_odata_string(&input.subject_contains);
        filters.push(format!("contains(subject, '{subject}')"));
    }
    if let Some(attachments) = input.has_attachments {
        filters.push(format!("hasAttachments eq {attachments}"));
    }
    if !input.importance.is_empty() {
        let importance = sanitize_odata_string(&input.importance);
        filters.push(format!("importance eq '{importance}'"));
    }
    if !input.received_after.is_empty() {
        let after = sanitize_odata_datetime(&input.received_after)?;
        filters.push(format!("receivedDateTime ge {after}"));
    }
    if !input.received_before.is_empty() {
        let before = sanitize_odata_datetime(&input.received_before)?;
        filters.push(format!("receivedDateTime le {before}"));
    }
    Ok(filters)
}

pub async fn list_mail(graph: &GraphClient, input: &ListMailInput) -> Result<Page, GraphError> {
    let folder = validate_folder(&input.folder)?;
    let filters = mail_filters(input)?;
    let mut query = GraphQuery::new().with("$select", MAIL_LIST_FIELDS);
    if filters.is_empty() && !input.search.is_empty() {
        query.set("$search", format!("\"{}\"", sanitize_search(&input.search)));
    } else {
        query.set("$orderby", "receivedDateTime desc");
        if !filters.is_empty() {
            query.set("$filter", filters.join(" and "));
        }
    }
    let window = Window {
        skip: clamp_skip(input.skip),
        top: Some(clamp_top(input.top, DEFAULT_MAX_TOP)),
    };
    graph
        .get_collection(
            &format!("/me/mailFolders/{folder}/messages"),
            &query,
            window,
        )
        .await
}
