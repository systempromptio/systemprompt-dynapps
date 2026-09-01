//! The Atlassian REST client used for indexing.
//!
//! Basic auth with the *caller's* e-mail and API token, taken from the
//! request headers of the `sync_knowledge` call. Atlassian enforces the
//! caller's permissions on every read, which is what makes "grant everything
//! fetched to the caller" correct: a page this client cannot see is a page
//! the caller cannot see.

use crate::downstream::AtlassianCredentials;
use crate::error::KnowledgeBankError;
// JSON: protocol boundary. Confluence and Jira return large, deeply nested,
// expansion-dependent payloads whose shape changes with the query parameters;
// the connectors reach for the handful of fields they need by pointer rather
// than modelling a schema Atlassian varies per request.
use serde_json::Value;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// An authenticated Atlassian REST client for one site and one caller.
pub struct AtlassianClient {
    http: reqwest::Client,
    credentials: AtlassianCredentials,
}

impl std::fmt::Debug for AtlassianClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtlassianClient")
            .field("base_url", &self.credentials.base_url)
            .finish_non_exhaustive()
    }
}

impl AtlassianClient {
    pub fn new(credentials: AtlassianCredentials) -> Result<Self, KnowledgeBankError> {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()?;
        Ok(Self { http, credentials })
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.credentials.base_url
    }

    pub async fn get(&self, path_and_query: &str) -> Result<Value, KnowledgeBankError> {
        let url = self.absolute(path_and_query);
        let response = self
            .http
            .get(&url)
            .basic_auth(&self.credentials.email, Some(&self.credentials.api_token))
            .header("Accept", "application/json")
            .send()
            .await?;
        Self::decode(path_and_query, response).await
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value, KnowledgeBankError> {
        let url = self.absolute(path);
        let response = self
            .http
            .post(&url)
            .basic_auth(&self.credentials.email, Some(&self.credentials.api_token))
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;
        Self::decode(path, response).await
    }

    fn absolute(&self, path_and_query: &str) -> String {
        if path_and_query.starts_with("http://") || path_and_query.starts_with("https://") {
            return path_and_query.to_owned();
        }
        format!(
            "{}/{}",
            self.credentials.base_url,
            path_and_query.trim_start_matches('/')
        )
    }

    async fn decode(path: &str, response: reqwest::Response) -> Result<Value, KnowledgeBankError> {
        let status = response.status();
        if !status.is_success() {
            let detail = response.text().await.unwrap_or_default();
            let detail: String = detail.chars().take(300).collect();
            return Err(KnowledgeBankError::Backend(format!(
                "atlassian {path} returned HTTP {status}: {detail}"
            )));
        }
        Ok(response.json().await?)
    }
}

#[must_use]
pub fn json_str<'a>(value: &'a Value, pointer: &str) -> Option<&'a str> {
    value.pointer(pointer).and_then(Value::as_str)
}

#[must_use]
pub fn json_date(value: &Value, pointer: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    json_str(value, pointer)
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc))
}
