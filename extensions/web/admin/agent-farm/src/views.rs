//! View models of the two pages, built from farm-api responses. Pure: no I/O,
//! so every label and tone is unit-testable without a server.
//!
//! Costs come from systemprompt ([`crate::usage`]), never from farm-api's
//! `usage.cost_eur`.

use std::collections::HashMap;
use std::hash::BuildHasher;

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use crate::client::is_all_tenants;
use crate::models::{TaskList, TaskOut, TenantRef};
use crate::usage::{format_usd, gateway_session_id};
pub use crate::views_task::{
    LogsView, ResultView, TaskDetailView, TaskInput, TaskPageContext, identity, limits, logs_view,
    result_note, result_note_view, result_view, task_detail, task_page, times, usage,
};
pub use crate::views_tenant::{TenantOption, TenantSelect, tenant_options, tenant_select};

pub const BASE_URL: &str = "/admin/farm/tasks";
const UNKNOWN_TENANT: &str = "—";
pub const NONE: &str = "—";

#[derive(Debug, Serialize)]
pub struct TaskRowView {
    #[serde(rename = "task_id")]
    pub id: String,
    pub short_id: String,
    pub href: String,
    pub agent: String,
    pub is_prompt: bool,
    pub scope: Option<String>,
    pub tenant: String,
    pub owner: String,
    pub state: String,
    pub state_tone: &'static str,
    pub failure_reason: Option<String>,
    pub started: String,
    pub finished: String,
    pub cost: String,
}

#[derive(Debug, Serialize)]
pub struct TasksPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub select: TenantSelect,
    pub rows: Vec<TaskRowView>,
    pub has_rows: bool,
    pub count_label: String,
    pub loaded: bool,
    pub next_url: Option<String>,
    pub first_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct KeyValue {
    pub label: &'static str,
    pub value: String,
}

pub fn state_tone(state: &str) -> &'static str {
    match state {
        "succeeded" => "ok",
        "failed" => "err",
        "running" => "accent",
        "queued" | "pending" => "info",
        "cancelled" | "expired" => "muted",
        _ => "warn",
    }
}

pub fn format_time(value: Option<&str>) -> String {
    let Some(raw) = value.filter(|v| !v.is_empty()) else {
        return NONE.to_owned();
    };
    DateTime::parse_from_rfc3339(raw).map_or_else(
        |_| raw.to_owned(),
        |t| {
            t.with_timezone(&Utc)
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string()
        },
    )
}

pub fn format_duration(seconds: Option<f64>) -> String {
    let Some(s) = seconds.filter(|s| s.is_finite() && *s >= 0.0) else {
        return NONE.to_owned();
    };
    let total = s.round() as i64;
    match total {
        t if t < 60 => format!("{t} s"),
        t if t < 3600 => format!("{} min {} s", t / 60, t % 60),
        t => format!("{} h {} min", t / 3600, (t % 3600) / 60),
    }
}

pub fn opt_num(value: Option<i64>) -> String {
    value.map_or_else(|| NONE.to_owned(), |v| v.to_string())
}

pub fn opt_text(value: Option<&str>) -> String {
    value
        .filter(|v| !v.is_empty())
        .map_or_else(|| NONE.to_owned(), ToOwned::to_owned)
}

pub fn agent_label(task: &TaskOut) -> (String, bool) {
    task.agent.as_deref().filter(|a| !a.is_empty()).map_or_else(
        || ("free prompt".to_owned(), true),
        |a| (a.to_owned(), false),
    )
}

pub fn href(base: &str, pairs: &[(&str, &str)]) -> String {
    let mut url = base.to_owned();
    let mut sep = '?';
    for (key, value) in pairs {
        if value.is_empty() || (*key == "tenant" && is_all_tenants(value)) {
            continue;
        }
        url.push(sep);
        url.push_str(key);
        url.push('=');
        url.push_str(&urlencoding::encode(value));
        sep = '&';
    }
    url
}

pub fn row_tenant(own: Option<&str>, filter: &str) -> Option<String> {
    own.filter(|t| !t.is_empty())
        .or_else(|| Some(filter).filter(|f| !is_all_tenants(f)))
        .map(ToOwned::to_owned)
}

pub fn tenant_label(tenant: Option<&str>) -> String {
    tenant.map_or_else(|| UNKNOWN_TENANT.to_owned(), ToOwned::to_owned)
}

pub fn task_href(id: &str, tenant: Option<&str>) -> String {
    href(
        &format!("{BASE_URL}/{}", urlencoding::encode(id)),
        &[("tenant", tenant.unwrap_or_default())],
    )
}

pub fn list_href(tenant: &str, cursor: Option<&str>) -> String {
    href(
        BASE_URL,
        &[("tenant", tenant), ("cursor", cursor.unwrap_or_default())],
    )
}

pub fn short_id(id: &str) -> String {
    id.chars().take(8).collect()
}

fn row_cost<H: BuildHasher>(task: &TaskOut, costs: Option<&HashMap<String, i64, H>>) -> String {
    let session = gateway_session_id(
        task.usage
            .as_ref()
            .and_then(|u| u.gateway_session_id.as_deref()),
    );
    match (session, costs) {
        (Some(id), Some(costs)) => format_usd(costs.get(&id).copied().unwrap_or(0)),
        _ => NONE.to_owned(),
    }
}

pub fn task_row<H: BuildHasher>(
    task: &TaskOut,
    tenant: &str,
    costs: Option<&HashMap<String, i64, H>>,
) -> TaskRowView {
    let (agent, is_prompt) = agent_label(task);
    let own = row_tenant(task.tenant.as_deref(), tenant);
    TaskRowView {
        href: task_href(&task.id, own.as_deref()),
        short_id: short_id(&task.id),
        id: task.id.clone(),
        agent,
        is_prompt,
        scope: task.scope.clone().filter(|s| !s.is_empty()),
        tenant: tenant_label(own.as_deref()),
        owner: opt_text(task.owner.as_deref()),
        state_tone: state_tone(&task.state),
        state: task.state.clone(),
        failure_reason: task.failure_reason.clone(),
        started: format_time(task.started_at.as_deref()),
        finished: format_time(task.finished_at.as_deref()),
        cost: row_cost(task, costs),
    }
}

/// What the list page shows. `list` is `None` when farm-api could not be
/// read; the reason is already on the page's fault list.
#[derive(Debug)]
pub struct TasksInput<'a> {
    pub tenant: &'a str,
    pub tenants: &'a [TenantRef],
    pub cursor: Option<&'a str>,
    pub list: Option<&'a TaskList>,
    pub costs: Option<&'a HashMap<String, i64>>,
}

pub fn tasks_page(input: &TasksInput<'_>) -> TasksPageContext {
    let rows: Vec<TaskRowView> = input
        .list
        .map(|l| {
            l.items
                .iter()
                .map(|t| task_row(t, input.tenant, input.costs))
                .collect()
        })
        .unwrap_or_default();
    let next_cursor = input
        .list
        .and_then(|l| l.next_cursor.as_deref())
        .filter(|c| !c.is_empty());
    let count_label = match rows.len() {
        1 => "1 task on this page".to_owned(),
        n => format!("{n} tasks on this page"),
    };
    TasksPageContext {
        page: "farm-tasks",
        title: "Tasks",
        breadcrumbs: vec![
            BreadcrumbView::link("Farm", "/admin/farm"),
            BreadcrumbView::current("Tasks"),
        ],
        select: tenant_select(BASE_URL, input.tenants, input.tenant),
        has_rows: !rows.is_empty(),
        rows,
        count_label,
        loaded: input.list.is_some(),
        next_url: next_cursor.map(|c| list_href(input.tenant, Some(c))),
        first_url: input.cursor.map(|_| list_href(input.tenant, None)),
    }
}
