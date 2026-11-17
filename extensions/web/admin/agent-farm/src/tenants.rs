//! Tenants: one row per tenant the credential can see, with what each runs.
//! Agents, schedules, the tasks of the last seven days by state, and what
//! systemprompt recorded they cost.
//!
//! The farm-api budget does not grow with the number of tenants: `whoami`, one
//! agents list and one schedules list read with `tenant=*`, and at most
//! [`TASK_PAGES`] pages of tasks read the same way. The rows are those lists
//! grouped by the `tenant` each item carries. A period with more tasks than
//! the pages hold says its task numbers are partial. Every number links to the
//! view that holds it, filtered by the row's tenant.

use std::collections::BTreeMap;

use axum::Extension;
use axum::response::Response;
use chrono::Utc;
use serde::Serialize;
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use crate::client::{ALL_TENANTS, FarmApi};
use crate::cost::{self, CostReport, DayRange, aggregate, resolve_range, session_totals};
use crate::dashboard::{TASK_PAGES, state_rank};
use crate::handlers::{TenantQuery, or_fault, require_console, scope};
use crate::models::{AgentList, ScheduleList, TaskOut, TenantRef};
use crate::sp::SharedSp;
use crate::types::SessionTotalRow;
use crate::usage::{count_label, format_usd};
use crate::views::{href, state_tone};
use axum::extract::Query;

const NONE: &str = "—";

/// A count that links to the view holding the items.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Counted {
    pub value: String,
    pub href: String,
}

/// How many of a tenant's tasks are in one state.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct StateCount {
    pub label: String,
    pub tone: &'static str,
}

#[derive(Debug, Serialize)]
pub struct TenantRowView {
    pub slug: String,
    pub name: String,
    pub has_name: bool,
    pub href: String,
    pub agents: Counted,
    pub schedules: Counted,
    pub tasks: Counted,
    pub states: Vec<StateCount>,
    pub cost: Counted,
    pub credentials_href: String,
}

#[derive(Debug, Serialize)]
pub struct TenantsPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub rows: Vec<TenantRowView>,
    pub has_rows: bool,
    pub loaded: bool,
    pub count_label: String,
    pub period: String,
    pub partial: Option<String>,
}

/// What the page read. A `None` is a block that could not be read (the reason
/// is on the page's faults) and shows a dash.
pub struct TenantsInput<'a> {
    pub tenants: &'a [TenantRef],
    pub range: &'a DayRange,
    pub agents: Option<&'a AgentList>,
    pub schedules: Option<&'a ScheduleList>,
    pub tasks: Option<&'a [(String, TaskOut)]>,
    pub totals: Option<&'a [SessionTotalRow]>,
    pub partial: Option<String>,
    pub loaded: bool,
}

fn tenant_of(own: Option<&String>) -> Option<&str> {
    own.map(String::as_str).filter(|t| !t.is_empty())
}

fn count_in<T>(items: &[T], tenant: impl Fn(&T) -> Option<&String>, slug: &str) -> usize {
    items
        .iter()
        .filter(|item| tenant_of(tenant(item)) == Some(slug))
        .count()
}

fn counted(value: Option<usize>, view: &str, slug: &str) -> Counted {
    Counted {
        value: value.map_or_else(|| NONE.to_owned(), |n| n.to_string()),
        href: href(&format!("/admin/farm/{view}"), &[("tenant", slug)]),
    }
}

fn state_counts(tasks: &[(String, TaskOut)], slug: &str) -> Vec<StateCount> {
    let mut by_state: BTreeMap<(u8, &str), usize> = BTreeMap::new();
    for (_, t) in tasks
        .iter()
        .filter(|(_, t)| tenant_of(t.tenant.as_ref()) == Some(slug))
    {
        *by_state
            .entry((state_rank(&t.state), t.state.as_str()))
            .or_default() += 1;
    }
    by_state
        .into_iter()
        .map(|((_, state), n)| StateCount {
            label: format!("{n} {}", if state.is_empty() { "unknown" } else { state }),
            tone: state_tone(state),
        })
        .collect()
}

fn tenant_row(
    t: &TenantRef,
    input: &TenantsInput<'_>,
    report: Option<&CostReport>,
) -> TenantRowView {
    let slug = t.tenant.as_str();
    let task_count = input
        .tasks
        .map(|tasks| count_in(&tasks[..], |(_, task)| task.tenant.as_ref(), slug));
    let name = t
        .name
        .as_deref()
        .filter(|n| !n.is_empty() && *n != slug)
        .map(ToOwned::to_owned);
    let cost = report.map(|r| {
        r.by_tenant
            .iter()
            .find(|row| row.key == slug)
            .map_or_else(|| format_usd(0), |row| row.cost.clone())
    });
    TenantRowView {
        slug: t.tenant.clone(),
        has_name: name.is_some(),
        name: name.unwrap_or_else(|| t.tenant.clone()),
        href: href("/admin/farm", &[("tenant", slug)]),
        agents: counted(
            input
                .agents
                .map(|a| count_in(&a.items[..], |x| x.tenant.as_ref(), slug)),
            "agents",
            slug,
        ),
        schedules: counted(
            input
                .schedules
                .map(|s| count_in(&s.items[..], |x| x.tenant.as_ref(), slug)),
            "schedules",
            slug,
        ),
        tasks: counted(task_count, "tasks", slug),
        states: input
            .tasks
            .map(|tasks| state_counts(tasks, slug))
            .unwrap_or_default(),
        cost: Counted {
            value: cost.unwrap_or_else(|| NONE.to_owned()),
            href: href(
                "/admin/farm/cost",
                &[
                    ("tenant", slug),
                    ("from", &input.range.from.to_string()),
                    ("to", &input.range.to.to_string()),
                ],
            ),
        },
        credentials_href: href("/admin/farm/credentials", &[("tenant", slug)]),
    }
}

pub fn tenants_view(input: &TenantsInput<'_>) -> TenantsPageContext {
    let report = input
        .tasks
        .zip(input.totals)
        .map(|(tasks, totals)| aggregate(tasks, totals));
    let rows: Vec<TenantRowView> = input
        .tenants
        .iter()
        .map(|t| tenant_row(t, input, report.as_ref()))
        .collect();
    TenantsPageContext {
        page: "farm-tenants",
        title: "Tenants",
        breadcrumbs: vec![
            BreadcrumbView::link("Farm", "/admin/farm"),
            BreadcrumbView::current("Tenants"),
        ],
        has_rows: !rows.is_empty(),
        count_label: count_label(rows.len() as i64, "tenant", "tenants"),
        rows,
        loaded: input.loaded,
        period: format!(
            "Tasks and cost: {} to {} (UTC), {}",
            input.range.from,
            input.range.to,
            input.tasks.map_or_else(
                || "tasks unavailable".to_owned(),
                |t| count_label(t.len() as i64, "task", "tasks")
            )
        ),
        partial: input.partial.clone(),
    }
}

pub async fn tenants_page(
    shell: Page,
    Extension(api): Extension<FarmApi>,
    Extension(sp): Extension<SharedSp>,
    Query(query): Query<TenantQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let mut s = scope(&api, &query).await;
    let range = resolve_range(None, None, Utc::now().date_naive());

    let mut agents = None;
    let mut schedules = None;
    let mut tasks = None;
    let mut partial = None;
    let mut errors = Faults::new();
    if s.ready {
        let all = ALL_TENANTS.to_owned();
        let (a, sc, read) = tokio::join!(
            api.list_agents(&all),
            api.list_schedules(&all),
            cost::fetch(
                &api,
                std::slice::from_ref(&all),
                &range,
                TASK_PAGES,
                &mut errors
            ),
        );
        agents = or_fault(a, "Agents", &mut s.faults);
        schedules = or_fault(sc, "Schedules", &mut s.faults);
        let unread = read.0.is_empty() && !errors.as_slice().is_empty();
        s.faults.extend(errors.into_vec());
        tasks = (!unread).then_some(read.0);
        partial = read.1;
    }
    let totals = session_totals(&sp, tasks.as_deref(), &mut s.faults).await;
    let ctx = tenants_view(&TenantsInput {
        tenants: &s.tenants,
        range: &range,
        agents: agents.as_ref(),
        schedules: schedules.as_ref(),
        tasks: tasks.as_deref(),
        totals: totals.as_deref(),
        partial,
        loaded: s.ready,
    });
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-tenants",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &s.faults,
    ))
}
