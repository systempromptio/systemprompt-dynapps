//! File content through the shared file layer.
//!
//! Graph answers an item's `/content` with a 302 to a pre-signed download
//! URL; the Graph client follows it without the user's bearer (the Belgian
//! server did not, so its `d365_get_file_content` returned `null`). Text of
//! up to 256 KiB comes back inline for the model to read, audited like a
//! kept download (`McpFiles::return_inline`); any other file is kept as one
//! of the caller's documents and comes back as a `FileOut`, never as base64
//! and never as Graph's pre-signed URL. An item without Graph's `file` facet
//! (a folder) is refused before any download.
//!
//! An upload comes in as a `FileRef`, or as base64 only up to the inline
//! limit. Graph takes a file of under 4 MiB in one PUT and a larger one
//! through an upload session this server fills itself.

use reqwest::Method;
use rmcp::ErrorData as McpError;
use schemars::JsonSchema;
use serde::Deserialize;
// JSON: drive items are Graph resources passed through as Graph returns them.
use serde_json::{Value, json};
use systemprompt_mcp_shared::files::{DownloadedFile, FileCall, FileOut, FileRef, McpFiles};

use super::drive::{DriveItemInput, item_path, root_path};
use super::mail_attach::{one_source, text_of};
use super::reply::GraphItem;
use crate::graph::sanitize::validate_path;
use crate::graph::{GraphBody, GraphClient, GraphError, GraphQuery, GraphRequest};

// Why: Graph's simple upload (`PUT …/content`) is documented for files up to
// 4 MB on every cloud; larger ones go through `createUploadSession`.
pub const SIMPLE_UPLOAD_MAX_BYTES: usize = 4 * 1024 * 1024;
pub const INLINE_TEXT_MAX_BYTES: usize = 256 * 1024;
const OCTET_STREAM: &str = "application/octet-stream";
const TEXT_MARKERS: [&str; 5] = [
    "text/",
    "application/json",
    "application/xml",
    "+xml",
    "+json",
];
const SAVED_MESSAGE: &str = "Saved to the user's documents: `file` holds the reference and a \
    download link valid for 10 minutes.";

pub const GET_FILE_CONTENT_DESCRIPTION: &str = "Get the content of a file. A text file of up to \
    256 KB comes back as text in `content`; any other file is saved to the user's documents and \
    returned as `file` (a reference with a download link).";
pub const DOWNLOAD_FILE_DESCRIPTION: &str = "Download a drive item: it is saved to the user's \
    documents and returned as `file`, a reference with a download link valid for 10 minutes.";
pub const UPLOAD_FILE_DESCRIPTION: &str = "Upload a file to a OneDrive folder, replacing a file \
    of the same name. Pass the file as `file` (an upload, blob or document reference) or, for \
    files of up to 64 KB, as `content_base64`. Files of 4 MB or more go through an upload \
    session automatically.";

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UploadFileInput {
    #[schemars(
        description = "Destination folder relative to the drive root (e.g. 'Documents/Reports')."
    )]
    pub folder_path: String,
    #[schemars(description = "Name for the uploaded file.")]
    pub filename: String,
    #[schemars(
        description = "File bytes as base64, only for files of up to 64 KB; otherwise use `file`."
    )]
    #[serde(default)]
    pub content_base64: String,
    #[schemars(description = "A drive ID; omit for the user's own OneDrive.")]
    #[serde(default)]
    pub drive_id: String,
    #[schemars(
        description = "The file to upload, by reference (upload_id, blob_ref or document_id)."
    )]
    #[serde(default)]
    pub file: FileRef,
}

fn is_text(mime: &str) -> bool {
    TEXT_MARKERS.iter().any(|marker| mime.contains(marker))
}

fn validate_name(name: &str) -> Result<&str, GraphError> {
    if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\']) {
        return Err(GraphError::Invalid(
            "Invalid filename: it must be a name, not a path".to_owned(),
        ));
    }
    Ok(name)
}

async fn fetch_file(
    graph: &GraphClient,
    input: &DriveItemInput,
) -> Result<(Value, String, Vec<u8>), GraphError> {
    let path = item_path(&input.drive_id, &input.item_id, "")?;
    let metadata = graph.get_json(&path, &GraphQuery::new()).await?;
    let Some(facet) = metadata.get("file") else {
        return Err(GraphError::Invalid(
            "item_id is not a file; list a folder with d365_list_files".to_owned(),
        ));
    };
    let mime = text_of(facet, "mimeType").unwrap_or_default();
    let content = item_path(&input.drive_id, &input.item_id, "/content")?;
    let bytes = graph
        .send(&GraphRequest::new(Method::GET, content))
        .await?
        .body;
    Ok((metadata, mime, bytes))
}

async fn keep(
    files: &McpFiles,
    call: FileCall<'_>,
    metadata: &Value,
    mime: &str,
    bytes: Vec<u8>,
) -> Result<Value, McpError> {
    let download = DownloadedFile {
        name: text_of(metadata, "name").unwrap_or_else(|| "file".to_owned()),
        mime: if mime.is_empty() { OCTET_STREAM } else { mime }.to_owned(),
        bytes,
    };
    let file: FileOut = files.store_download(call, download).await?;
    Ok(serde_json::to_value(&file).map_err(GraphError::Decode)?)
}

pub async fn get_file_content(
    graph: &GraphClient,
    files: &McpFiles,
    call: FileCall<'_>,
    input: DriveItemInput,
) -> Result<GraphItem, McpError> {
    let (metadata, mime, mut bytes) = fetch_file(graph, &input).await?;
    if is_text(&mime) && bytes.len() <= INLINE_TEXT_MAX_BYTES {
        match String::from_utf8(bytes) {
            Ok(content) => {
                files.return_inline(call, content.len() as u64).await?;
                return Ok(GraphItem::new(
                    json!({ "content": content, "mimeType": mime }),
                ));
            },
            Err(not_utf8) => bytes = not_utf8.into_bytes(),
        }
    }
    let file = keep(files, call, &metadata, &mime, bytes).await?;
    Ok(GraphItem::new(json!({
        "name": metadata.get("name"),
        "mimeType": mime,
        "size": metadata.get("size"),
        "webUrl": metadata.get("webUrl"),
        "file": file,
        "message": SAVED_MESSAGE,
    })))
}

pub async fn download_file(
    graph: &GraphClient,
    files: &McpFiles,
    call: FileCall<'_>,
    input: DriveItemInput,
) -> Result<GraphItem, McpError> {
    let (metadata, mime, bytes) = fetch_file(graph, &input).await?;
    let file = keep(files, call, &metadata, &mime, bytes).await?;
    Ok(GraphItem::new(json!({
        "name": metadata.get("name"),
        "size": metadata.get("size"),
        "mimeType": mime,
        "file": file,
        "message": SAVED_MESSAGE,
    })))
}

pub async fn upload_file(
    graph: &GraphClient,
    files: &McpFiles,
    call: FileCall<'_>,
    input: UploadFileInput,
) -> Result<GraphItem, McpError> {
    let folder = validate_path(&input.folder_path)?;
    let name = validate_name(&input.filename)?;
    let relative = format!("{folder}/{name}");
    let content_path = root_path(&input.drive_id, &relative, "/content")?;
    let session_path = root_path(&input.drive_id, &relative, "/createUploadSession")?;
    let source = one_source(&input.content_base64, &input.file, "content_base64")?;
    let bytes = files.resolve_ref(call, &source).await?.read_all().await?;
    if bytes.len() < SIMPLE_UPLOAD_MAX_BYTES {
        let body = GraphBody::Bytes {
            content_type: OCTET_STREAM.to_owned(),
            data: bytes,
        };
        let request = GraphRequest::new(Method::PUT, content_path).with_body(body);
        return Ok(GraphItem::new(graph.send(&request).await?.json()?));
    }
    let replace = json!({ "item": { "@microsoft.graph.conflictBehavior": "replace" } });
    let session = graph.post_json(&session_path, replace).await?;
    let upload_url = text_of(&session, "uploadUrl")
        .ok_or_else(|| GraphError::api(502, None, "Graph opened no upload session"))?;
    let created = graph.upload_in_chunks(&upload_url, &bytes).await?;
    Ok(GraphItem::new(created))
}
