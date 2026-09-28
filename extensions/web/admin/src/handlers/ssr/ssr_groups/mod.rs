//! `/admin/groups` — every group, with its membership, entitlement and spend.
//!
//! `unassigned` is a row here like any other. It is derived rather than
//! stored — anyone with no `group_members` row falls into it — and the whole
//! point of the page is that the number next to it is visible rather than
//! implicit, so it is listed and badged, never filtered out.
//!
//! The spend columns are exclusively attributed, so the rows plus the
//! unattributed row at the foot of the table reproduce the instance total.
//! That is the property the page is built around: a group listing whose
//! numbers do not add up to anything cannot answer "where did the money go".
//!
//! `unassigned` is never offered as a mapping destination: it is where people
//! land when no mapping matched, so mapping a directory group into it would be
//! a contradiction.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::sync_plane::{is_sync_tab, plane_card_by_id, sync_tab};
use super::types::{MemberSetChipView, TabLinkView};

mod data;
mod sorting;
mod view;

pub(crate) const UNASSIGNED_GROUP: &str = "unassigned";

#[derive(Debug, Deserialize)]
pub(crate) struct GroupsQuery {
    range: Option<String>,
    sort: Option<String>,
    dir: Option<String>,
    page: Option<i64>,
    source: Option<String>,
    tab: Option<String>,
}

fn tabs(on_sync: bool) -> Vec<TabLinkView> {
    vec![
        TabLinkView {
            slug: "groups",
            label: "Groups",
            href: sorting::BASE_URL.to_owned(),
            is_active: !on_sync,
            count: None,
        },
        sync_tab(sorting::BASE_URL, on_sync),
    ]
}

pub(crate) async fn groups_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<GroupsQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let window_days = sorting::window_days(query.range.as_deref());
    let range = query.range.as_deref().unwrap_or("30d").to_owned();
    let sort = sorting::sort_key(query.sort.as_deref());
    let dir = sorting::direction(query.dir.as_deref());

    let on_sync = is_sync_tab(query.tab.as_deref());
    let sync = if on_sync {
        plane_card_by_id(&pool, repositories::sync::groups::PLANE_ID, "").await?
    } else {
        None
    };

    let source = sorting::source_filter(query.source.as_deref());
    let mut listing = data::load_listing(&pool, window_days).await;
    sorting::apply(&mut listing.rows, sort, dir);

    let data = view::page_data(view::GroupsPageInput {
        listing,
        window_days,
        facets: view::ListingFacets {
            range: &range,
            sort,
            dir,
            source: &source,
        },
        page: query.page.unwrap_or(1),
        sync,
        on_sync,
        can_manage: user_ctx.is_admin,
        can_map: user_ctx.is_platform_admin,
        group_options: group_options(&pool).await,
    });

    Ok(super::render_typed_page(
        &engine, "groups", &data, &user_ctx, &mkt_ctx,
    ))
}

async fn group_options(pool: &PgPool) -> Vec<MemberSetChipView> {
    repositories::groups::crud::list_groups(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "group option list failed"))
        .unwrap_or_default()
        .into_iter()
        .filter(|g| g.id != UNASSIGNED_GROUP)
        .map(|g| MemberSetChipView {
            id: g.id.as_str().to_owned(),
            label: g.name,
        })
        .collect()
}
