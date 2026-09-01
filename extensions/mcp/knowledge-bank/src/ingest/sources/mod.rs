//! Source connectors. Each turns one configured category into a
//! [`FetchOutcome`]: normalised documents, the set of refs that exist
//! upstream, and whether that set is complete enough to prune against.

pub mod atlassian;
pub mod confluence;
pub mod files;
pub mod jira;

pub use atlassian::{AtlassianClient, AtlassianCredentials};

use super::document::FetchOutcome;
use super::grants::GrantResolver;
use crate::config::{SourceConfig, SourceKind};
use crate::error::KnowledgeBankError;
use chrono::{DateTime, Utc};

/// Everything a connector needs for one run of one source.
pub struct FetchRequest<'a> {
    pub source_type: &'a str,
    pub source: &'a SourceConfig,
    pub grants: &'a GrantResolver,
    pub last_sync: Option<DateTime<Utc>>,
    pub include_comments: bool,
    pub atlassian: Option<&'a AtlassianClient>,
}

impl std::fmt::Debug for FetchRequest<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FetchRequest")
            .field("source_type", &self.source_type)
            .field("last_sync", &self.last_sync)
            .finish_non_exhaustive()
    }
}

pub async fn fetch(request: &FetchRequest<'_>) -> Result<FetchOutcome, KnowledgeBankError> {
    match request.source.kind {
        SourceKind::Files => files::fetch(request).await,
        SourceKind::Confluence => confluence::fetch(request, require_atlassian(request)?).await,
        SourceKind::Jira => jira::fetch(request, require_atlassian(request)?).await,
    }
}

fn require_atlassian<'a>(
    request: &FetchRequest<'a>,
) -> Result<&'a AtlassianClient, KnowledgeBankError> {
    request.atlassian.ok_or_else(|| {
        KnowledgeBankError::NotConfigured(format!(
            "source '{}' needs the Atlassian service account: set atlassian_base_url, \
             atlassian_email and atlassian_api_token in the profile secret store",
            request.source_type
        ))
    })
}
