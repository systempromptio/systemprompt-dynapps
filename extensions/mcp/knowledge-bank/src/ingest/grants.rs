//! Who a freshly indexed document is granted to.
//!
//! An Atlassian document is granted to the caller who synced it — Atlassian
//! enforced the read for them and nobody else. A file-sourced document is
//! team material with no upstream reader list, so it is granted to the
//! subjects its category configures (`role:user` by default).

use crate::config::{SourceConfig, SourceKind};
use crate::principal::Subject;
use systemprompt::identifiers::UserId;

/// The grant a document receives once stored.
#[derive(Debug, Clone)]
pub enum GrantPolicy<'a> {
    Caller(&'a UserId),
    Configured(Vec<Subject>),
}

#[must_use]
pub fn policy_for<'a>(source: &SourceConfig, caller: &'a UserId) -> GrantPolicy<'a> {
    match source.kind {
        SourceKind::Files | SourceKind::GoogleDrive => {
            GrantPolicy::Configured(source.configured_subjects())
        },
        SourceKind::Confluence | SourceKind::Jira => GrantPolicy::Caller(caller),
    }
}
