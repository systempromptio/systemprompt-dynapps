//! The caller's own Atlassian credentials, read from the MCP request headers.
//!
//! Nobody runs this server locally: users reach it through
//! `/api/v1/mcp/knowledge-bank`, and the only credential that travels with a
//! request is theirs. The proxy forwards every header but `host` and the
//! proxy-auth header, lower-cased, and rmcp exposes them on `tools/call` as
//! `http::request::Parts`. `sync_knowledge` reads Confluence and Jira *as the
//! caller* with these, so what it indexes is exactly what that account may
//! read. Values are never logged; audit rows carry only the caller id.

use crate::config::AtlassianSettings;
use crate::error::KnowledgeBankError;
use axum::http::HeaderMap;

pub const HEADER_EMAIL: &str = "x-atlassian-email";
pub const HEADER_API_TOKEN: &str = "x-atlassian-api-token";
pub const HEADER_BASE_URL: &str = "x-atlassian-base-url";

/// The caller's Atlassian account, as Basic-auth material.
#[derive(Clone)]
pub struct AtlassianCredentials {
    pub base_url: String,
    pub email: String,
    pub api_token: String,
}

impl std::fmt::Debug for AtlassianCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtlassianCredentials")
            .field("base_url", &self.base_url)
            .field("email", &self.email)
            .finish_non_exhaustive()
    }
}

impl AtlassianCredentials {
    pub fn from_headers(
        headers: &HeaderMap,
        settings: &AtlassianSettings,
    ) -> Result<Self, KnowledgeBankError> {
        let email = required(headers, HEADER_EMAIL)?;
        let api_token = required(headers, HEADER_API_TOKEN)?;
        let base_url = optional(headers, HEADER_BASE_URL)
            .unwrap_or_else(|| settings.base_url.trim().to_owned())
            .trim_end_matches('/')
            .to_owned();
        if base_url.is_empty() {
            return Err(KnowledgeBankError::Config(
                "atlassian.base_url is not configured and no x-atlassian-base-url header was sent"
                    .to_owned(),
            ));
        }
        Ok(Self {
            base_url,
            email,
            api_token,
        })
    }
}

fn optional(headers: &HeaderMap, name: &'static str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
}

fn required(headers: &HeaderMap, name: &'static str) -> Result<String, KnowledgeBankError> {
    optional(headers, name).ok_or(KnowledgeBankError::MissingHeader(name))
}
