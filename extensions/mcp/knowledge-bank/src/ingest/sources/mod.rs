//! Source connectors.
//!
//! Each turns one configured category into a [`FetchOutcome`]: normalised
//! documents, the set of refs that exist upstream, and whether that set is
//! complete enough to drop grants against.
//!
//! [`SourceConnector`] is the seam: the shipped implementation reads
//! Confluence and Jira with the caller's own credentials, and the tests
//! implement it in memory so the write path can be proven without Atlassian.

pub mod atlassian;
pub mod confluence;
pub mod jira;

pub use atlassian::AtlassianClient;

use super::document::FetchOutcome;
use crate::config::{SourceConfig, SourceKind};
use crate::downstream::AtlassianCredentials;
use crate::error::KnowledgeBankError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashSet;

/// Everything a connector needs for one run of one source.
pub struct FetchRequest<'a> {
    pub source_type: &'a str,
    pub source: &'a SourceConfig,
    pub last_sync: Option<DateTime<Utc>>,
    pub include_comments: bool,
    pub known_doc_ids: &'a HashSet<String>,
}

impl std::fmt::Debug for FetchRequest<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FetchRequest")
            .field("source_type", &self.source_type)
            .field("last_sync", &self.last_sync)
            .field("known", &self.known_doc_ids.len())
            .finish_non_exhaustive()
    }
}

impl FetchRequest<'_> {
    // Why: a document may be skipped as unchanged only if the index already
    // holds it. A caller's cursor can be newer than a document that was since
    // deleted as an orphan (every reader dropped it), and that document must
    // be read again, not assumed present.
    #[must_use]
    pub fn can_skip(&self, doc_id: &str, updated: Option<DateTime<Utc>>) -> bool {
        match (self.last_sync, updated) {
            (Some(last), Some(updated_at)) => {
                updated_at <= last && self.known_doc_ids.contains(doc_id)
            },
            _ => false,
        }
    }
}

/// The read side of ingestion for one source category.
#[async_trait]
pub trait SourceConnector: Send + Sync {
    async fn fetch(&self, request: &FetchRequest<'_>) -> Result<FetchOutcome, KnowledgeBankError>;
}

/// The shipped connector: Confluence and Jira, read as the caller.
#[derive(Debug)]
pub struct AtlassianConnector {
    client: AtlassianClient,
}

impl AtlassianConnector {
    pub fn new(credentials: AtlassianCredentials) -> Result<Self, KnowledgeBankError> {
        Ok(Self {
            client: AtlassianClient::new(credentials)?,
        })
    }
}

#[async_trait]
impl SourceConnector for AtlassianConnector {
    async fn fetch(&self, request: &FetchRequest<'_>) -> Result<FetchOutcome, KnowledgeBankError> {
        match request.source.kind {
            SourceKind::Confluence => confluence::fetch(request, &self.client).await,
            SourceKind::Jira => jira::fetch(request, &self.client).await,
        }
    }
}
