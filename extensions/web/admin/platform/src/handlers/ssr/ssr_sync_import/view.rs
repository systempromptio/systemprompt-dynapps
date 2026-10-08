//! View types and the page context for `/admin/sync/import/{stage}` — the
//! staged-archive preview.

use serde::Serialize;

use crate::handlers::ssr::ssr_sync::{BASE_URL as SYNC_URL, DOCS_URL};
use systemprompt_web_admin_core::repositories::sync::archive::manifest::ArchiveManifest;
use systemprompt_web_admin_core::repositories::sync::archive::staging::{
    OtherEntry, StagedArchive,
};
use systemprompt_web_admin_core::ssr::sync_plane::{HashView, PlaneCardView};
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ManifestView {
    pub release: String,
    pub exported_at: String,
    pub exported_by: String,
    pub base_tree_hash: Option<HashView>,
    pub composed_hash: Option<HashView>,
    pub planes: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AbsentPlaneView {
    pub label: &'static str,
    pub source_file: &'static str,
    pub owner_url: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PreviewKpiView {
    pub label: &'static str,
    pub value: usize,
    pub note: &'static str,
    pub tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct ImportPreviewPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub can_write: bool,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub stage_id: String,
    pub staged_by: String,
    pub staged_at: String,
    pub expires_at: String,
    pub manifest: Option<ManifestView>,
    pub manifest_error: Option<String>,
    pub kpis: Vec<PreviewKpiView>,
    pub planes: Vec<PlaneCardView>,
    pub absent: Vec<AbsentPlaneView>,
    pub other: Vec<OtherEntry>,
    // Why: handlebars-rust has no `.length`, so the section takes the count as a field.
    pub other_count: usize,
    pub apply_all_url: String,
    pub discard_url: String,
    pub sync_url: &'static str,
    pub docs_url: &'static str,
}

fn kpis(cards: &[PlaneCardView], other: usize) -> Vec<PreviewKpiView> {
    let clean = cards.iter().filter(|c| c.is_clean).count();
    vec![
        PreviewKpiView {
            label: "Planes in the archive",
            value: cards.len(),
            note: "each previewed against this database",
            tone: "info",
        },
        PreviewKpiView {
            label: "Already in step",
            value: clean,
            note: "an apply would move nothing",
            tone: "ok",
        },
        PreviewKpiView {
            label: "Would change",
            value: cards.len() - clean,
            note: "choose a direction per plane",
            tone: if cards.len() > clean { "warn" } else { "ok" },
        },
        PreviewKpiView {
            label: "Served from code",
            value: other,
            note: "not applied here; commit or publish",
            tone: "muted",
        },
    ]
}

pub(super) fn apply_url(stage_id: &str) -> String {
    format!("/api/public/admin/sync/import/{stage_id}/apply")
}

fn manifest_view(m: &ArchiveManifest) -> ManifestView {
    ManifestView {
        release: m.release.clone(),
        exported_at: m.exported_at.to_rfc3339(),
        exported_by: m.exported_by.clone(),
        base_tree_hash: m.base_tree_hash.as_deref().and_then(HashView::of),
        composed_hash: m.composed_hash.as_deref().and_then(HashView::of),
        planes: m.planes.len(),
    }
}

pub(super) struct PreviewPlanes {
    pub cards: Vec<PlaneCardView>,
    pub absent: Vec<AbsentPlaneView>,
}

pub(super) fn page_data(
    stage: &StagedArchive,
    planes: PreviewPlanes,
    can_write: bool,
) -> ImportPreviewPageData {
    ImportPreviewPageData {
        page: "sync",
        title: "Import preview",
        can_write,
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Code sync", SYNC_URL),
            BreadcrumbView::current("Import preview"),
        ],
        stage_id: stage.id.clone(),
        staged_by: stage.actor.clone(),
        staged_at: stage.created_at.to_rfc3339(),
        expires_at: stage.expires_at().to_rfc3339(),
        manifest: stage.manifest.as_ref().map(manifest_view),
        manifest_error: stage.manifest_error.clone(),
        kpis: kpis(&planes.cards, stage.other.len()),
        planes: planes.cards,
        absent: planes.absent,
        other_count: stage.other.len(),
        other: stage.other.clone(),
        apply_all_url: apply_url(&stage.id),
        discard_url: format!("/api/public/admin/sync/import/{}", stage.id),
        sync_url: SYNC_URL,
        docs_url: DOCS_URL,
    }
}
