//! Mail reads beside `d365_list_mail`: one message, a KQL search over every
//! folder, the folder list, a message's attachment list, the inbox rules and
//! another mailbox the user may read (shared or delegated). Each takes the
//! Belgian server's parameters, defaults and `$select`.

use schemars::JsonSchema;
use serde::Deserialize;

use super::mail::MAIL_LIST_FIELDS;
use super::reply::{GraphItem, GraphList};
use crate::graph::sanitize::{
    DEFAULT_MAX_TOP, clamp_skip, clamp_top, sanitize_search, validate_folder, validate_id,
};
use crate::graph::{GraphClient, GraphError, GraphId, GraphQuery, Window};

pub const MAIL_DETAIL_FIELDS: &str = "id,subject,from,toRecipients,ccRecipients,bccRecipients,\
    receivedDateTime,body,isRead,hasAttachments,importance,flag,categories";
pub const MAIL_FOLDER_FIELDS: &str =
    "id,displayName,parentFolderId,childFolderCount,unreadItemCount,totalItemCount";
pub const ATTACHMENT_LIST_FIELDS: &str = "id,name,contentType,size,isInline";
const FOLDER_PAGE: u32 = 100;

pub const GET_MAIL_DESCRIPTION: &str = "Get full details of a single email message: \
    recipients, body, flag and categories.";
pub const SEARCH_MAIL_DESCRIPTION: &str = "Search all mail folders with KQL (Keyword Query \
    Language): \"subject:budget\", \"from:john@example.com\", \"hasattachment:true\", \
    \"received>=2024-01-01\"; plain keywords search subject, body and sender. Page with `top` \
    (1-50) and `skip`.";
pub const LIST_MAIL_FOLDERS_DESCRIPTION: &str = "List the user's top-level mail folders with \
    display name, unread count, total item count and folder ID.";
pub const LIST_MAIL_ATTACHMENTS_DESCRIPTION: &str = "List the attachments of an email message \
    (name, size, content type). Use d365_get_mail_attachment to fetch one as a file.";
pub const LIST_MAIL_RULES_DESCRIPTION: &str = "List the inbox message rules, with the \
    conditions and actions of each.";
pub const LIST_SHARED_MAILBOX_DESCRIPTION: &str = "List messages from a shared mailbox or \
    another user's mailbox, newest first. Requires delegate or shared mailbox permission.";

const fn default_top() -> i64 {
    25
}

fn default_folder() -> String {
    "inbox".to_owned()
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MessageIdInput {
    #[schemars(description = "The unique ID of the message.")]
    pub message_id: GraphId,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SearchMailInput {
    #[schemars(description = "KQL search query string.")]
    pub query: String,
    #[schemars(description = "Number of results to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Number of items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct SkipInput {
    #[schemars(description = "Number of items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListAttachmentsInput {
    #[schemars(description = "The unique ID of the message.")]
    pub message_id: GraphId,
    #[schemars(description = "Number of items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SharedMailboxInput {
    #[schemars(
        description = "User ID or email address of the shared mailbox (e.g. \"shared@company.com\")."
    )]
    pub user_id: GraphId,
    #[schemars(description = "Mail folder to list (default \"inbox\").")]
    #[serde(default = "default_folder")]
    pub folder: String,
    #[schemars(description = "Number of messages to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Number of items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

pub async fn get_mail(graph: &GraphClient, input: MessageIdInput) -> Result<GraphItem, GraphError> {
    let id = validate_id(&input.message_id, "message_id")?;
    let query = GraphQuery::new().with("$select", MAIL_DETAIL_FIELDS);
    graph
        .get_json(&format!("/me/messages/{id}"), &query)
        .await
        .map(GraphItem::new)
}

pub async fn search_mail(
    graph: &GraphClient,
    input: SearchMailInput,
) -> Result<GraphList, GraphError> {
    let query = GraphQuery::new()
        .with("$search", format!("\"{}\"", sanitize_search(&input.query)))
        .with("$select", MAIL_LIST_FIELDS);
    let window = Window {
        skip: clamp_skip(input.skip),
        top: Some(clamp_top(input.top, DEFAULT_MAX_TOP)),
    };
    graph
        .get_collection("/me/messages", &query, window)
        .await
        .map(GraphList::from)
}

pub async fn list_mail_folders(
    graph: &GraphClient,
    input: SkipInput,
) -> Result<GraphList, GraphError> {
    let query = GraphQuery::new().with("$select", MAIL_FOLDER_FIELDS);
    let window = Window {
        skip: clamp_skip(input.skip),
        top: Some(FOLDER_PAGE),
    };
    graph
        .get_collection("/me/mailFolders", &query, window)
        .await
        .map(GraphList::from)
}

pub async fn list_mail_attachments(
    graph: &GraphClient,
    input: ListAttachmentsInput,
) -> Result<GraphList, GraphError> {
    let id = validate_id(&input.message_id, "message_id")?;
    let query = GraphQuery::new().with("$select", ATTACHMENT_LIST_FIELDS);
    let window = Window {
        skip: clamp_skip(input.skip),
        top: None,
    };
    graph
        .get_collection(&format!("/me/messages/{id}/attachments"), &query, window)
        .await
        .map(GraphList::from)
}

pub async fn list_mail_rules(
    graph: &GraphClient,
    input: SkipInput,
) -> Result<GraphList, GraphError> {
    let window = Window {
        skip: clamp_skip(input.skip),
        top: None,
    };
    graph
        .get_collection(
            "/me/mailFolders/inbox/messageRules",
            &GraphQuery::new(),
            window,
        )
        .await
        .map(GraphList::from)
}

pub async fn list_shared_mailbox_messages(
    graph: &GraphClient,
    input: SharedMailboxInput,
) -> Result<GraphList, GraphError> {
    let user = validate_id(&input.user_id, "user_id")?;
    let folder = validate_folder(&input.folder)?;
    let query = GraphQuery::new()
        .with("$select", MAIL_LIST_FIELDS)
        .with("$orderby", "receivedDateTime desc");
    let window = Window {
        skip: clamp_skip(input.skip),
        top: Some(clamp_top(input.top, DEFAULT_MAX_TOP)),
    };
    graph
        .get_collection(
            &format!("/users/{user}/mailFolders/{folder}/messages"),
            &query,
            window,
        )
        .await
        .map(GraphList::from)
}
