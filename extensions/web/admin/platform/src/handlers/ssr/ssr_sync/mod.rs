//! `/admin/sync` — Code sync: where declarations come from and how they
//! move between the repository and this instance.
//!
//! Three tabs. *Sources* lists every place a declaration comes from in one
//! table — the internal base tree this repository ships and each external
//! bundle the profile pins — with the hash each declares and what it owns,
//! and offers the in-place import that recomposes them, above one row per
//! projected plane linking to its sync location. *Access review* is
//! the access-control plane, entity by entity: what code and this database
//! disagree about, settled one row at a time with a stated reason, above the
//! whole-plane card. *Export & import*
//! moves the projected planes as one archive: the database rendered back
//! to its files for a commit, or an uploaded archive staged for a preview
//! on `/admin/sync/import/{stage}`. What each plane's database says
//! against its file lives at the plane's `sync_url` — its owner page's Sync
//! tab, or Access review for access control.
//!
//! A marketplace participant gets the scoped layout: the bundles that own
//! their marketplaces, one access-control card per marketplace with the
//! apply narrowed to it, the source refresh, and the sync trail filtered to
//! what touched their marketplaces. The archive and the whole-plane writes
//! stay with administrators, and so do the review and archive tabs.
//!
//! An unreadable profile or cache is the Sources panel's headline, not a
//! 500 — the rest of the page still renders.

mod access_review;
mod planes;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::MarketplaceId;

use self::view::MarketplaceSyncView;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult, AdminResult};
use systemprompt_web_admin_core::repositories::sync::core_planes::CorePlanes;
use systemprompt_web_admin_core::repositories::sync::history::{
    SyncHistoryRow, SyncHistoryScope, list_sync_history,
};
use systemprompt_web_admin_core::repositories::sync::plane_ids::ACCESS_CONTROL;
use systemprompt_web_admin_core::repositories::sync::sources::{SourcesView, build_sources};
use systemprompt_web_admin_core::ssr::core_plane::{CorePlaneCardOpts, core_plane_card};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;
use systemprompt_web_admin_core::types::UserContext;

pub(crate) const DOCS_URL: &str = "/documentation/services-sync";
pub(crate) const BASE_URL: &str = "/admin/sync";
pub(crate) const EXPORT_ZIP_URL: &str = "/api/public/admin/sync/export.zip";
pub(crate) const IMPORT_URL: &str = "/api/public/admin/sync/import";
const HISTORY_LIMIT: i64 = 50;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SyncQuery {
    pub tab: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncTab {
    Sources,
    Access,
    Archive,
}

impl SyncTab {
    fn parse(tab: Option<&str>) -> Self {
        match tab {
            Some("access") => Self::Access,
            Some("archive") => Self::Archive,
            _ => Self::Sources,
        }
    }

    fn for_viewer(is_participant: bool, tab: Option<&str>) -> Self {
        if is_participant {
            Self::Sources
        } else {
            Self::parse(tab)
        }
    }
}

fn tabs(active: SyncTab, attention: usize) -> Vec<TabLinkView> {
    vec![
        TabLinkView {
            slug: "sources",
            label: "Sources",
            href: BASE_URL.to_owned(),
            is_active: active == SyncTab::Sources,
            count: None,
        },
        TabLinkView {
            slug: "access",
            label: "Access review",
            href: format!("{BASE_URL}?tab=access"),
            is_active: active == SyncTab::Access,
            count: i64::try_from(attention).ok().filter(|n| *n > 0),
        },
        TabLinkView {
            slug: "archive",
            label: "Export & import",
            href: format!("{BASE_URL}?tab=archive"),
            is_active: active == SyncTab::Archive,
            count: None,
        },
    ]
}

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
    core_planes: &CorePlanes,
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
        marketplace_cards(pool, core_planes, ids).await?,
        list_sync_history(pool, SyncHistoryScope::Marketplaces(ids), HISTORY_LIMIT).await?,
    ))
}

pub(crate) async fn sync_page(
    Extension(services): Extension<
        systemprompt_web_admin_core::extension_services::ExtensionServices,
    >,
    page: Page,
    Extension(core_planes): Extension<CorePlanes>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SyncQuery>,
) -> AdminHtmlResult<Response> {
    let Page {
        engine,
        user: user_ctx,
        marketplace: mkt_ctx,
    } = page;
    if !user_ctx.has_scoped_console() {
        return Err(AdminError::Forbidden(
            "Console access or marketplace participation required.".to_owned(),
        )
        .into());
    }
    let is_participant = !user_ctx.is_console;
    let (sources, sources_unreadable) = visible_sources(&user_ctx);
    let (marketplaces, history) = scoped_cards(&pool, &core_planes, &user_ctx).await?;
    let tab = SyncTab::for_viewer(is_participant, query.tab.as_deref());
    let access_review = if tab == SyncTab::Access {
        Some(access_review::access_review(&services, &pool, &core_planes).await?)
    } else {
        None
    };

    let planes = if tab == SyncTab::Sources && user_ctx.is_console {
        planes::plane_rows(&pool, &core_planes).await?
    } else {
        Vec::new()
    };

    let page = view::page_data(view::SyncPageInput {
        tab,
        is_participant,
        can_write: user_ctx.is_admin,
        marketplaces,
        history,
        access_review,
        planes,
        sources,
        sources_unreadable,
    });
    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_typed_page(
            &engine, "sync", &page, &user_ctx, &mkt_ctx,
        ),
    )
}

async fn marketplace_cards(
    pool: &PgPool,
    core_planes: &CorePlanes,
    marketplace_ids: &[MarketplaceId],
) -> AdminResult<Vec<MarketplaceSyncView>> {
    let mut out = Vec::with_capacity(marketplace_ids.len());
    for id in marketplace_ids {
        let entity_filter = format!("marketplace/{id}");
        let opts = CorePlaneCardOpts {
            entity_filter: &entity_filter,
            show_export: false,
        };
        let Some(card) = core_plane_card(pool, core_planes, ACCESS_CONTROL, &opts).await? else {
            continue;
        };
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
        base: systemprompt_web_admin_core::repositories::sync::sources::BaseSourceView {
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
