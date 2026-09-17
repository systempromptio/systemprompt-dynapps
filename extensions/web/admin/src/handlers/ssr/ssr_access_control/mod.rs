//! `/admin/access-control` — every governed entity, who reaches it and why,
//! and whether the code agrees with the database.
//!
//! Two tabs. *Rules*: a short account of how a decision is made (the
//! ladder), the entities table, and the audience grid resolved by the real
//! resolver. *Sync*: the `access_control` plane — `rules.yaml` against this
//! database, with the three directions. Editing lives with the subject — a
//! group's band on its Access tab, a person's overrides on theirs. Reading
//! here is the CONSOLE tier so a project manager can audit.

mod audience;
mod bands;
mod data;
pub(crate) mod entities;
mod summary;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::access_control::build_matrix_sections;
use crate::handlers::shared;
use crate::handlers::ssr::sync_plane::{is_sync_tab, plane_card_by_id, sync_tab};
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};
use crate::repositories::access_control::rules::{
    RULE_CAP, count_open_entities, list_ledger_rules,
};
use crate::repositories::sync::access_control::{declared_now, drift_now};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use entities::{AcQuery, BASE_URL, DOCS_URL, EntitiesInput, SYNC_URL};
use view::{AccessControlPageData, AudienceGridView, DriftBannerView};

fn tabs(on_sync: bool) -> Vec<TabLinkView> {
    vec![
        TabLinkView {
            slug: "rules",
            label: "Rules",
            href: BASE_URL.to_owned(),
            is_active: !on_sync,
            count: None,
        },
        sync_tab(BASE_URL, on_sync),
    ]
}

pub(crate) async fn access_control_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AcQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    if let Some(user) = query.user.as_deref().filter(|u| !u.is_empty()) {
        let target = format!("/admin/users/{}?tab=access", urlencoding::encode(user));
        return Ok(Redirect::permanent(&target).into_response());
    }
    let services_path = shared::get_services_path()?;
    let on_sync = is_sync_tab(query.tab.as_deref());

    let stats = data::load_stats(&pool).await;
    let ledger_rows = list_ledger_rules(&pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access-control: rule listing failed"))
        .unwrap_or_default();
    let open_entities = count_open_entities(&pool).await?;

    // Why: the file failing to load is reported on the page, not as a 500.
    // The rules that ARE enforced are still worth showing while the operator
    // fixes the declaration.
    let (drift, declared_count, declared_unreadable) = match declared_now().await {
        Ok(declared) => match drift_now(&pool, &declared).await {
            Ok(drift) => (Some(drift), declared.rule_count(), None),
            Err(e) => (None, declared.rule_count(), Some(e.to_string())),
        },
        Err(e) => (None, 0, Some(e.to_string())),
    };

    // Why: the Sync tab reads the plane, not the ledger — the audience grid
    // runs the real resolver per entity and is not paid for a tab that
    // never shows it.
    let (grid, audiences, sync) = if on_sync {
        let filter = query.entity.clone().unwrap_or_default();
        let card = plane_card_by_id(
            &pool,
            crate::repositories::sync::access_control::PLANE_ID,
            &filter,
        )
        .await?;
        (AudienceGridView::default(), 0, card)
    } else {
        let audience = audience::audience(&pool).await;
        let audiences = audience.columns.len();
        let sections = build_matrix_sections(&services_path);
        (
            audience::grid(&pool, audience, &sections).await,
            audiences,
            None,
        )
    };

    let capped = i64::try_from(ledger_rows.len()).unwrap_or(i64::MAX) >= RULE_CAP;
    let entities = entities::build(
        &EntitiesInput {
            rows: &ledger_rows,
            drift: drift.as_ref(),
            open_entities,
            audiences,
            capped,
        },
        &query,
    );

    let banner = drift
        .as_ref()
        .filter(|d| !d.is_clean())
        .map(|d| DriftBannerView {
            counts: d.counts(),
            url: SYNC_URL,
            declared: declared_count,
        });

    let page = AccessControlPageData {
        page: "access-control",
        title: "Access control",
        can_write: user_ctx.is_admin,
        stats,
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("People & access", "/admin/users"),
            BreadcrumbView::current("Access control"),
        ],
        tabs: tabs(on_sync),
        sync,
        docs_url: DOCS_URL,
        sync_url: SYNC_URL,
        drift: banner,
        declared_unreadable,
        entities,
        audience: grid,
    };

    Ok(super::render_typed_page(
        &engine,
        "access-control",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}
