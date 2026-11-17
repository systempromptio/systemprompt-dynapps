//! Cost: what the farm's tasks of a date range cost in systemprompt, by
//! agent, tenant and person, in USD.
//!
//! farm-api knows the tasks (agent, tenant, owner); systemprompt knows what
//! each one cost. The console reads the tasks created in the range from
//! farm-api (`since`/`until`, 100 per page, following `next_cursor`), then
//! sums systemprompt's `ai_requests` for all their gateway sessions in one
//! query, and groups the result here. Reading the tasks is bounded, not
//! scalable: at most [`MAX_PAGES`] farm-api pages per request, and when the
//! range holds more the page says its totals are partial. Tasks without a
//! gateway session cannot be linked to systemprompt and are counted apart.


use axum::Extension;
use axum::extract::Query;
use axum::response::Response;
use chrono::Utc;
use serde::Serialize;
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use crate::client::{COST_PAGE_SIZE, FarmApi, is_all_tenants};
use crate::handlers::{TenantQuery, require_console, scope};
use crate::models::{TaskOut, TenantRef};
use crate::sp::SharedSp;
use crate::types::SessionTotalRow;
use crate::views::{TenantSelect, tenant_select};
pub use crate::cost_range::{CostQuery, DEFAULT_DAYS, DayRange, clean, parse_day, resolve_range};
pub use crate::cost_report::{Bucket, CostReport, CostRow, aggregate, rows, sessions, task_session};

const URL: &str = "/admin/farm/cost";
/// farm-api pages read for one request, across all tenants. With
/// [`COST_PAGE_SIZE`] this is the most tasks one cost page counts.
pub const MAX_PAGES: u32 = 20;

/// systemprompt's totals for the sessions of `tasks`, in one query. `None`
/// when there are no tasks to ask about (they could not be read) or
/// systemprompt failed (the fault is recorded).
pub async fn session_totals(
    sp: &SharedSp,
    tasks: Option<&[(String, TaskOut)]>,
    faults: &mut Faults,
) -> Option<Vec<SessionTotalRow>> {
    let ids = sessions(tasks?);
    if ids.is_empty() {
        return Some(Vec::new());
    }
    match sp.session_totals(&ids).await {
        Ok(totals) => Some(totals),
        Err(e) => {
            faults.record("Systemprompt usage", &e);
            None
        },
    }
}

#[derive(Debug, Serialize)]
pub struct CostPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub select: TenantSelect,
    pub from: String,
    pub to: String,
    pub notes: Vec<String>,
    pub report: Option<CostReport>,
    pub partial: Option<String>,
}

pub struct CostInput<'a> {
    pub tenant: &'a str,
    pub tenants: &'a [TenantRef],
    pub range: &'a DayRange,
    pub tasks: Option<&'a [(String, TaskOut)]>,
    /// systemprompt's totals for the tasks' sessions; `None` when it could
    /// not be read (the fault is on the page).
    pub totals: Option<&'a [SessionTotalRow]>,
    pub partial: Option<String>,
}

pub fn cost_view(input: &CostInput<'_>) -> CostPageContext {
    CostPageContext {
        page: "farm-cost",
        title: "Cost",
        breadcrumbs: vec![
            BreadcrumbView::link("Farm", "/admin/farm"),
            BreadcrumbView::current("Cost"),
        ],
        select: tenant_select(URL, input.tenants, input.tenant),
        from: input.range.from.to_string(),
        to: input.range.to.to_string(),
        notes: input.range.notes.clone(),
        report: input
            .tasks
            .zip(input.totals)
            .map(|(tasks, totals)| aggregate(tasks, totals)),
        partial: input.partial.clone(),
    }
}

/// Reads the tasks of `tenants` created in `range`, spending at most
/// `max_pages` farm-api calls in total. A tenant that means "all" is one
/// stream across tenants: farm-api pages it with a single cursor. The second
/// value says why the result is not the whole range, if it is not.
pub async fn fetch(
    api: &FarmApi,
    tenants: &[String],
    range: &DayRange,
    max_pages: u32,
    faults: &mut Faults,
) -> (Vec<(String, TaskOut)>, Option<String>) {
    let (since, until) = range.instants();
    let mut tasks = Vec::new();
    let mut budget = max_pages;
    let mut partial = None;
    'tenants: for tenant in tenants {
        let mut cursor: Option<String> = None;
        loop {
            if budget == 0 {
                partial = Some(format!(
                    "Partial total: the range holds more than the {} tasks this page reads. \
                     Narrow the dates or pick one tenant.",
                    max_pages * COST_PAGE_SIZE
                ));
                break 'tenants;
            }
            budget -= 1;
            match api
                .list_tasks_between(tenant, &since, &until, cursor.as_deref())
                .await
            {
                Ok(page) => {
                    tasks.extend(page.items.into_iter().map(|t| (tenant.clone(), t)));
                    match page.next_cursor.filter(|c| !c.is_empty()) {
                        Some(next) => cursor = Some(next),
                        None => break,
                    }
                },
                Err(e) => {
                    faults.record("Cost", &e);
                    partial = Some(if is_all_tenants(tenant) {
                        "Partial total: the tasks could not be read completely.".to_owned()
                    } else {
                        format!(
                            "Partial total: tasks of tenant {tenant} could not be read completely."
                        )
                    });
                    break;
                },
            }
        }
    }
    (tasks, partial)
}

pub async fn cost_page(
    shell: Page,
    Extension(api): Extension<FarmApi>,
    Extension(sp): Extension<SharedSp>,
    Query(query): Query<CostQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let tenant_query = TenantQuery {
        tenant: query.tenant.clone(),
        cursor: None,
    };
    let mut s = scope(&api, &tenant_query).await;
    let range = resolve_range(
        clean(query.from.as_ref()),
        clean(query.to.as_ref()),
        Utc::now().date_naive(),
    );
    // "All" is one stream of farm-api pages (`tenant=*`), so the page budget
    // is the same for one tenant and for every tenant.
    let (tasks, partial) = if s.ready {
        let (tasks, partial) = fetch(
            &api,
            std::slice::from_ref(&s.tenant),
            &range,
            MAX_PAGES,
            &mut s.faults,
        )
        .await;
        (Some(tasks), partial)
    } else {
        (None, None)
    };
    let totals = session_totals(&sp, tasks.as_deref(), &mut s.faults).await;
    let ctx = cost_view(&CostInput {
        tenant: &s.tenant,
        tenants: &s.tenants,
        range: &range,
        tasks: tasks.as_deref(),
        totals: totals.as_deref(),
        partial,
    });
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-cost",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &s.faults,
    ))
}
