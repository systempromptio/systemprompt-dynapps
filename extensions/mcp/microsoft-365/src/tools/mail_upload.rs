//! The Belgian upload-session pair for large mail attachments, bound to the
//! caller.
//!
//! `d365_create_mail_attachment_upload_session` opens a session on a draft
//! and records its `uploadUrl` for the caller in the client's
//! [`UploadSessions`](crate::graph::upload_sessions::UploadSessions).
//! `d365_upload_attachment_chunk` sends one byte range only to a URL recorded
//! for the same caller and not yet expired, and only on Microsoft's upload
//! hosts; a URL from anywhere else (another tenant's session included) is
//! refused before the file is opened. A chunk over Graph's 4 MiB limit is
//! refused before it is read or decoded. `d365_add_mail_attachment` does the
//! whole upload in one call; this pair keeps the Belgian surface.

use chrono::Utc;
use rmcp::ErrorData as McpError;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use systemprompt::identifiers::UserId;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_shared::files::{FileCall, FileRef, McpFiles};

use super::mail_attach::{default_mime, one_source, open_session, text_of};
use super::reply::GraphItem;
use crate::graph::upload::check_chunk_size;
use crate::graph::{GraphClient, GraphError, GraphId};

pub const CREATE_UPLOAD_SESSION_DESCRIPTION: &str = "Create an upload session for a large mail \
    attachment (over 3 MB, up to about 150 MB) and return its uploadUrl, which only you can \
    send chunks to, for up to an hour. d365_add_mail_attachment with `file` does the whole \
    upload in one call.";
pub const UPLOAD_CHUNK_DESCRIPTION: &str = "Upload one byte range of a file to the uploadUrl \
    that d365_create_mail_attachment_upload_session returned to you; any other URL, or one \
    past its expiry, is refused. The chunk is `chunk_base64` (up to 64 KB) or `file`, at most \
    4 MiB; the last chunk returns the attachment.";

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
    #[schemars(
        description = "The uploadUrl that d365_create_mail_attachment_upload_session returned to you."
    )]
    pub upload_url: String,
    #[schemars(description = "This chunk's bytes as base64 (up to 64 KB); otherwise use `file`.")]
    #[serde(default)]
    pub chunk_base64: String,
    #[schemars(description = "First byte offset of this chunk in the full file.")]
    pub byte_range_start: i64,
    #[schemars(description = "Total size of the full file in bytes.")]
    pub total_size: i64,
    #[schemars(
        description = "This chunk's bytes (at most 4 MiB) as a file reference, instead of chunk_base64."
    )]
    #[serde(default)]
    pub file: FileRef,
}

fn session_owner(ctx: &RequestContext) -> Result<&UserId, GraphError> {
    ctx.user
        .as_ref()
        .map(|user| &user.id)
        .ok_or_else(|| GraphError::Invalid("an upload session needs a signed-in caller".to_owned()))
}

fn decoded_len(base64: &str) -> u64 {
    let symbols = base64
        .bytes()
        .filter(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/'))
        .count() as u64;
    symbols * 3 / 4
}

pub async fn create_upload_session(
    graph: &GraphClient,
    _files: &McpFiles,
    call: FileCall<'_>,
    input: UploadSessionInput,
) -> Result<GraphItem, McpError> {
    let owner = session_owner(call.ctx)?;
    let size = match u64::try_from(input.size_bytes) {
        Ok(size) if size > 0 => size,
        _ => {
            return Err(GraphError::Invalid("size_bytes must be positive".to_owned()).into());
        },
    };
    let session = open_session(
        graph,
        &input.message_id,
        &input.filename,
        size,
        &input.content_type,
    )
    .await?;
    if let Some(upload_url) = text_of(&session, "uploadUrl") {
        let expiry = text_of(&session, "expirationDateTime");
        graph
            .upload_sessions()
            .issue(&upload_url, owner, expiry.as_deref(), Utc::now());
    }
    Ok(GraphItem::new(session))
}

pub async fn upload_attachment_chunk(
    graph: &GraphClient,
    files: &McpFiles,
    call: FileCall<'_>,
    input: UploadChunkInput,
) -> Result<GraphItem, McpError> {
    let owner = session_owner(call.ctx)?;
    graph
        .upload_sessions()
        .admit(&input.upload_url, owner, Utc::now())?;
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
    check_chunk_size(decoded_len(&input.chunk_base64))?;
    let resolved = files.resolve_ref(call, &source).await?;
    check_chunk_size(resolved.size_bytes.unwrap_or_default())?;
    let chunk = resolved.read_all().await?;
    let reply = graph.put_chunk(&url, &chunk, start, total).await?;
    Ok(GraphItem::new(
        json!({"status_code": reply.status, "response": reply.body}),
    ))
}
