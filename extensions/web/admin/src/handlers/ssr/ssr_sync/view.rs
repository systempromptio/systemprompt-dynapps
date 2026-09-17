//! View types for `/admin/sync` — Code sync.

use serde::Serialize;

use crate::handlers::ssr::sync_plane::HashView;
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};
use crate::repositories::sync::sources::SourcesView;

// Why: one plane as the export card lists it.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ExportPlaneView {
    pub id: &'static str,
    pub label: &'static str,
    pub source_file: &'static str,
    pub owner_url: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct SyncPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub can_write: bool,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub tabs: Vec<TabLinkView>,
    pub show_sources: bool,
    pub docs_url: &'static str,
    pub configuration_url: &'static str,
    pub observability_url: &'static str,
    pub export_zip_url: &'static str,
    pub kit_patch_url: &'static str,
    pub import_url: &'static str,
    pub sources: SourcesView,
    pub composed_hash: Option<HashView>,
    pub last_reconciled_hash: Option<HashView>,
    pub base_tree_hash: Option<HashView>,
    pub sources_unreadable: Option<String>,
    pub export_planes: Vec<ExportPlaneView>,
    pub stage_ttl_minutes: i64,
}
