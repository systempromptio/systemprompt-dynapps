//! View types and the page context for `/admin/sync` — Code sync.
//!
//! `can_write` is the administrator's whole-plane and archive authority;
//! `can_refresh` is the source refresh any participant may press;
//! `is_participant` switches the page to the scoped layout with no tabs.

use serde::Serialize;
use systemprompt::identifiers::MarketplaceId;
use systemprompt_web_admin_core::repositories::sync::core_planes::{
    CORE_PLANE_IDS, core_plane_meta,
};

use super::access_review::AccessReviewView;
use super::planes::PlaneRowView;
use super::{DOCS_URL, EXPORT_ZIP_URL, IMPORT_URL, SyncTab, tabs};
use systemprompt_web_admin_core::repositories::sync::archive::staging::STAGE_TTL_MINUTES;
use systemprompt_web_admin_core::repositories::sync::attention::access_attention;
use systemprompt_web_admin_core::repositories::sync::history::SyncHistoryRow;
use systemprompt_web_admin_core::repositories::sync::registry::planes;
use systemprompt_web_admin_core::repositories::sync::sources::SourcesView;
use systemprompt_web_admin_core::ssr::core_plane::CorePlaneCardView;
use systemprompt_web_admin_core::ssr::sync_plane::HashView;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SyncHistoryView {
    pub id: String,
    pub actor: String,
    pub actor_url: String,
    pub action: String,
    pub plane: String,
    pub description: String,
    pub marketplaces: Vec<String>,
    pub at: String,
}

impl From<SyncHistoryRow> for SyncHistoryView {
    fn from(row: SyncHistoryRow) -> Self {
        Self {
            id: row.id,
            actor: row.display_name,
            actor_url: format!("/admin/users/{}", urlencoding::encode(row.user_id.as_str())),
            action: row.action,
            plane: row.plane.unwrap_or_default(),
            description: row.description,
            marketplaces: row.marketplaces,
            at: row.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct MarketplaceSyncView {
    pub marketplace_id: MarketplaceId,
    pub versions_url: String,
    pub card: CorePlaneCardView,
}

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
    pub can_refresh: bool,
    pub is_participant: bool,
    pub marketplaces: Vec<MarketplaceSyncView>,
    pub history: Vec<SyncHistoryView>,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub tabs: Vec<TabLinkView>,
    pub show_sources: bool,
    pub show_access: bool,
    pub access_review: Option<AccessReviewView>,
    pub planes: Vec<PlaneRowView>,
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
    pub export_planes: Vec<ExportPlaneView>,
    pub stage_ttl_minutes: i64,
}

pub(super) struct SyncPageInput {
    pub tab: SyncTab,
    pub is_participant: bool,
    pub can_write: bool,
    pub marketplaces: Vec<MarketplaceSyncView>,
    pub history: Vec<SyncHistoryRow>,
    pub access_review: Option<AccessReviewView>,
    pub planes: Vec<PlaneRowView>,
    pub export_planes: Vec<ExportPlaneView>,
    pub sources: SourcesView,
}

pub(super) fn export_planes(
    boot: &systemprompt_web_admin_core::extension_boot::ExtensionBoot,
) -> Vec<ExportPlaneView> {
    let install = planes(boot).into_iter().map(|p| ExportPlaneView {
        id: p.id(),
        label: p.label(),
        source_file: p.source_file(),
        owner_url: p.owner_url(),
    });
    let core = CORE_PLANE_IDS
        .iter()
        .filter_map(|id| core_plane_meta(id))
        .map(|m| ExportPlaneView {
            id: m.id,
            label: m.label,
            source_file: m.source_file,
            owner_url: m.owner_url,
        });
    install.chain(core).collect()
}

pub(super) fn page_data(input: SyncPageInput) -> SyncPageData {
    let SyncPageInput {
        tab,
        sources,
        export_planes,
        ..
    } = input;
    SyncPageData {
        page: "sync",
        title: "Code sync",
        can_write: input.can_write,
        can_refresh: true,
        is_participant: input.is_participant,
        marketplaces: input.marketplaces,
        history: input
            .history
            .into_iter()
            .map(SyncHistoryView::from)
            .collect(),
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Platform", "/admin/configuration"),
            BreadcrumbView::current("Code sync"),
        ],
        tabs: if input.is_participant {
            Vec::new()
        } else {
            tabs(tab, access_attention())
        },
        show_sources: tab == SyncTab::Sources,
        show_access: tab == SyncTab::Access,
        access_review: input.access_review,
        planes: input.planes,
        docs_url: DOCS_URL,
        configuration_url: "/admin/configuration",
        observability_url: "/admin/system/observability",
        export_zip_url: EXPORT_ZIP_URL,
        kit_patch_url: "/api/public/admin/sync/export/kit",
        import_url: IMPORT_URL,
        composed_hash: sources.composed_hash.as_deref().and_then(HashView::of),
        last_reconciled_hash: sources
            .last_reconciled_hash
            .as_deref()
            .and_then(HashView::of),
        base_tree_hash: sources.base.tree_hash.as_deref().and_then(HashView::of),
        sources,
        export_planes,
        stage_ttl_minutes: STAGE_TTL_MINUTES,
    }
}
