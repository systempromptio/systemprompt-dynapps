//! Upload sessions.
//!
//! Graph answers `createUploadSession` with a pre-authenticated `uploadUrl`
//! (Outlook for mail attachments, `SharePoint` for drive items) that takes
//! the bytes in `Content-Range` chunks and must not get the user's bearer.
//! Only a URL on Microsoft's upload hosts, or on the configured Graph
//! endpoint's own origin (a national cloud or a simulation), is admitted; it
//! passes core's outbound guard on every chunk. The Belgian server PUT to
//! whatever URL the model handed it. A chunk is at most [`MAX_CHUNK_BYTES`].

use reqwest::header::CONTENT_RANGE;
use systemprompt::models::net::{LoopbackPolicy, validate_outbound_url};
use url::Url;

use super::send::finish;
use super::{GraphClient, GraphError};

pub const UPLOAD_HOSTS: [&str; 3] = [
    "outlook.office.com",
    "outlook.office365.com",
    "graph.microsoft.com",
];
pub const UPLOAD_HOST_SUFFIX: &str = ".sharepoint.com";
// Why: Graph takes upload-session ranges of at most 4 MiB, in multiples of
// 320 KiB for drive items; ten of those serves mail and drive alike.
pub const MAX_CHUNK_BYTES: usize = 4 * 1024 * 1024;
pub const UPLOAD_CHUNK_BYTES: usize = 10 * 320 * 1024;

/// What one chunk PUT answered: its status and its JSON body (`Null` when
/// empty), the completed attachment or item on the last chunk.
#[derive(Debug, Clone, PartialEq)]
pub struct ChunkReply {
    pub status: u16,
    // JSON: the upload session's state or the created Graph resource.
    pub body: serde_json::Value,
}

pub fn check_chunk_size(len: u64) -> Result<(), GraphError> {
    if len <= MAX_CHUNK_BYTES as u64 {
        return Ok(());
    }
    Err(GraphError::Invalid(format!(
        "an upload chunk can be at most 4 MiB ({MAX_CHUNK_BYTES} bytes) and this one is \
         {len} bytes: send the file in byte ranges of at most 4 MiB"
    )))
}

fn on_upload_host(url: &Url) -> bool {
    url.scheme() == "https"
        && url.host_str().is_some_and(|host| {
            let host = host.to_ascii_lowercase();
            UPLOAD_HOSTS.contains(&host.as_str()) || host.ends_with(UPLOAD_HOST_SUFFIX)
        })
}

impl GraphClient {
    pub fn admit_upload_url(&self, raw: &str) -> Result<Url, GraphError> {
        let Ok(url) = Url::parse(raw.trim()) else {
            return Err(GraphError::Invalid("upload_url is not a URL".to_owned()));
        };
        let own_origin = url.origin() == self.endpoint().base().origin();
        if !(own_origin || on_upload_host(&url))
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(GraphError::OffEndpoint);
        }
        let loopback = if own_origin {
            LoopbackPolicy::Allow
        } else {
            LoopbackPolicy::Deny
        };
        match validate_outbound_url(url.as_str(), loopback) {
            Ok(admitted) => Ok(admitted),
            Err(error) => {
                tracing::warn!(%error, "upload URL refused by the outbound guard");
                Err(GraphError::OffEndpoint)
            },
        }
    }

    pub async fn put_chunk(
        &self,
        upload_url: &Url,
        chunk: &[u8],
        start: u64,
        total: u64,
    ) -> Result<ChunkReply, GraphError> {
        if chunk.is_empty() {
            return Err(GraphError::Invalid(
                "an upload chunk cannot be empty".to_owned(),
            ));
        }
        check_chunk_size(chunk.len() as u64)?;
        let end = start + chunk.len() as u64 - 1;
        if end >= total {
            return Err(GraphError::Invalid(format!(
                "chunk bytes {start}-{end} run past total_size {total}"
            )));
        }
        let response = self
            .http
            .put(upload_url.clone())
            .header(CONTENT_RANGE, format!("bytes {start}-{end}/{total}"))
            .body(chunk.to_vec())
            .send()
            .await?;
        let reply = finish(response, self.limits().max_response_bytes).await?;
        Ok(ChunkReply {
            status: reply.status,
            body: reply.json()?,
        })
    }

    // JSON: the resource the session created, from the last chunk's reply.
    pub async fn upload_in_chunks(
        &self,
        upload_url: &str,
        bytes: &[u8],
    ) -> Result<serde_json::Value, GraphError> {
        let url = self.admit_upload_url(upload_url)?;
        let total = bytes.len() as u64;
        let mut last = serde_json::Value::Null;
        let mut start = 0u64;
        for chunk in bytes.chunks(UPLOAD_CHUNK_BYTES) {
            last = self.put_chunk(&url, chunk, start, total).await?.body;
            start += chunk.len() as u64;
        }
        Ok(last)
    }
}
