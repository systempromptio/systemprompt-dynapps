//! Mail attachments through the shared file layer.
//!
//! A downloaded attachment is kept as one of the caller's documents and comes
//! back as a `FileOut` (the Belgian server returned its base64 to the model);
//! an attachment to add comes in as a `FileRef`, or as base64 only up to the
//! inline limit. Graph takes an attachment of under 3 MB in one POST and a
//! larger one only through an upload session, which this server fills in
//! chunks itself so the bytes never pass through the model.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rmcp::ErrorData as McpError;
use schemars::JsonSchema;
use serde::Deserialize;
// JSON: attachments are Graph resources passed through as Graph returns them.
use serde_json::{Value, json};
use systemprompt_mcp_shared::files::{DownloadedFile, FileCall, FileRef, McpFiles};

use super::reply::GraphItem;
use crate::graph::sanitize::validate_id;
use crate::graph::{GraphClient, GraphError, GraphId, GraphQuery};

// Why: Graph's limit for a file attachment POSTed whole; larger ones need an
// upload session.
pub const SINGLE_POST_MAX_BYTES: usize = 3 * 1024 * 1024;
const DEFAULT_MIME: &str = "application/octet-stream";

pub const GET_MAIL_ATTACHMENT_DESCRIPTION: &str = "Get an email attachment as a file: it is \
    saved to the user's documents and returned as a reference with a download link (the \
    content is not inlined).";
pub const ADD_MAIL_ATTACHMENT_DESCRIPTION: &str = "Attach a file to a draft message. Pass the \
    file as `file` (an upload, blob or document reference) or, for files of up to 64 KB, as \
    `content_base64`. Files of 3 MB or more are sent through an upload session automatically.";
pub const CREATE_UPLOAD_SESSION_DESCRIPTION: &str = "Create an upload session for a large mail \
    attachment (over 3 MB, up to about 150 MB) and return its uploadUrl. \
    d365_add_mail_attachment with `file` does the whole upload in one call.";
pub const UPLOAD_CHUNK_DESCRIPTION: &str = "Upload one byte range of a file to an upload \
    session's uploadUrl (from d365_create_mail_attachment_upload_session). The chunk is \
    `chunk_base64` (up to 64 KB) or `file`; the last chunk returns the attachment.";
pub const DELETE_MAIL_ATTACHMENT_DESCRIPTION: &str = "Remove an attachment from a draft message.";

fn default_mime() -> String {
    DEFAULT_MIME.to_owned()
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AttachmentIdInput {
    #[schemars(description = "The unique ID of the message.")]
    pub message_id: GraphId,
    #[schemars(description = "The unique ID of the attachment.")]
    pub attachment_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AddAttachmentInput {
    #[schemars(description = "The draft message ID to attach to.")]
    pub message_id: GraphId,
    #[schemars(description = "Display name of the attachment.")]
    pub filename: String,
    #[schemars(
        description = "File bytes as base64, only for files of up to 64 KB; otherwise use `file`."
    )]
    #[serde(default)]
    pub content_base64: String,
    #[schemars(
        description = "MIME type (e.g. \"application/pdf\"); the default keeps the file's own."
    )]
    #[serde(default = "default_mime")]
    pub content_type: String,
    #[schemars(
        description = "The file to attach, by reference (upload_id, blob_ref or document_id)."
    )]
    #[serde(default)]
    pub file: FileRef,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UploadSessionInput {
    #[schemars(description = "The draft message ID to attach to.")]
    pub message_id: GraphId,
    #[schemars(description = "Display name of the attachment.")]
    pub filename: String,
    #[schemars(description = "Total file size in bytes.")]
    pub size_bytes: i64,
    #[schemars(description = "MIME type.")]
    #[serde(default = "default_mime")]
    pub content_type: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UploadChunkInput {
    #[schemars(description = "The uploadUrl from d365_create_mail_attachment_upload_session.")]
    pub upload_url: String,
    #[schemars(description = "This chunk's bytes as base64 (up to 64 KB); otherwise use `file`.")]
    #[serde(default)]
    pub chunk_base64: String,
    #[schemars(description = "First byte offset of this chunk in the full file.")]
    pub byte_range_start: i64,
    #[schemars(description = "Total size of the full file in bytes.")]
    pub total_size: i64,
    #[schemars(description = "This chunk's bytes as a file reference, instead of chunk_base64.")]
    #[serde(default)]
    pub file: FileRef,
}

fn attachment_path(message: &str, attachment: &str) -> Result<String, GraphError> {
    let message = validate_id(message, "message_id")?;
    let attachment = validate_id(attachment, "attachment_id")?;
    Ok(format!("/me/messages/{message}/attachments/{attachment}"))
}

pub(super) fn one_source(base64: &str, file: &FileRef, field: &str) -> Result<FileRef, GraphError> {
    match (base64.is_empty(), *file == FileRef::default()) {
        (false, true) => Ok(FileRef {
            content_base64: base64.to_owned(),
            ..FileRef::default()
        }),
        (true, false) => Ok(file.clone()),
        _ => Err(GraphError::Invalid(format!(
            "pass exactly one of {field} or file"
        ))),
    }
}

pub(super) fn text_of(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_owned)
}

pub async fn get_mail_attachment(
    graph: &GraphClient,
    files: &McpFiles,
    call: FileCall<'_>,
    input: AttachmentIdInput,
) -> Result<GraphItem, McpError> {
    let path = attachment_path(&input.message_id, &input.attachment_id)?;
    let mut attachment = graph.get_json(&path, &GraphQuery::new()).await?;
    let Some(encoded) = text_of(&attachment, "contentBytes") else {
        return Ok(GraphItem::new(attachment));
    };
    if let Some(fields) = attachment.as_object_mut() {
        fields.remove("contentBytes");
    }
    let bytes = match STANDARD.decode(encoded) {
        Ok(bytes) => bytes,
        Err(error) => {
            let message = format!("Graph attachment bytes are not base64: {error}");
            return Err(GraphError::api(502, None, &message).into());
        },
    };
    let download = DownloadedFile {
        name: text_of(&attachment, "name").unwrap_or_else(|| "attachment".to_owned()),
        mime: text_of(&attachment, "contentType").unwrap_or_else(default_mime),
        bytes,
    };
    let file = files.store_download(call, download).await?;
    attachment["file"] = serde_json::to_value(&file).map_err(GraphError::Decode)?;
    Ok(GraphItem::new(attachment))
}

async fn open_session(
    graph: &GraphClient,
    message: &str,
    name: &str,
    size: u64,
    mime: &str,
) -> Result<Value, GraphError> {
    let message = validate_id(message, "message_id")?;
    let item = json!({"AttachmentItem": {
        "attachmentType": "file", "name": name, "size": size, "contentType": mime,
    }});
    graph
        .post_json(
            &format!("/me/messages/{message}/attachments/createUploadSession"),
            item,
        )
        .await
}

pub async fn add_mail_attachment(
    graph: &GraphClient,
    files: &McpFiles,
    call: FileCall<'_>,
    input: AddAttachmentInput,
) -> Result<GraphItem, McpError> {
    let message = validate_id(&input.message_id, "message_id")?;
    let source = one_source(&input.content_base64, &input.file, "content_base64")?;
    let resolved = files.resolve_ref(call, &source).await?;
    let mime = if input.content_type == DEFAULT_MIME {
        resolved.mime.clone()
    } else {
        input.content_type.clone()
    };
    let bytes = resolved.read_all().await?;
    if bytes.len() < SINGLE_POST_MAX_BYTES {
        let body = json!({
            "@odata.type": "#microsoft.graph.fileAttachment",
            "name": input.filename,
            "contentType": mime,
            "contentBytes": STANDARD.encode(&bytes),
        });
        let path = format!("/me/messages/{message}/attachments");
        return Ok(GraphItem::new(graph.post_json(&path, body).await?));
    }
    let session = open_session(graph, message, &input.filename, bytes.len() as u64, &mime).await?;
    let upload_url = text_of(&session, "uploadUrl")
        .ok_or_else(|| GraphError::api(502, None, "Graph opened no upload session"))?;
    let created = graph.upload_in_chunks(&upload_url, &bytes).await?;
    Ok(GraphItem::new(json!({
        "attached": true,
        "message_id": message,
        "name": input.filename,
        "size": bytes.len(),
        "upload_session": true,
        "attachment": created,
    })))
}

pub async fn create_upload_session(
    graph: &GraphClient,
    input: UploadSessionInput,
) -> Result<GraphItem, GraphError> {
    let size = match u64::try_from(input.size_bytes) {
        Ok(size) if size > 0 => size,
        _ => {
            return Err(GraphError::Invalid(
                "size_bytes must be positive".to_owned(),
            ));
        },
    };
    open_session(
        graph,
        &input.message_id,
        &input.filename,
        size,
        &input.content_type,
    )
    .await
    .map(GraphItem::new)
}

pub async fn upload_attachment_chunk(
    graph: &GraphClient,
    files: &McpFiles,
    call: FileCall<'_>,
    input: UploadChunkInput,
) -> Result<GraphItem, McpError> {
    let url = graph.admit_upload_url(&input.upload_url)?;
    let (Ok(start), Ok(total)) = (
        u64::try_from(input.byte_range_start),
        u64::try_from(input.total_size),
    ) else {
        return Err(GraphError::Invalid(
            "byte_range_start and total_size must not be negative".to_owned(),
        )
        .into());
    };
    let source = one_source(&input.chunk_base64, &input.file, "chunk_base64")?;
    let chunk = files.resolve_ref(call, &source).await?.read_all().await?;
    let reply = graph.put_chunk(&url, &chunk, start, total).await?;
    Ok(GraphItem::new(
        json!({"status_code": reply.status, "response": reply.body}),
    ))
}

pub async fn delete_mail_attachment(
    graph: &GraphClient,
    input: AttachmentIdInput,
) -> Result<GraphItem, GraphError> {
    let path = attachment_path(&input.message_id, &input.attachment_id)?;
    graph.delete(&path).await?;
    Ok(GraphItem::new(json!({
        "deleted": true,
        "message_id": input.message_id,
        "attachment_id": input.attachment_id,
    })))
}
