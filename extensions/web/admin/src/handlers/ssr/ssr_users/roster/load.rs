//! The roster's reads: the page of rows, the headline numbers, the role facet,
//! the group and project name maps, and the create-user drawer's choices.
//!
//! The page and its facets are independent reads joined into one round of
//! latency. A failed name-map read degrades to showing the id, which is still a
//! working link; the drawer's listings are read only for someone who can open
//! it.

use std::collections::BTreeMap;

use sqlx::PgPool;

use crate::error::AdminResult;
use crate::repositories;
use crate::repositories::users::roster::{DEFAULT_PAGE_SIZE, RosterQuery};

use super::RosterUrlState;
use super::context::{CreateUserChoiceView, CreateUserPanelView};

pub(super) struct RosterRead {
    pub(super) rows: Vec<repositories::users::roster::RosterRow>,
    pub(super) total: i64,
    pub(super) totals: repositories::users::roster::RosterStats,
    pub(super) roles: Vec<String>,
    pub(super) group_names: BTreeMap<String, String>,
    pub(super) project_names: BTreeMap<String, String>,
}

pub(super) async fn load_roster(
    pool: &PgPool,
    scope: &repositories::scope::SubjectScope,
    url: &RosterUrlState,
) -> AdminResult<RosterRead> {
    let query = RosterQuery {
        filter: url.filter,
        role: url.role.clone(),
        search: url.search.clone(),
        sort: url.sort,
        limit: DEFAULT_PAGE_SIZE,
        offset: url.page.saturating_mul(DEFAULT_PAGE_SIZE),
    };
    let (page_res, stats_res, roles_res, (group_names, project_names)) = tokio::join!(
        repositories::users::roster::list_users_paged(pool, scope, &query),
        repositories::users::roster::get_roster_stats(pool, scope),
        repositories::users::queries::list_distinct_roles(pool),
        load_scope_names(pool),
    );
    let (rows, total) =
        page_res.inspect_err(|e| tracing::warn!(error = %e, "roster page query failed"))?;
    Ok(RosterRead {
        rows,
        total,
        totals: stats_res.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "roster totals query failed");
            repositories::users::roster::RosterStats::default()
        }),
        roles: roles_res
            .inspect_err(|e| tracing::warn!(error = %e, "roster role facet failed"))
            .unwrap_or_default(),
        group_names,
        project_names,
    })
}

pub(super) async fn create_user_panel(
    pool: &PgPool,
    sso: &crate::handlers::adfs_auth::AdfsConfig,
    can_write: bool,
) -> CreateUserPanelView {
    let sso_provisions = sso.enabled && sso.auto_provision;
    if !can_write || sso_provisions {
        return CreateUserPanelView {
            sso_provisions,
            groups: Vec::new(),
            projects: Vec::new(),
        };
    }
    let groups = repositories::groups::crud::list_groups(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "roster: group choices unavailable"))
        .unwrap_or_default()
        .into_iter()
        .filter(|g| !g.is_system)
        .map(|g| CreateUserChoiceView {
            id: g.id.into_inner(),
            name: g.name,
        })
        .collect();
    let projects = repositories::projects::crud::list_projects(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "roster: project choices unavailable"))
        .unwrap_or_default()
        .into_iter()
        .map(|p| CreateUserChoiceView {
            id: p.id.into_inner(),
            name: p.name,
        })
        .collect();
    CreateUserPanelView {
        sso_provisions,
        groups,
        projects,
    }
}

async fn load_scope_names(pool: &PgPool) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    let (groups, projects) = tokio::join!(
        repositories::groups::crud::list_groups(pool),
        repositories::projects::crud::list_projects(pool),
    );
    (
        groups
            .unwrap_or_default()
            .into_iter()
            .map(|g| (g.id.as_str().to_owned(), g.name))
            .collect(),
        projects
            .unwrap_or_default()
            .into_iter()
            .map(|p| (p.id.as_str().to_owned(), p.name))
            .collect(),
    )
}
