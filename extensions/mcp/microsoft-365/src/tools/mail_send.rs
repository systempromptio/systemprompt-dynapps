//! Mail writes on messages: send, reply, forward, drafts, update, delete and
//! move.
//!
//! Sends, replies, forwards and deletes are destructive in the Belgian
//! annotations (they cannot be taken back); the reply of a write Graph
//! answers with 202/204 is the same summary the Belgian server returned.

use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

use super::mail_read::MessageIdInput;
use super::reply::GraphItem;
use crate::graph::sanitize::{validate_folder, validate_id};
use crate::graph::{GraphClient, GraphError, GraphId};

pub const SEND_MAIL_DESCRIPTION: &str = "Send an email message from the user's mailbox.";
pub const REPLY_MAIL_DESCRIPTION: &str =
    "Reply to an email message, to the sender or with reply_all to every recipient.";
pub const FORWARD_MAIL_DESCRIPTION: &str =
    "Forward an email message to other recipients, with an optional comment above it.";
pub const CREATE_DRAFT_DESCRIPTION: &str = "Create a draft email message in the Drafts folder; \
    send it later with d365_send_draft.";
pub const SEND_DRAFT_DESCRIPTION: &str = "Send an existing draft email message.";
pub const UPDATE_MAIL_DESCRIPTION: &str = "Update an email message: mark it read or unread, \
    change its importance or set its categories (an empty list clears them).";
pub const DELETE_MAIL_DESCRIPTION: &str = "Delete an email message (moves it to Deleted \
    Items, where it can be recovered).";
pub const MOVE_MAIL_DESCRIPTION: &str = "Move an email message to another folder, by \
    well-known name (\"inbox\", \"archive\", \"deleteditems\", \"drafts\", \"junkemail\", \
    \"sentitems\") or folder ID.";

fn default_importance() -> String {
    "normal".to_owned()
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SendMailInput {
    #[schemars(description = "Recipient email addresses.")]
    pub to: Vec<String>,
    #[schemars(description = "Email subject line.")]
    pub subject: String,
    #[schemars(description = "Email body content (plain text or HTML).")]
    pub body: String,
    #[schemars(description = "CC recipient email addresses (optional).")]
    #[serde(default)]
    pub cc: Vec<String>,
    #[schemars(description = "BCC recipient email addresses (optional).")]
    #[serde(default)]
    pub bcc: Vec<String>,
    #[schemars(description = "If true, body is HTML (default false).")]
    #[serde(default)]
    pub is_html: bool,
    #[schemars(
        description = "Message importance: \"low\", \"normal\" or \"high\" (default \"normal\")."
    )]
    #[serde(default = "default_importance")]
    pub importance: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReplyMailInput {
    #[schemars(description = "The unique ID of the message to reply to.")]
    pub message_id: GraphId,
    #[schemars(description = "The reply body content (plain text).")]
    pub body: String,
    #[schemars(description = "If true, reply to all recipients (default false).")]
    #[serde(default)]
    pub reply_all: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ForwardMailInput {
    #[schemars(description = "The unique ID of the message to forward.")]
    pub message_id: GraphId,
    #[schemars(description = "Recipient email addresses to forward to.")]
    pub to: Vec<String>,
    #[schemars(description = "Optional comment above the forwarded message.")]
    #[serde(default)]
    pub comment: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateDraftInput {
    #[schemars(description = "Email subject line.")]
    pub subject: String,
    #[schemars(description = "Email body content (plain text or HTML).")]
    pub body: String,
    #[schemars(description = "Recipient email addresses.")]
    pub to: Vec<String>,
    #[schemars(description = "CC recipient email addresses (optional).")]
    #[serde(default)]
    pub cc: Vec<String>,
    #[schemars(description = "If true, body is HTML (default false).")]
    #[serde(default)]
    pub is_html: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateMailInput {
    #[schemars(description = "The unique ID of the message to update.")]
    pub message_id: GraphId,
    #[schemars(description = "True marks the message read, false unread; omitted leaves it.")]
    #[serde(default)]
    #[schemars(with = "bool")]
    pub is_read: Option<bool>,
    #[schemars(description = "Importance: \"low\", \"normal\" or \"high\"; omitted leaves it.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub importance: Option<String>,
    #[schemars(
        description = "Category names to assign; an empty list clears them, omitted leaves them."
    )]
    #[serde(default)]
    #[schemars(with = "Vec<String>")]
    pub categories: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct MoveMailInput {
    #[schemars(description = "The unique ID of the message to move.")]
    pub message_id: GraphId,
    #[schemars(
        description = "Destination folder: \"inbox\", \"archive\", \"deleteditems\", \"drafts\", \"junkemail\", \"sentitems\" or a folder ID."
    )]
    pub destination_folder: String,
}

pub fn recipients(addresses: &[String]) -> Value {
    Value::Array(
        addresses
            .iter()
            .map(|address| json!({"emailAddress": {"address": address}}))
            .collect(),
    )
}

fn body_of(content: &str, is_html: bool) -> Value {
    let kind = if is_html { "HTML" } else { "Text" };
    json!({"contentType": kind, "content": content})
}

pub async fn send_mail(graph: &GraphClient, input: SendMailInput) -> Result<GraphItem, GraphError> {
    let mut message = json!({
        "subject": input.subject,
        "body": body_of(&input.body, input.is_html),
        "toRecipients": recipients(&input.to),
        "importance": input.importance,
    });
    if !input.cc.is_empty() {
        message["ccRecipients"] = recipients(&input.cc);
    }
    if !input.bcc.is_empty() {
        message["bccRecipients"] = recipients(&input.bcc);
    }
    graph
        .post_json("/me/sendMail", json!({"message": message}))
        .await?;
    Ok(GraphItem::new(
        json!({"sent": true, "to": input.to, "subject": input.subject}),
    ))
}

pub async fn reply_mail(
    graph: &GraphClient,
    input: ReplyMailInput,
) -> Result<GraphItem, GraphError> {
    let id = validate_id(&input.message_id, "message_id")?;
    let action = if input.reply_all { "replyAll" } else { "reply" };
    graph
        .post_json(
            &format!("/me/messages/{id}/{action}"),
            json!({"comment": input.body}),
        )
        .await?;
    Ok(GraphItem::new(
        json!({"replied": true, "message_id": id, "reply_all": input.reply_all}),
    ))
}

pub async fn forward_mail(
    graph: &GraphClient,
    input: ForwardMailInput,
) -> Result<GraphItem, GraphError> {
    let id = validate_id(&input.message_id, "message_id")?;
    let mut payload = json!({"toRecipients": recipients(&input.to)});
    if !input.comment.is_empty() {
        payload["comment"] = Value::String(input.comment.clone());
    }
    graph
        .post_json(&format!("/me/messages/{id}/forward"), payload)
        .await?;
    Ok(GraphItem::new(
        json!({"forwarded": true, "message_id": id, "to": input.to}),
    ))
}

pub async fn create_draft(
    graph: &GraphClient,
    input: CreateDraftInput,
) -> Result<GraphItem, GraphError> {
    let mut message = json!({
        "subject": input.subject,
        "body": body_of(&input.body, input.is_html),
        "toRecipients": recipients(&input.to),
    });
    if !input.cc.is_empty() {
        message["ccRecipients"] = recipients(&input.cc);
    }
    graph
        .post_json("/me/messages", message)
        .await
        .map(GraphItem::new)
}

pub async fn send_draft(
    graph: &GraphClient,
    input: MessageIdInput,
) -> Result<GraphItem, GraphError> {
    let id = validate_id(&input.message_id, "message_id")?;
    graph.post_empty(&format!("/me/messages/{id}/send")).await?;
    Ok(GraphItem::new(json!({"sent": true, "message_id": id})))
}

pub async fn update_mail(
    graph: &GraphClient,
    input: UpdateMailInput,
) -> Result<GraphItem, GraphError> {
    let id = validate_id(&input.message_id, "message_id")?;
    let mut updates = serde_json::Map::new();
    if let Some(read) = input.is_read {
        updates.insert("isRead".to_owned(), Value::Bool(read));
    }
    if let Some(importance) = &input.importance {
        updates.insert("importance".to_owned(), Value::String(importance.clone()));
    }
    if let Some(categories) = &input.categories {
        updates.insert("categories".to_owned(), json!(categories));
    }
    if updates.is_empty() {
        return Ok(GraphItem::new(
            json!({"updated": false, "reason": "No properties to update"}),
        ));
    }
    graph
        .patch_json(&format!("/me/messages/{id}"), Value::Object(updates))
        .await
        .map(GraphItem::new)
}

pub async fn delete_mail(
    graph: &GraphClient,
    input: MessageIdInput,
) -> Result<GraphItem, GraphError> {
    let id = validate_id(&input.message_id, "message_id")?;
    graph.delete(&format!("/me/messages/{id}")).await?;
    Ok(GraphItem::new(json!({"deleted": true, "message_id": id})))
}

pub async fn move_mail(graph: &GraphClient, input: MoveMailInput) -> Result<GraphItem, GraphError> {
    let id = validate_id(&input.message_id, "message_id")?;
    let destination = validate_folder(&input.destination_folder)?;
    graph
        .post_json(
            &format!("/me/messages/{id}/move"),
            json!({"destinationId": destination}),
        )
        .await
        .map(GraphItem::new)
}
