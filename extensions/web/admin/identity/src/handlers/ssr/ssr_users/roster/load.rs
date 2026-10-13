//! The roster's reads: the page of rows, the headline numbers, the role facet,
//! the group and project name maps, and the create-user drawer's choices.
//!
//! The page and its facets are independent reads joined into one round of
//! latency. Only the page itself fails the request; a failed facet or name-map
//! read is recorded on the page's fault list and degrades to its empty value
//! (a name map falls back to showing the id, which is still a working link).
//! The drawer's listings are read only for someone who can open it.

use std::collections::BTreeMap;

use sqlx::PgPool;

use crate::repositories;
use crate::repositories::users::roster::{DEFAULT_PAGE_SIZE, RosterQuery};
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::ssr::faults::Faults;

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
    scope: &systemprompt_web_admin_core::repositories::scope::SubjectScope,
    url: &RosterUrlState,
    faults: &mut Faults,
) -> AdminResult<RosterRead> {
    let query = RosterQuery {
        filter: url.filter,
        role: url.role.clone(),
        search: url.search.clone(),
        sort: url.sort,
        limit: DEFAULT_PAGE_SIZE,
        offset: url.page.saturating_mul(DEFAULT_PAGE_SIZE),
    };
    let (page_res, stats_res, roles_res, groups_res, projects_res) = tokio::join!(
        repositories::users::roster::list_users_paged(pool, scope, &query),
        repositories::users::roster::get_roster_stats(pool, scope),
        repositories::users::queries::list_distinct_roles(pool),
        systemprompt_web_admin_core::repositories::scope::containers::list_group_options(pool),
        systemprompt_web_admin_core::repositories::scope::containers::list_project_options(pool),
    );
    let (rows, total) = page_res?;
    Ok(RosterRead {
        rows,
        total,
        totals: faults.take("roster totals", stats_res),
        roles: faults.take("role facet", roles_res),
        group_names: faults
            .take("group names", groups_res)
            .into_iter()
            .map(|g| (g.id.as_str().to_owned(), g.name))
            .collect(),
        project_names: faults
            .take("project names", projects_res)
            .into_iter()
            .map(|p| (p.id.as_str().to_owned(), p.name))
            .collect(),
    })
}

pub(super) async fn create_user_panel(
    pool: &PgPool,
    sso_provisions: bool,
    can_write: bool,
    faults: &mut Faults,
) -> CreateUserPanelView {
    if !can_write || sso_provisions {
        return CreateUserPanelView {
            sso_provisions,
            groups: Vec::new(),
            projects: Vec::new(),
        };
    }
    let groups = faults
        .take(
            "group choices",
            systemprompt_web_admin_core::repositories::scope::containers::list_group_options(pool)
                .await,
        )
        .into_iter()
        .filter(|g| !g.is_system)
        .map(|g| CreateUserChoiceView {
            id: g.id.into_inner(),
            name: g.name,
        })
        .collect();
    let projects = faults
        .take(
            "project choices",
            systemprompt_web_admin_core::repositories::scope::containers::list_project_options(
                pool,
            )
            .await,
        )
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
