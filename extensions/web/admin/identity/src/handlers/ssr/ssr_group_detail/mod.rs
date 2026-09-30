//! `/admin/groups/{group_id}` — one group across five tabs (its marketplaces
//! are the first section of Access), and the
//! Unassigned bucket as a variant of the same page.
//!
//! Unassigned is not a group anyone created: it is everyone with no group row
//! at all, so it has no rules and no directory mapping of its own. The page
//! renders it with a callout and an assign control instead of the stat row and
//! the Access and Mappings tabs, because those would edit a subject that can
//! never hold a rule.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt_web_shared::GroupId;

use crate::repositories;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::types::UserContext;

use super::people_chart::daily_requests_chart;
use super::people_view;
use super::ssr_groups::UNASSIGNED_GROUP;
use super::types::{
    GroupDetailPageData, GroupOverviewView, MemberSetChipView, MembersTabView, UsageLeaderRowView,
};
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::Viewer;
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

mod data;
mod tabs;
mod view;

#[derive(Debug, Deserialize)]
pub(crate) struct TabQuery {
    tab: Option<String>,
}

pub(crate) async fn group_detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(group_id): Path<GroupId>,
    Query(query): Query<TabQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    if query.tab.as_deref() == Some("marketplaces") {
        return Ok(Redirect::to(&format!("/admin/groups/{group_id}?tab=access")).into_response());
    }

    let group = repositories::groups::crud::find_group(&pool, &group_id).await?;
    let is_unassigned = group_id == UNASSIGNED_GROUP;
    let page = if is_unassigned {
        "unassigned"
    } else {
        "group-detail"
    };

    let Some(group) = group else {
        return Err(AdminError::NotFound("No such group.".to_owned()).into());
    };

    let active = tabs::resolve_tab(query.tab.as_deref(), is_unassigned);
    let mut faults = Faults::new();
    let counts = data::load_counts(&pool, &group_id, &mut faults).await;
    let usage = data::load_usage(&pool, &group_id, &mut faults).await;

    let data = GroupDetailPageData {
        page,
        title: group.name.clone(),
        breadcrumbs: breadcrumbs(&group.name),
        tabs: tabs::tab_links(&group_id, active, is_unassigned, &counts),
        stats: if is_unassigned {
            Vec::new()
        } else {
            people_view::stat_tiles(&usage, counts.members)
        },
        overview: load_overview(&pool, &group_id, active, &mut faults).await,
        members: load_members(&pool, &group_id, active, &shell.user, &mut faults).await,
        access: load_access(&pool, &group_id, active, &mut faults).await,
        projects: load_projects(&pool, &group_id, active, &mut faults).await,
        mappings: load_mappings(&pool, &group_id, active, &shell.user, &mut faults).await,
        active_tab: active.to_owned(),
        group_name: group.name,
        description: group.description,
        is_unassigned,
        not_found: false,
        can_manage: shell.user.is_admin,
        can_map: shell.user.is_platform_admin,
        export: (!is_unassigned).then(|| export_view(&group_id)),
        group_id,
    };

    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_page_with_faults(
            &shell.engine,
            "group-detail",
            &data,
            Viewer::new(&shell.user, &shell.marketplace),
            &faults,
        ),
    )
}

async fn load_overview(
    pool: &PgPool,
    group_id: &GroupId,
    active: &str,
    faults: &mut Faults,
) -> Option<GroupOverviewView> {
    if active != tabs::USAGE {
        return None;
    }
    let loaded = data::load_usage_tab(pool, group_id, faults).await;
    let top = loaded
        .leaderboard
        .first()
        .map_or(0, |r| r.cost_microdollars);
    Some(GroupOverviewView {
        attribution: people_view::attribution_rows(&loaded.attribution),
        models: people_view::model_rows(&loaded.models),
        daily_requests: daily_requests_chart(&loaded.daily),
        skills: people_view::name_count_rows(&loaded.skills),
        tools: people_view::name_count_rows(&loaded.tools),
        leaderboard: loaded
            .leaderboard
            .iter()
            .map(|r| UsageLeaderRowView {
                href: format!("/admin/users/{}", urlencoding::encode(r.user_id.as_str())),
                user_id: r.user_id.clone(),
                requests: r.requests,
                tokens: r.tokens,
                cost_microdollars: r.cost_microdollars,
                share_pct: people_view::share(r.cost_microdollars, top),
            })
            .collect(),
    })
}

async fn load_members(
    pool: &PgPool,
    group_id: &GroupId,
    active: &str,
    user_ctx: &UserContext,
    faults: &mut Faults,
) -> Option<MembersTabView> {
    if active != tabs::MEMBERS {
        return None;
    }
    let loaded = data::load_members(pool, group_id, user_ctx.is_admin, faults).await;
    let assign_targets = if group_id.as_str() == UNASSIGNED_GROUP && user_ctx.is_admin {
        assign_targets(pool, faults).await
    } else {
        Vec::new()
    };
    Some(MembersTabView {
        rows: view::group_member_rows(&loaded, user_ctx.is_admin),
        addable_users: loaded.addable,
        assign_targets,
    })
}

async fn assign_targets(pool: &PgPool, faults: &mut Faults) -> Vec<MemberSetChipView> {
    faults
        .take(
            "assign targets",
            systemprompt_web_admin_core::repositories::scope::containers::list_group_options(pool)
                .await,
        )
        .into_iter()
        .filter(|g| g.id != UNASSIGNED_GROUP)
        .map(|g| MemberSetChipView {
            id: g.id.as_str().to_owned(),
            label: g.name,
        })
        .collect()
}

async fn load_access(
    pool: &PgPool,
    group_id: &GroupId,
    active: &str,
    faults: &mut Faults,
) -> Option<Vec<super::types::AccessSectionView>> {
    if active != tabs::ACCESS {
        return None;
    }
    Some(data::load_access(pool, group_id, faults).await)
}

async fn load_projects(
    pool: &PgPool,
    group_id: &GroupId,
    active: &str,
    faults: &mut Faults,
) -> Option<Vec<super::types::ProjectRowView>> {
    if active != tabs::PROJECTS {
        return None;
    }
    let rows = data::load_projects(pool, group_id, faults).await;
    Some(people_view::linked_rows(&rows, "projects"))
}

async fn load_mappings(
    pool: &PgPool,
    group_id: &GroupId,
    active: &str,
    user_ctx: &UserContext,
    faults: &mut Faults,
) -> Option<Vec<super::types::MappingRowView>> {
    if active != tabs::MAPPINGS {
        return None;
    }
    let rows = data::load_mappings(pool, group_id, faults).await;
    Some(people_view::mapping_rows(
        rows.into_iter().map(|r| (r.ad_group, r.source)),
        user_ctx.is_platform_admin,
    ))
}

fn export_view(group_id: &GroupId) -> systemprompt_web_admin_core::export::view::ExportView {
    let query = systemprompt_web_admin_core::export::view::query_string(&[(
        "group",
        Some(group_id.as_str()),
    )]);
    systemprompt_web_admin_core::export::view::ExportView::new(
        &["requests", "sessions", "analysis-conversations"],
        &query,
    )
    .with_transcripts(systemprompt_web_admin_core::export::transcripts::TranscriptSource::Sessions)
}

fn breadcrumbs(name: &str) -> Vec<BreadcrumbView> {
    vec![
        BreadcrumbView::link("Groups", "/admin/groups"),
        BreadcrumbView::current(name),
    ]
}
