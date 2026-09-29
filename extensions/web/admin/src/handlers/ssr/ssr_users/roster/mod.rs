//! `/admin/users` — the roster.
//!
//! One flat, paged table: who is spending what is the number an admin comes
//! here for, so group membership is a column and a filter, not a bucketing.
//!
//! Every facet lives in the URL —
//! `?filter=&role=&q=&group=&project=&sort=&dir=&page=` — so a view is
//! bookmarkable and the browser back button works.

mod columns;
mod context;
mod form;
mod load;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::export::view::ExportView;
use systemprompt_web_admin_core::ssr::list_view::PageWindow;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use crate::repositories::users::roster::{DEFAULT_PAGE_SIZE, RosterFilter, RosterSort};
use systemprompt_web_admin_core::repositories::scope::ScopeRequest;

use context::RosterContext;
use form::{preserved_hidden, role_options};
use load::{create_user_panel, load_roster};

use super::BASE_URL;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::Viewer;
use systemprompt_web_admin_core::ssr::page::Page;

pub(super) struct RosterUrlState {
    pub filter: RosterFilter,
    pub role: Option<String>,
    pub search: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
    pub sort: RosterSort,
    pub page: i64,
}

impl RosterUrlState {
    fn from_params(params: &RosterQueryParams) -> Self {
        Self {
            filter: RosterFilter::parse_filter(params.filter.as_deref()),
            role: non_empty(params.role.as_deref()),
            search: non_empty(params.q.as_deref()),
            group: non_empty(params.group.as_deref()),
            project: non_empty(params.project.as_deref()),
            sort: RosterSort::parse_sort(params.sort.as_deref(), params.dir.as_deref()),
            page: params.page.unwrap_or(0).max(0),
        }
    }

    const fn dir_param(&self) -> &'static str {
        if self.sort.descending { "desc" } else { "asc" }
    }

    fn link_prefix(&self, drop: &[&str]) -> String {
        let mut parts: Vec<(String, String)> = Vec::new();
        let mut push = |key: &str, value: String| {
            if !value.is_empty() && !drop.contains(&key) {
                parts.push((key.to_owned(), value));
            }
        };
        push("filter", self.filter.as_str().to_owned());
        push("role", self.role.clone().unwrap_or_default());
        push("q", self.search.clone().unwrap_or_default());
        push("group", self.group.clone().unwrap_or_default());
        push("project", self.project.clone().unwrap_or_default());
        push("sort", self.sort.column.to_owned());
        push("dir", self.dir_param().to_owned());
        if self.page > 0 && !drop.contains(&"page") {
            parts.push(("page".to_owned(), self.page.to_string()));
        }
        let qs = parts
            .into_iter()
            .map(|(k, v)| format!("{k}={}", urlencoding::encode(&v)))
            .collect::<Vec<_>>()
            .join("&");
        if qs.is_empty() {
            format!("{BASE_URL}?")
        } else {
            format!("{BASE_URL}?{qs}&")
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct RosterQueryParams {
    pub filter: Option<String>,
    pub role: Option<String>,
    pub q: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
}

pub(crate) async fn users_page(
    shell: Page,
    Extension(sso): Extension<crate::handlers::adfs_auth::AdfsDeps>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<RosterQueryParams>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let url = RosterUrlState::from_params(&params);
    let request =
        ScopeRequest::from_query(&shell.user, url.group.as_deref(), url.project.as_deref());
    let scope = systemprompt_web_admin_core::repositories::scope::membership::get_subject_scope(
        &pool, &request,
    )
    .await?;
    let mut faults = Faults::new();
    let read = load_roster(&pool, &scope, &url, &mut faults).await?;

    let shown = i64::try_from(read.rows.len()).unwrap_or(DEFAULT_PAGE_SIZE);
    let window = PageWindow::new(url.page, DEFAULT_PAGE_SIZE, read.total, shown, "users");
    let rows = view::build_rows(&view::RowInput {
        rows: &read.rows,
        group_names: &read.group_names,
        project_names: &read.project_names,
    });

    let data = RosterContext {
        page: "users",
        title: "Users",
        breadcrumbs: vec![BreadcrumbView::current("Users")],
        kpis: view::build_kpis(&url, &read.totals),
        chips: view::build_chips(&url, &read.totals),
        role_options: role_options(&read.roles, url.role.as_deref()),
        search: url.search.clone().unwrap_or_default(),
        scope_filter: systemprompt_web_admin_core::ssr::list_view::scope_filter_from_names(
            &shell.user,
            &request,
            BASE_URL,
            preserved_hidden(&url),
            systemprompt_web_admin_core::ssr::list_view::ScopeNames {
                groups: &read.group_names,
                projects: &read.project_names,
            },
        ),
        sort_headers: columns::build_sort_headers(&url, url.sort),
        has_rows: !rows.is_empty(),
        row_count: rows.len(),
        rows,
        pagination: view::build_pagination(&url, window),
        can_write: shell.user.is_admin,
        role_choices: read.roles,
        create_user: create_user_panel(&pool, &sso.config, shell.user.is_admin, &mut faults).await,
        export: export_view(&url),
    };

    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_page_with_faults(
            &shell.engine,
            "users",
            &data,
            Viewer::new(&shell.user, &shell.marketplace),
            &faults,
        ),
    )
}

fn export_view(url: &RosterUrlState) -> ExportView {
    ExportView::single(
        "users",
        &systemprompt_web_admin_core::export::view::query_string(&[
            ("filter", Some(url.filter.as_str())),
            ("role", url.role.as_deref()),
            ("q", url.search.as_deref()),
            ("group", url.group.as_deref()),
            ("project", url.project.as_deref()),
            ("sort", Some(url.sort.column)),
            ("dir", Some(url.dir_param())),
        ]),
    )
}

fn non_empty(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}
