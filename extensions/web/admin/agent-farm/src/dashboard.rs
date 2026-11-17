//! Dashboard: one page that summarises the farm, all tenants mixed unless one
//! is chosen, and links to each view. Tenants, agents, schedules, credentials,
//! the tasks of the last seven days by state, and what systemprompt recorded
//! they cost.
//!
//! The number of farm-api calls is fixed and small: `whoami`, one list each
//! for agents and schedules, the two credential scopes, and at most
//! [`TASK_PAGES`] pages of tasks (which both count the states and name the
//! sessions whose cost systemprompt sums in one query). When the period holds
//! more tasks than those pages, the page says its numbers are partial. Any
//! block that cannot be read shows a dash, adds a fault to the shell's notice
//! and leaves the rest of the page standing.

use std::collections::BTreeMap;

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

use crate::client::{FarmApi, is_all_tenants};
use crate::cost::{self, CostReport, DayRange, aggregate, resolve_range, session_totals};
use crate::handlers::{TenantQuery, or_fault, require_console, scope};
use crate::models::{AgentList, CredentialList, ScheduleList, ScheduleOut, TaskOut, TenantRef};
use crate::sp::SharedSp;
use crate::types::SessionTotalRow;
use crate::usage::count_label;
use crate::views::{TenantSelect, href, state_tone, tenant_select};

const URL: &str = "/admin/farm";
pub const NONE: &str = "—";
pub const TASK_PAGES: u32 = 3;

/// One clickable tile (`components/kpi`).
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Tile {
    pub label: String,
    pub value: String,
    pub note: Option<String>,
    pub href: String,
    pub tone: Option<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct DashboardContext {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub select: TenantSelect,
    pub overview: Vec<Tile>,
    pub task_tiles: Vec<Tile>,
    pub period: String,
    pub partial: Option<String>,
}

/// What the dashboard read. A `None` is a block that could not be read (the
/// reason is on the page's faults).
pub struct DashboardInput<'a> {
    pub tenant: &'a str,
    pub tenants: &'a [TenantRef],
    pub range: &'a DayRange,
    pub agents: Option<&'a AgentList>,
    pub schedules: Option<&'a ScheduleList>,
    pub credentials: Option<usize>,
    pub tasks: Option<&'a [(String, TaskOut)]>,
    pub totals: Option<&'a [SessionTotalRow]>,
    pub partial: Option<String>,
}

fn link(view: &str, tenant: &str) -> String {
    href(&format!("{URL}/{view}"), &[("tenant", tenant)])
}

pub fn count_or_dash(n: Option<usize>) -> String {
    n.map_or_else(|| NONE.to_owned(), |n| n.to_string())
}

pub fn state_rank(state: &str) -> u8 {
    match state {
        "running" => 0,
        "queued" | "pending" => 1,
        "succeeded" => 2,
        "failed" => 3,
        _ => 4,
    }
}

pub fn task_tiles(tasks: Option<&[(String, TaskOut)]>, tenant: &str) -> Vec<Tile> {
    let href = link("tasks", tenant);
    let Some(tasks) = tasks else {
        return vec![Tile {
            label: "Tasks".to_owned(),
            value: NONE.to_owned(),
            note: Some("unavailable".to_owned()),
            href,
            tone: None,
        }];
    };
    let mut by_state: BTreeMap<(u8, &str), usize> = BTreeMap::new();
    for (_, t) in tasks {
        *by_state
            .entry((state_rank(&t.state), t.state.as_str()))
            .or_default() += 1;
    }
    let mut tiles = vec![Tile {
        label: "Tasks".to_owned(),
        value: tasks.len().to_string(),
        note: Some("in the period".to_owned()),
        href: href.clone(),
        tone: None,
    }];
    tiles.extend(by_state.into_iter().map(|((_, state), n)| Tile {
        label: if state.is_empty() { "unknown" } else { state }.to_owned(),
        value: n.to_string(),
        note: None,
        href: href.clone(),
        tone: Some(state_tone(state)).filter(|t| *t != "muted"),
    }));
    tiles
}

fn schedule_note(schedules: &[ScheduleOut]) -> String {
    let active = schedules.iter().filter(|s| s.state == "active").count();
    format!("{active} active")
}

fn tenants_tile(input: &DashboardInput<'_>) -> Tile {
    Tile {
        label: "Tenants".to_owned(),
        value: if input.tenants.is_empty() {
            NONE.to_owned()
        } else {
            input.tenants.len().to_string()
        },
        note: Some(if is_all_tenants(input.tenant) {
            "showing all".to_owned()
        } else {
            format!("showing {}", input.tenant)
        }),
        href: format!("{URL}/tenants"),
        tone: None,
    }
}

fn cost_tile(input: &DashboardInput<'_>, report: Option<&CostReport>) -> Tile {
    Tile {
        label: "Cost (USD)".to_owned(),
        value: report.map_or_else(|| NONE.to_owned(), |r| r.total_cost.clone()),
        note: report.map(|r| r.calls_label.clone()),
        href: href(
            &format!("{URL}/cost"),
            &[
                ("tenant", input.tenant),
                ("from", &input.range.from.to_string()),
                ("to", &input.range.to.to_string()),
            ],
        ),
        tone: None,
    }
}

pub fn dashboard_view(input: &DashboardInput<'_>) -> DashboardContext {
    let tenant = input.tenant;
    let report = input
        .tasks
        .zip(input.totals)
        .map(|(tasks, totals)| aggregate(tasks, totals));
    let overview = vec![
        tenants_tile(input),
        Tile {
            label: "Agents".to_owned(),
            value: count_or_dash(input.agents.map(|a| a.items.len())),
            note: None,
            href: link("agents", tenant),
            tone: None,
        },
        Tile {
            label: "Schedules".to_owned(),
            value: count_or_dash(input.schedules.map(|s| s.items.len())),
            note: input.schedules.map(|s| schedule_note(&s.items)),
            href: link("schedules", tenant),
            tone: None,
        },
        Tile {
            label: "Credentials".to_owned(),
            value: count_or_dash(input.credentials),
            note: Some("names only".to_owned()),
            href: link("credentials", tenant),
            tone: None,
        },
        cost_tile(input, report.as_ref()),
    ];
    DashboardContext {
        page: "farm",
        title: "Farm",
        breadcrumbs: vec![BreadcrumbView::current("Farm")],
        select: tenant_select(URL, input.tenants, input.tenant),
        overview,
        task_tiles: task_tiles(input.tasks, tenant),
        period: format!(
            "{} to {} (UTC), {}",
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

pub async fn dashboard_page(
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
    let mut credentials = None;
    let mut tasks = None;
    let mut partial = None;
    let mut errors = Faults::new();
    if s.ready {
        let tenant = s.tenant.clone();
        let (a, sc, personal, shared, read) = tokio::join!(
            api.list_agents(&tenant),
            api.list_schedules(&tenant),
            api.list_credentials(&tenant, "personal"),
            api.list_credentials(&tenant, "shared"),
            cost::fetch(
                &api,
                std::slice::from_ref(&tenant),
                &range,
                TASK_PAGES,
                &mut errors
            ),
        );
        agents = or_fault(a, "Agents", &mut s.faults);
        schedules = or_fault(sc, "Schedules", &mut s.faults);
        credentials = match (
            or_fault(personal, "Credentials", &mut s.faults),
            or_fault(shared, "Credentials", &mut s.faults),
        ) {
            (Some(p), Some(sh)) => Some(credential_count(&p, &sh)),
            _ => None,
        };
        let unread = read.0.is_empty() && !errors.as_slice().is_empty();
        s.faults.extend(errors.into_vec());
        tasks = (!unread).then_some(read.0);
        partial = read.1;
    }
    let totals = session_totals(&sp, tasks.as_deref(), &mut s.faults).await;
    let ctx = dashboard_view(&DashboardInput {
        tenant: &s.tenant,
        tenants: &s.tenants,
        range: &range,
        agents: agents.as_ref(),
        schedules: schedules.as_ref(),
        credentials,
        tasks: tasks.as_deref(),
        totals: totals.as_deref(),
        partial,
    });
    Ok(render_page_with_faults(
        &shell.engine,
        "farm",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &s.faults,
    ))
}

const fn credential_count(personal: &CredentialList, shared: &CredentialList) -> usize {
    personal.items.len() + shared.items.len()
}
