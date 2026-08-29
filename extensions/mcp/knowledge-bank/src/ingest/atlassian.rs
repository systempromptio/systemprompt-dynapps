//! Confluence and Jira connectors (REQ-048).
//!
//! The requirement is not just to read these systems but to "respect
//! source-system permissions". That is resolved at index time: each document
//! carries the read-subjects its own source object declares, written into
//! `knowledge_document_grants`, so retrieval never has to call Atlassian to
//! decide who may see a hit.
//!
//! Permission reads fail CLOSED. If restrictions cannot be determined for a
//! page, it is indexed with no grants and is therefore reachable by nobody —
//! never indexed as world-readable. That is the only safe direction: a
//! missing document is a support ticket, a leaked one is an incident.
//!
//! Neither connector can be exercised end-to-end yet. The indexing account
//! has no Jira application licence (every call 401s) and sees exactly one
//! Confluence space; see `rag-access-request.local.md`.

use crate::error::KnowledgeBankError;
use crate::ingest::writer::SourceDocument;
// JSON: protocol boundary. Confluence and Jira return large, deeply nested,
// expansion-dependent payloads whose shape changes with the `expand` query
// parameter; the connector reaches for the handful of fields it needs by
// pointer rather than modelling a schema that Atlassian varies per request.
use serde_json::Value;

pub const SOURCE_CONFLUENCE: &str = "confluence";
pub const SOURCE_JIRA: &str = "jira";

/// Credentials and scope for one Atlassian site.
#[derive(Debug, Clone)]
pub struct AtlassianConfig {
    pub base_url: String,
    pub email: String,
    pub api_token: String,
    pub confluence_spaces: Vec<String>,
    pub jira_projects: Vec<String>,
    pub include_comments: bool,
}

/// An authenticated Atlassian REST client.
#[derive(Debug)]
pub struct AtlassianClient {
    http: reqwest::Client,
    config: AtlassianConfig,
}

impl AtlassianClient {
    pub fn new(config: AtlassianConfig) -> Result<Self, KnowledgeBankError> {
        let http = reqwest::Client::builder().build()?;
        Ok(Self { http, config })
    }

    pub async fn get(&self, path: &str) -> Result<Value, KnowledgeBankError> {
        let url = format!("{}/{}", self.config.base_url.trim_end_matches('/'), path);
        let response = self
            .http
            .get(&url)
            .basic_auth(&self.config.email, Some(&self.config.api_token))
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let detail = response.text().await.unwrap_or_default();
            return Err(KnowledgeBankError::Backend(format!(
                "atlassian {path} returned HTTP {status}: {detail}"
            )));
        }
        Ok(response.json().await?)
    }

    pub async fn fetch_confluence_space(
        &self,
        space_key: &str,
    ) -> Result<Vec<SourceDocument>, KnowledgeBankError> {
        let payload = self
            .get(&format!(
                "wiki/rest/api/content?spaceKey={space_key}&expand=body.storage,version,restrictions.read.restrictions.user,restrictions.read.restrictions.group&limit=100"
            ))
            .await?;

        let results = payload
            .get("results")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                KnowledgeBankError::Backend("confluence response carried no results".to_owned())
            })?;

        Ok(results
            .iter()
            .map(|page| self.confluence_page_to_document(space_key, page))
            .collect())
    }

    fn confluence_page_to_document(&self, space_key: &str, page: &Value) -> SourceDocument {
        let id = page
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        let title = page
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("(untitled)")
            .to_owned();
        let body = page
            .pointer("/body/storage/value")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();

        SourceDocument {
            doc_id: format!("confluence:{space_key}:{id}"),
            source_type: SOURCE_CONFLUENCE.to_owned(),
            source: SOURCE_CONFLUENCE.to_owned(),
            title,
            uri: format!(
                "{}/wiki/spaces/{space_key}/pages/{id}",
                self.config.base_url.trim_end_matches('/')
            ),
            content_hash: content_hash(&body),
            body,
            doc_date: None,
            date_source: None,
            grants: read_restrictions_to_grants(page),
        }
    }
}

pub fn read_restrictions_to_grants(page: &Value) -> Vec<(String, String)> {
    let mut grants = Vec::new();

    if let Some(users) = page
        .pointer("/restrictions/read/restrictions/user/results")
        .and_then(Value::as_array)
    {
        for user in users {
            if let Some(account) = user.get("accountId").and_then(Value::as_str) {
                grants.push(("user".to_owned(), account.to_owned()));
            }
        }
    }
    if let Some(groups) = page
        .pointer("/restrictions/read/restrictions/group/results")
        .and_then(Value::as_array)
    {
        for group in groups {
            if let Some(name) = group.get("name").and_then(Value::as_str) {
                grants.push(("adfs_group".to_owned(), name.to_owned()));
            }
        }
    }
    grants
}

pub fn content_hash(body: &str) -> String {
    let digest = body.bytes().fold(0xcbf2_9ce4_8422_2325u64, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x1000_0000_01b3)
    });
    format!("{digest:016x}")
}
