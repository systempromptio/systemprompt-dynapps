//! `/admin/sync` — Code sync: where declarations come from and how they
//! move between the repository and this instance.
//!
//! Two tabs. *Sources* lists every place a declaration comes from in one
//! table — the internal base tree this repository ships and each external
//! bundle the profile pins — with the hash each declares and what it owns,
//! and offers the in-place import that recomposes them. *Export & import*
//! moves the projected planes as one archive: the database rendered back
//! to its files for a commit, or an uploaded archive staged for a preview
//! on `/admin/sync/import/{stage}`. What each plane's database says
//! against its file lives on the plane's owner page, under its Sync tab.
//!
//! A marketplace participant gets the scoped layout: the bundles that own
//! their marketplaces, one access-control card per marketplace with the
//! apply narrowed to it, the source refresh, and the sync trail filtered to
//! what touched their marketplaces. The archive and the whole-plane writes
//! stay with administrators.

mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::MarketplaceId;

use self::view::{ExportPlaneView, MarketplaceSyncView, SyncHistoryView, SyncPageData};
use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::sync_plane::{HashView, PlaneCardOpts, plane_card};
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};
use crate::repositories::sync::access_control::{AccessControlPlane, PLANE_ID};
use crate::repositories::sync::archive::staging::STAGE_TTL_MINUTES;
use crate::repositories::sync::history::{SyncHistoryRow, SyncHistoryScope, list_sync_history};
use crate::repositories::sync::plane::DeclarationSource;
use crate::repositories::sync::registry::planes;
use crate::repositories::sync::sources::{SourcesView, build_sources};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

pub(crate) const DOCS_URL: &str = "/documentation/services-sync";
pub(crate) const BASE_URL: &str = "/admin/sync";
pub(crate) const EXPORT_ZIP_URL: &str = "/api/public/admin/sync/export.zip";
pub(crate) const IMPORT_URL: &str = "/api/public/admin/sync/import";
const HISTORY_LIMIT: i64 = 50;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SyncQuery {
    pub tab: Option<String>,
}

fn tabs(show_sources: bool) -> Vec<TabLinkView> {
    vec![
        TabLinkView {
            slug: "sources",
            label: "Sources",
            href: BASE_URL.to_owned(),
            is_active: show_sources,
            count: None,
        },
        TabLinkView {
            slug: "archive",
            label: "Export & import",
            href: format!("{BASE_URL}?tab=archive"),
            is_active: !show_sources,
            count: None,
        },
    ]
}

// Why: an unreadable profile or cache is the panel's headline, not a
// 500 — the rest of the page still renders. A participant sees the bundles
// that own their marketplaces and nothing of the rest of the estate's kits.
fn visible_sources(user_ctx: &UserContext) -> (SourcesView, Option<String>) {
    let (mut sources, unreadable) = match build_sources() {
        Ok(s) => (s, None),
        Err(e) => (empty_sources(), Some(e.to_string())),
    };
    if !user_ctx.is_console {
        sources.bundles.retain(|b| {
            b.owns
                .marketplaces
                .iter()
                .any(|m| user_ctx.participation.includes_marketplace(m))
        });
    }
    (sources, unreadable)
}

async fn scoped_cards(
    pool: &PgPool,
    user_ctx: &UserContext,
) -> AdminResult<(Vec<MarketplaceSyncView>, Vec<SyncHistoryRow>)> {
    if user_ctx.is_console {
        return Ok((
            Vec::new(),
            list_sync_history(pool, SyncHistoryScope::All, HISTORY_LIMIT).await?,
        ));
    }
    let ids = &user_ctx.participation.marketplaces;
    Ok((
        marketplace_cards(pool, ids).await?,
        list_sync_history(pool, SyncHistoryScope::Marketplaces(ids), HISTORY_LIMIT).await?,
    ))
}

pub(crate) async fn sync_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SyncQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.has_scoped_console() {
        return Err(AdminError::Forbidden(
            "Console access or marketplace participation required.".to_owned(),
        )
        .into());
    }
    let is_participant = !user_ctx.is_console;
    let (sources, sources_unreadable) = visible_sources(&user_ctx);
    let (marketplaces, history) = scoped_cards(&pool, &user_ctx).await?;
    // Why: the archive tab is an administrator's; a participant's page has
    // one layout and no tabs.
    let show_sources = is_participant || query.tab.as_deref() != Some("archive");
    let export_planes = planes()
        .iter()
        .map(|p| ExportPlaneView {
            id: p.id(),
            label: p.label(),
            source_file: p.source_file(),
            owner_url: p.owner_url(),
        })
        .collect();

    let page = SyncPageData {
        page: "sync",
        title: "Code sync",
        can_write: user_ctx.is_admin,
        can_refresh: true,
        is_participant,
        marketplaces,
        history: history.into_iter().map(SyncHistoryView::from).collect(),
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Platform", "/admin/configuration"),
            BreadcrumbView::current("Code sync"),
        ],
        tabs: if is_participant {
            Vec::new()
        } else {
            tabs(show_sources)
        },
        show_sources,
        docs_url: DOCS_URL,
        configuration_url: "/admin/configuration",
        observability_url: crate::handlers::ssr::ssr_observability::PAGE_URL,
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
        sources_unreadable,
        export_planes,
        stage_ttl_minutes: STAGE_TTL_MINUTES,
    };
    Ok(super::render_typed_page(
        &engine, "sync", &page, &user_ctx, &mkt_ctx,
    ))
}

// Why: one access-control card per marketplace, narrowed to that entity so
// the diff table and the apply both speak of one marketplace.
async fn marketplace_cards(
    pool: &PgPool,
    marketplace_ids: &[MarketplaceId],
) -> AdminResult<Vec<MarketplaceSyncView>> {
    let plane = AccessControlPlane;
    let mut out = Vec::with_capacity(marketplace_ids.len());
    for id in marketplace_ids {
        let entity_filter = format!("marketplace/{id}");
        let opts = PlaneCardOpts {
            entity_filter: &entity_filter,
            apply_url: Some(format!("/api/public/admin/sync/planes/{PLANE_ID}/apply")),
            show_export: false,
            source_label: None,
        };
        let card = plane_card(pool, &plane, DeclarationSource::Disk, &opts).await?;
        out.push(MarketplaceSyncView {
            marketplace_id: id.clone(),
            versions_url: format!(
                "/admin/analysis/versions/{}",
                urlencoding::encode(id.as_str())
            ),
            card,
        });
    }
    Ok(out)
}

pub(crate) const fn empty_sources() -> SourcesView {
    SourcesView {
        base: crate::repositories::sync::sources::BaseSourceView {
            version: env!("CARGO_PKG_VERSION"),
            tree_hash: None,
            tree_path: String::new(),
            provenance: "unknown",
            provenance_error: None,
        },
        bundles: Vec::new(),
        composed_hash: None,
        last_reconciled_hash: None,
        restart_pending: false,
        on_fetch_failure: String::new(),
        refresh_hint: String::new(),
    }
}
