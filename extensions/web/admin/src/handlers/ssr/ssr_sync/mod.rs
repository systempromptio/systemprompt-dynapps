//! `/admin/sync` — Code ↔ Instance: every source a declaration comes from,
//! every plane it projects into, and the difference between the two.
//!
//! Sources first (the base tree and each pinned bundle, with hashes and
//! provenance), then one card per registered plane: declared hash, what was
//! last applied and by whom, the drift totals, the three directions, and the
//! drift table filterable by entity. The actions are the JSON API in
//! [`crate::handlers::sync`]; this page only reads.

mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use self::view::{AppliedView, HashView, PlaneCardView, SyncPageData};
use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::handlers::sync::plane_status;
use crate::repositories::sync::registry::planes;
use crate::repositories::sync::sources::{SourcesView, build_sources};
use crate::repositories::sync::state::SyncStateRow;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

pub(crate) const DOCS_URL: &str = "/documentation/services-sync";

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SyncQuery {
    // Why: `<kind>/<id>` narrows every plane's table to one entity — the
    // access-control ledger's deep link.
    pub entity: Option<String>,
}

fn applied_view(state: Option<&SyncStateRow>, declared_hash: &str) -> Option<AppliedView> {
    let s = state?;
    let hash = s.applied_hash.as_deref()?;
    Some(AppliedView {
        hash: HashView::of(hash),
        mode: s.applied_mode.clone().unwrap_or_default().replace('_', " "),
        at: s.applied_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
        by: s.applied_by.clone().unwrap_or_default(),
        base_tree_hash: s.base_tree_hash.as_deref().and_then(HashView::of),
        composed_hash: s.composed_hash.as_deref().and_then(HashView::of),
        declared_changed: !declared_hash.is_empty() && declared_hash != hash,
    })
}

pub(crate) async fn sync_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SyncQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    // Why: an unreadable profile or cache is the panel's headline, not a
    // 500 — the planes below still render.
    let (sources, sources_unreadable) = match build_sources() {
        Ok(s) => (s, None),
        Err(e) => (empty_sources(), Some(e.to_string())),
    };
    let filter = query.entity.unwrap_or_default();

    let mut cards = Vec::new();
    for plane in planes() {
        let status = plane_status(&pool, plane.as_ref()).await?;
        let mut rows = status.drift.rows;
        if let Some((kind, id)) = filter.split_once('/') {
            rows.retain(|r| r.entity_type == kind && r.entity_id == id);
        }
        cards.push(PlaneCardView {
            id: status.id,
            label: status.label,
            source_file: status.source_file,
            projection: status.projection,
            declared_hash: HashView::of(&status.drift.declared_hash),
            declared_count: status.drift.declared_count,
            in_db: status.drift.in_db,
            kpis: status.drift.kpis,
            actions: status.drift.actions,
            row_count: rows.len(),
            rows,
            is_clean: status.drift.is_clean && status.drift.unreadable.is_none(),
            unreadable: status.drift.unreadable,
            applied: applied_view(status.state.as_ref(), &status.drift.declared_hash),
            export_url: format!("/api/public/admin/sync/planes/{}/export", status.id),
            entity_filter: filter.clone(),
        });
    }

    let page = SyncPageData {
        page: "sync",
        title: "Code ↔ Instance",
        can_write: user_ctx.is_admin,
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::current("Sync"),
        ],
        docs_url: DOCS_URL,
        access_control_url: "/admin/access-control",
        composed_hash: sources.composed_hash.as_deref().and_then(HashView::of),
        last_reconciled_hash: sources
            .last_reconciled_hash
            .as_deref()
            .and_then(HashView::of),
        base_tree_hash: sources.base.tree_hash.as_deref().and_then(HashView::of),
        sources,
        planes: cards,
        entity_filter: filter,
        sources_unreadable,
    };
    Ok(super::render_typed_page(
        &engine, "sync", &page, &user_ctx, &mkt_ctx,
    ))
}

const fn empty_sources() -> SourcesView {
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
