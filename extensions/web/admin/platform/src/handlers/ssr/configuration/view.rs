//! View types for `/admin/configuration`.

use serde::Serialize;

use super::secrets::SecretsCardView;
use systemprompt_web_admin_core::ssr::retention::RetentionView;
use systemprompt_web_admin_core::ssr::sync_plane::{AppliedView, HashView};
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ConfigRowView {
    pub id: &'static str,
    pub label: &'static str,
    pub purpose: &'static str,
    pub path: String,
    pub is_dir: bool,
    pub files: usize,
    pub source: String,
    pub source_tone: &'static str,
    pub hash: Option<HashView>,
    pub is_projected: bool,
    pub projection: Option<&'static str>,
    pub state: &'static str,
    pub state_tone: &'static str,
    pub state_note: String,
    pub applied: Option<AppliedView>,
    pub link: Option<String>,
    pub link_label: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ConfigKpiView {
    pub label: &'static str,
    pub value: usize,
    pub note: &'static str,
    pub tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConfigurationPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub can_write: bool,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub tabs: Vec<TabLinkView>,
    pub release: &'static str,
    pub base_tree_hash: Option<HashView>,
    pub composed_hash: Option<HashView>,
    pub provenance: &'static str,
    pub bundle_count: usize,
    pub kpis: Vec<ConfigKpiView>,
    pub rows: Vec<ConfigRowView>,
    pub retention: RetentionView,
    pub secrets: SecretsCardView,
    pub sync_url: &'static str,
    pub export_zip_url: &'static str,
    pub docs_url: &'static str,
}
