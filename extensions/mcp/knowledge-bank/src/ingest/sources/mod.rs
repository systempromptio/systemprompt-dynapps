//! Source connectors.
//!
//! Each turns one configured category into a [`FetchOutcome`]: normalised
//! documents, the set of refs that exist upstream, and whether that set is
//! complete enough to prune against.
//!
//! [`SourceConnector`] is the seam. The shipped [`ConfiguredConnector`]
//! dispatches on the category's kind: files categories read a directory
//! tree and need nothing from the caller; Confluence and Jira read through
//! the caller's connected Atlassian MCP and refuse to run without it. The
//! tests implement the trait in memory so the write path can be proven
//! without either.

pub mod atlassian;
pub mod confluence;
pub mod files;
pub mod jira;

pub use atlassian::AtlassianMcp;

use super::document::FetchOutcome;
use crate::config::{FilesSettings, SourceConfig, SourceKind};
use crate::error::KnowledgeBankError;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashSet;

/// Everything a connector needs for one run of one source.
pub struct FetchRequest<'a> {
    pub source_type: &'a str,
    pub source: &'a SourceConfig,
    pub files: &'a FilesSettings,
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

/// Confluence and Jira, read as the caller through the connected MCP.
#[derive(Debug)]
pub struct AtlassianConnector {
    client: AtlassianMcp,
}

impl AtlassianConnector {
    #[must_use]
    pub const fn new(client: AtlassianMcp) -> Self {
        Self { client }
    }
}

// Why: the one message every "no Atlassian" branch shares, so a caller
// always learns the same fix.
fn not_connected() -> KnowledgeBankError {
    KnowledgeBankError::AtlassianUnavailable("no Atlassian connection for this run".to_owned())
}

#[async_trait]
impl SourceConnector for AtlassianConnector {
    async fn fetch(&self, request: &FetchRequest<'_>) -> Result<FetchOutcome, KnowledgeBankError> {
        match request.source.kind {
            SourceKind::Confluence => confluence::fetch(request, &self.client).await,
            SourceKind::Jira => jira::fetch(request, &self.client).await,
            SourceKind::Files => files::fetch(request).await,
            SourceKind::GoogleDrive => Err(KnowledgeBankError::InvalidInput(
                "Drive sync uses the durable source queue".to_owned(),
            )),
        }
    }
}

/// Directory trees only; stateless.
#[derive(Debug, Default, Clone, Copy)]
pub struct FilesConnector;

#[async_trait]
impl SourceConnector for FilesConnector {
    async fn fetch(&self, request: &FetchRequest<'_>) -> Result<FetchOutcome, KnowledgeBankError> {
        match request.source.kind {
            SourceKind::Files => files::fetch(request).await,
            SourceKind::Confluence | SourceKind::Jira | SourceKind::GoogleDrive => {
                Err(not_connected())
            },
        }
    }
}

/// The shipped connector: files always, Atlassian when the caller's
/// connected account could be reached.
#[derive(Debug)]
pub struct ConfiguredConnector {
    atlassian: Option<AtlassianConnector>,
    files: FilesConnector,
}

impl ConfiguredConnector {
    #[must_use]
    pub fn new(atlassian: Option<AtlassianMcp>) -> Self {
        Self {
            atlassian: atlassian.map(AtlassianConnector::new),
            files: FilesConnector,
        }
    }
}

#[async_trait]
impl SourceConnector for ConfiguredConnector {
    async fn fetch(&self, request: &FetchRequest<'_>) -> Result<FetchOutcome, KnowledgeBankError> {
        match (request.source.kind, &self.atlassian) {
            (SourceKind::Files, _) => self.files.fetch(request).await,
            (_, Some(atlassian)) => atlassian.fetch(request).await,
            (_, None) => Err(not_connected()),
        }
    }
}
