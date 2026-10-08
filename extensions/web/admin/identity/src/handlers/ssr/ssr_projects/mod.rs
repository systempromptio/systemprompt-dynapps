//! `/admin/projects` and `/admin/projects/{project_id}`.
//!
//! A project is work attribution and an ACL subject; a group is people and
//! entitlement. The listing answers "where is the spend and is the tooling
//! working"; the detail page answers it for one project across three tabs —
//! who is on it, what it did, and how it is configured. The report route
//! lays all of it out on one printable page.

use std::sync::Arc;
use systemprompt_web_shared::ProjectId;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::repositories;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::{MarketplaceContext, UserContext};

use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::Viewer;
use systemprompt_web_admin_core::ssr::list_view::{
    DEFAULT_PAGE_SIZE, PageWindow, Pagination, paginate,
};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

mod detail;
mod detail_view;
mod list;
mod report;
mod sort;

pub(super) const BASE_URL: &str = "/admin/projects";
pub(super) const WINDOW_LABEL: &str = "Last 30 days · attributed at request time";

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct ProjectTabQuery {
    tab: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct ProjectReportQuery {
    print: Option<String>,
}

pub(crate) async fn projects_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<list::ListQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let mut faults = Faults::new();
    let data = list::page_data(&pool, &query, user_ctx.is_admin, &mut faults).await;
    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_page_with_faults(
            &engine,
            "projects",
            &data,
            Viewer::new(&user_ctx, &mkt_ctx),
            &faults,
        ),
    )
}

pub(crate) async fn project_detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(project_id): Path<ProjectId>,
    Query(query): Query<ProjectTabQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let Some(project) = repositories::projects::crud::find_project(&pool, &project_id).await?
    else {
        return Err(AdminError::NotFound("No such project.".to_owned()).into());
    };

    let tab = active_tab(query.tab.as_deref());
    let mut faults = Faults::new();
    let loaded = detail::load(&pool, &project_id, &shell.user, &mut faults).await;
    let usage = if tab == "usage" {
        Some(detail::load_usage(&pool, &project_id, &mut faults).await)
    } else {
        None
    };
    let (mappings, rules) = if tab == "settings" {
        settings_reads(&pool, &project_id, &mut faults).await
    } else {
        (Vec::new(), Vec::new())
    };

    let data = detail::page_data(
        &project,
        &loaded,
        &shell.user,
        detail::TabReads {
            tab,
            usage: usage.as_ref(),
            mappings,
            rules: &rules,
        },
    );
    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_page_with_faults(
            &shell.engine,
            "project-detail",
            &data,
            Viewer::new(&shell.user, &shell.marketplace),
            &faults,
        ),
    )
}

pub(crate) async fn project_report_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(project_id): Path<ProjectId>,
    Query(query): Query<ProjectReportQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let Some(project) = repositories::projects::crud::find_project(&pool, &project_id).await?
    else {
        return Err(AdminError::NotFound("No such project.".to_owned()).into());
    };

    let mut faults = Faults::new();
    let mut usage_faults = Faults::new();
    let mut report_faults = Faults::new();
    let (detail, usage, extra) = tokio::join!(
        detail::load(&pool, &project_id, &shell.user, &mut faults),
        detail::load_usage(&pool, &project_id, &mut usage_faults),
        report::load(&pool, &project_id, &mut report_faults),
    );
    faults.extend(usage_faults.into_vec());
    faults.extend(report_faults.into_vec());
    let data = report::page_data(
        report::ReportInputs {
            project: &project,
            detail: &detail,
            usage: &usage,
            report: &extra,
            print_on_load: query.print.as_deref() == Some("1"),
        },
        &shell.user,
    );
    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_page_with_faults(
            &shell.engine,
            "project-report",
            &data,
            Viewer::new(&shell.user, &shell.marketplace),
            &faults,
        ),
    )
}

fn active_tab(requested: Option<&str>) -> &'static str {
    match requested {
        Some("usage") => "usage",
        Some("settings") => "settings",
        _ => "members",
    }
}

async fn settings_reads(
    pool: &PgPool,
    project_id: &ProjectId,
    faults: &mut Faults,
) -> (
    Vec<systemprompt_web_admin_core::types::projects::ProjectAdMappingRow>,
    Vec<systemprompt_web_admin_core::types::access_control::AccessControlRule>,
) {
    let mappings = faults.take(
        "directory mappings",
        repositories::projects::mappings::list_project_ad_mappings(pool, project_id).await,
    );
    let rules = faults
        .take(
            "access rules",
            systemprompt_web_admin_core::repositories::entitlement::list_all_rules(pool).await,
        )
        .into_iter()
        .filter(|r| r.rule_type.as_str() == "project" && r.rule_value == project_id.as_str())
        .collect();
    (mappings, rules)
}

fn project_query(project_id: &ProjectId) -> String {
    systemprompt_web_admin_core::export::view::query_string(&[(
        "project",
        Some(project_id.as_str()),
    )])
}

pub(super) fn export_view(
    project_id: &ProjectId,
) -> systemprompt_web_admin_core::export::view::ExportView {
    systemprompt_web_admin_core::export::view::ExportView::new(
        &["requests", "sessions", "analysis-conversations"],
        &project_query(project_id),
    )
    .with_transcripts(systemprompt_web_admin_core::export::transcripts::TranscriptSource::Sessions)
}

pub(super) fn report_export_view(
    project_id: &ProjectId,
) -> systemprompt_web_admin_core::export::view::ExportView {
    systemprompt_web_admin_core::export::view::ExportView::new(
        &[
            "report-customer-users",
            "report-customer-projects",
            "report-customer-models",
        ],
        &project_query(project_id),
    )
}

pub(super) fn breadcrumbs(name: &str) -> Vec<BreadcrumbView> {
    vec![
        BreadcrumbView::link("Projects", BASE_URL),
        BreadcrumbView::current(name),
    ]
}

pub(super) fn pct(part: i64, whole: i64) -> i64 {
    if whole <= 0 {
        return 0;
    }
    (part * 100 / whole).clamp(0, 100)
}

#[derive(Clone, Copy)]
pub(super) struct PageCounts {
    pub(super) page: i64,
    pub(super) total: i64,
    pub(super) offset: i64,
    pub(super) shown: i64,
}

pub(super) fn pagination(counts: PageCounts, noun: &'static str, base: &str) -> Pagination {
    let PageCounts {
        page,
        total,
        offset,
        shown,
    } = counts;
    let window = PageWindow::new(page - 1, DEFAULT_PAGE_SIZE, total, shown, noun);
    debug_assert_eq!(window.bounds().0, if shown == 0 { 0 } else { offset + 1 });
    paginate(window, |index| format!("{base}&page={}", index + 1))
}
