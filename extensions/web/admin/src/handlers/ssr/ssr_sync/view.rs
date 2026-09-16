//! View types for `/admin/sync`.

use serde::Serialize;

use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::sync::plane::{ActionCounts, DriftKpi, DriftRow};
use crate::repositories::sync::sources::SourcesView;

// Why: A hash as the page prints it: the first twelve characters, with the full
// value in a title attribute.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct HashView {
    pub short: String,
    pub full: String,
}

impl HashView {
    pub(crate) fn of(full: &str) -> Option<Self> {
        (!full.is_empty()).then(|| Self {
            short: full.chars().take(12).collect(),
            full: full.to_owned(),
        })
    }
}

// Why: "last applied ab12… by Ed on 16 Sep (overwrite)"; `declared_changed`
// says the declaration's hash differs from the one last applied.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AppliedView {
    pub hash: Option<HashView>,
    pub mode: String,
    pub at: String,
    pub by: String,
    pub base_tree_hash: Option<HashView>,
    pub composed_hash: Option<HashView>,
    pub declared_changed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PlaneCardView {
    pub id: &'static str,
    pub label: &'static str,
    pub source_file: &'static str,
    pub projection: &'static str,
    pub declared_hash: Option<HashView>,
    pub declared_count: usize,
    pub in_db: usize,
    pub kpis: Vec<DriftKpi>,
    pub actions: ActionCounts,
    pub rows: Vec<DriftRow>,
    pub row_count: usize,
    pub is_clean: bool,
    pub unreadable: Option<String>,
    pub applied: Option<AppliedView>,
    pub export_url: String,
    pub entity_filter: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct SyncPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub can_write: bool,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub docs_url: &'static str,
    pub access_control_url: &'static str,
    pub sources: SourcesView,
    pub composed_hash: Option<HashView>,
    pub last_reconciled_hash: Option<HashView>,
    pub base_tree_hash: Option<HashView>,
    pub planes: Vec<PlaneCardView>,
    pub entity_filter: String,
    pub sources_unreadable: Option<String>,
}
