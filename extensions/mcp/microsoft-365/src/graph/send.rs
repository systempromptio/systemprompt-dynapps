//! The one place a Graph request leaves the process: every URL passes core's
//! SSRF guard here, the caller's bearer is attached, a 429 is retried after
//! its `Retry-After` within bounds, a 401 gets one renewed token, and the
//! body is read up to the response limit.
//!
//! Nothing here logs a URL, a path or a body: Graph paths name mailboxes
//! and people.

use std::fmt;
use std::time::Duration;

use reqwest::header::{CONTENT_TYPE, RETRY_AFTER};
use reqwest::{Method, StatusCode};
use systemprompt::models::net::{LoopbackPolicy, validate_outbound_url};
use url::Url;

use super::error::api_error;
use super::{GraphClient, GraphError, GraphLimits, GraphQuery};

const ERROR_BODY_LIMIT: usize = 64 * 1024;

#[derive(Debug, Clone)]
enum Target {
    Path(String),
    Link(String),
}

#[derive(Clone)]
pub enum GraphBody {
    // JSON: a Graph request body; its shape is the Graph resource the tool writes.
    Json(serde_json::Value),
    Bytes { content_type: String, data: Vec<u8> },
}

impl fmt::Debug for GraphBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(_) => f.write_str("GraphBody::Json(..)"),
            Self::Bytes { content_type, data } => f
                .debug_struct("GraphBody::Bytes")
                .field("content_type", content_type)
                .field("len", &data.len())
                .finish(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphRequest {
    method: Method,
    target: Target,
    query: GraphQuery,
    body: Option<GraphBody>,
    headers: Vec<(String, String)>,
}

impl GraphRequest {
    pub fn new(method: Method, path: impl Into<String>) -> Self {
        Self {
            method,
            target: Target::Path(path.into()),
            query: GraphQuery::new(),
            body: None,
            headers: Vec::new(),
        }
    }

    pub fn link(link: impl Into<String>) -> Self {
        Self {
            target: Target::Link(link.into()),
            ..Self::new(Method::GET, String::new())
        }
    }

    #[must_use]
    pub fn with_query(mut self, query: GraphQuery) -> Self {
        self.query = query;
        self
    }

    #[must_use]
    pub fn with_body(mut self, body: GraphBody) -> Self {
        self.body = Some(body);
        self
    }

    #[must_use]
    pub fn with_header(mut self, name: &str, value: impl Into<String>) -> Self {
        self.headers.push((name.to_owned(), value.into()));
        self
    }
}

pub struct GraphResponse {
    pub status: u16,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

impl fmt::Debug for GraphResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GraphResponse")
            .field("status", &self.status)
            .field("content_type", &self.content_type)
            .field("len", &self.body.len())
            .finish()
    }
}

impl GraphResponse {
    // JSON: Graph resources are open-world; `Null` stands for an empty body (204).
    pub fn json(&self) -> Result<serde_json::Value, GraphError> {
        if self.body.is_empty() {
            return Ok(serde_json::Value::Null);
        }
        Ok(serde_json::from_slice(&self.body)?)
    }
}

pub(super) async fn execute(
    client: &GraphClient,
    request: &GraphRequest,
) -> Result<GraphResponse, GraphError> {
    let url = target_url(client, request)?;
    let limits = client.limits();
    let mut token = client.current_token().await?;
    let mut renewed = false;
    let mut throttled = 0_u32;
    loop {
        let response = attempt(client, request, &url, &token).await?;
        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            if renewed {
                return Err(GraphError::Unauthorized);
            }
            renewed = true;
            tracing::info!("Graph refused the access token; asking the broker again");
            token = client
                .renewed_token(&token)
                .await?
                .ok_or(GraphError::Unauthorized)?;
            continue;
        }
        if status == StatusCode::TOO_MANY_REQUESTS {
            let wait = retry_after(&response, limits);
            if throttled >= limits.max_throttle_retries || wait > limits.max_retry_wait {
                return Err(GraphError::Throttled {
                    retry_after_secs: wait.as_secs().max(1),
                });
            }
            throttled += 1;
            tracing::warn!(
                attempt = throttled,
                wait_ms = wait.as_millis() as u64,
                "Graph throttled the request; retrying after Retry-After"
            );
            tokio::time::sleep(wait).await;
            continue;
        }
        return finish(response, limits.max_response_bytes).await;
    }
}

fn target_url(client: &GraphClient, request: &GraphRequest) -> Result<Url, GraphError> {
    match &request.target {
        Target::Link(link) => client.endpoint().admit_link(link),
        Target::Path(path) => {
            let mut url = client.endpoint().resolve(path)?;
            if !request.query.is_empty() {
                url.set_query(Some(&request.query.encode()));
            }
            Ok(url)
        },
    }
}

async fn attempt(
    client: &GraphClient,
    request: &GraphRequest,
    url: &Url,
    token: &str,
) -> Result<reqwest::Response, GraphError> {
    let admitted = match validate_outbound_url(url.as_str(), LoopbackPolicy::Allow) {
        Ok(admitted) => admitted,
        Err(error) => {
            tracing::warn!(%error, "Graph URL refused by the outbound guard");
            return Err(GraphError::OffEndpoint);
        },
    };
    let mut builder = client
        .http
        .request(request.method.clone(), admitted)
        .bearer_auth(token);
    for (name, value) in &request.headers {
        builder = builder.header(name.as_str(), value.as_str());
    }
    builder = match &request.body {
        Some(GraphBody::Json(value)) => builder.json(value),
        Some(GraphBody::Bytes { content_type, data }) => builder
            .header(CONTENT_TYPE, content_type.as_str())
            .body(data.clone()),
        None => builder,
    };
    Ok(builder.send().await?)
}

fn retry_after(response: &reqwest::Response, limits: &GraphLimits) -> Duration {
    response
        .headers()
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map_or(limits.default_backoff, Duration::from_secs)
}

pub(super) async fn finish(
    mut response: reqwest::Response,
    limit: usize,
) -> Result<GraphResponse, GraphError> {
    let status = response.status();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    if status.is_success() {
        let body = read_capped(&mut response, limit).await?;
        return Ok(GraphResponse {
            status: status.as_u16(),
            content_type,
            body,
        });
    }
    let body = match read_capped(&mut response, ERROR_BODY_LIMIT.min(limit)).await {
        Ok(body) => body,
        Err(error) => {
            tracing::warn!(
                kind = error.kind(),
                status = status.as_u16(),
                "Graph error body unreadable"
            );
            Vec::new()
        },
    };
    Err(api_error(status.as_u16(), &body))
}

async fn read_capped(
    response: &mut reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, GraphError> {
    if response
        .content_length()
        .is_some_and(|declared| declared > limit as u64)
    {
        return Err(GraphError::TooLarge { limit });
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if body.len() + chunk.len() > limit {
            return Err(GraphError::TooLarge { limit });
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}
