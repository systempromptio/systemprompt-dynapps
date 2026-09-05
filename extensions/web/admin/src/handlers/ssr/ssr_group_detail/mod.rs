//! `/admin/access/groups/{group_id}` — one group across five tabs, and the
//! Unassigned bucket as a variant of the same page.
//!
//! Unassigned is not a group anyone created: it is everyone with no group row
//! at all, so it has no rules and no directory mapping of its own. The page
//! renders it with a callout and an assign control instead of the stat row and
//! the Access and Mappings tabs, because those would edit a subject that can
//! never hold a rule.

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::people_chart::daily_requests_chart;
use super::people_view;
use super::ssr_groups::UNASSIGNED_GROUP;
use super::types::{
    BreadcrumbView, GroupDetailPageData, GroupOverviewView, MemberSetChipView, MembersTabView,
    StatTileView,
};

mod data;
mod tabs;
mod view;

#[derive(Debug, Deserialize)]
pub(crate) struct TabQuery {
    tab: Option<String>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "axum extractor list; the router decides the arity, not this signature"
)]
pub(crate) async fn group_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(group_id): Path<String>,
    Query(query): Query<TabQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let group = repositories::groups::crud::find_group(&pool, &group_id).await?;
    let is_unassigned = group_id == UNASSIGNED_GROUP;
    let page = if is_unassigned {
        "unassigned"
    } else {
        "group-detail"
    };

    let Some(group) = group else {
        let data = not_found(page, &group_id);
        return Ok(super::render_typed_page(
            &engine,
            "group-detail",
            &data,
            &user_ctx,
            &mkt_ctx,
        ));
    };

    let active = tabs::resolve_tab(query.tab.as_deref(), is_unassigned);
    let counts = data::load_counts(&pool, &group_id).await;
    let usage = data::load_usage(&pool, &group_id).await;

    let data = GroupDetailPageData {
        page,
        title: group.name.clone(),
        crumbs: crumbs(&group.name),
        tabs: tabs::tab_links(&group_id, active, is_unassigned, &counts),
        stats: if is_unassigned {
            Vec::new()
        } else {
            people_view::stat_tiles(&usage, counts.members)
        },
        overview: load_overview(&pool, &group_id, active).await,
        members: load_members(&pool, &group_id, active, &user_ctx).await,
        access: load_access(&pool, &group_id, active).await,
        projects: load_projects(&pool, &group_id, active).await,
        mappings: load_mappings(&pool, &group_id, active, &user_ctx).await,
        active_tab: active.to_owned(),
        group_name: group.name,
        description: group.description,
        is_unassigned,
        not_found: false,
        can_manage: user_ctx.is_admin,
        can_map: user_ctx.is_platform_admin,
        group_id,
    };

    Ok(super::render_typed_page(
        &engine,
        "group-detail",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}

async fn load_overview(pool: &PgPool, group_id: &str, active: &str) -> Option<GroupOverviewView> {
    if active != tabs::OVERVIEW {
        return None;
    }
    let loaded = data::load_overview(pool, group_id).await;
    Some(GroupOverviewView {
        models: people_view::model_rows(&loaded.models),
        daily_requests: daily_requests_chart(&loaded.daily),
        skills: people_view::name_count_rows(&loaded.skills),
        tools: people_view::name_count_rows(&loaded.tools),
    })
}

async fn load_members(
    pool: &PgPool,
    group_id: &str,
    active: &str,
    user_ctx: &UserContext,
) -> Option<MembersTabView> {
    if active != tabs::MEMBERS {
        return None;
    }
    let loaded = data::load_members(pool, group_id, user_ctx.is_admin).await;
    let assign_targets = if group_id == UNASSIGNED_GROUP && user_ctx.is_admin {
        assign_targets(pool).await
    } else {
        Vec::new()
    };
    Some(MembersTabView {
        rows: view::group_member_rows(&loaded, user_ctx.is_admin),
        addable_users: loaded.addable,
        assign_targets,
    })
}

// Why: Destinations for the Unassigned page's assign control: every real group.
async fn assign_targets(pool: &PgPool) -> Vec<MemberSetChipView> {
    repositories::groups::crud::list_groups(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "assign target list failed"))
        .unwrap_or_default()
        .into_iter()
        .filter(|g| g.id != UNASSIGNED_GROUP)
        .map(|g| MemberSetChipView {
            id: g.id,
            label: g.name,
        })
        .collect()
}

async fn load_access(
    pool: &PgPool,
    group_id: &str,
    active: &str,
) -> Option<Vec<super::types::AccessSectionView>> {
    if active != tabs::ACCESS {
        return None;
    }
    Some(data::load_access(pool, group_id).await)
}

async fn load_projects(
    pool: &PgPool,
    group_id: &str,
    active: &str,
) -> Option<Vec<super::types::ProjectRowView>> {
    if active != tabs::PROJECTS {
        return None;
    }
    let rows = data::load_projects(pool, group_id).await;
    Some(people_view::linked_rows(&rows, "projects"))
}

async fn load_mappings(
    pool: &PgPool,
    group_id: &str,
    active: &str,
    user_ctx: &UserContext,
) -> Option<Vec<super::types::MappingRowView>> {
    if active != tabs::MAPPINGS {
        return None;
    }
    let rows = data::load_mappings(pool, group_id).await;
    Some(people_view::mapping_rows(
        rows.into_iter().map(|r| (r.ad_group, r.source)),
        user_ctx.is_platform_admin,
    ))
}

fn crumbs(name: &str) -> Vec<BreadcrumbView> {
    vec![
        BreadcrumbView {
            label: "Groups".to_owned(),
            href: Some("/admin/access/groups".to_owned()),
        },
        BreadcrumbView {
            label: name.to_owned(),
            href: None,
        },
    ]
}

fn not_found(page: &'static str, group_id: &str) -> GroupDetailPageData {
    GroupDetailPageData {
        page,
        title: "Group not found".to_owned(),
        group_id: group_id.to_owned(),
        group_name: group_id.to_owned(),
        description: None,
        is_unassigned: false,
        not_found: true,
        can_manage: false,
        can_map: false,
        crumbs: crumbs(group_id),
        tabs: Vec::new(),
        active_tab: String::new(),
        stats: Vec::<StatTileView>::new(),
        overview: None,
        members: None,
        access: None,
        projects: None,
        mappings: None,
    }
}
