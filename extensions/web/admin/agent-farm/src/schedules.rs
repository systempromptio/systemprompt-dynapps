//! Schedules: cron, next firing and the outcome of the last one. The stored
//! task body carries the prompt, so only the agent and scope it names are read.

use axum::Extension;
use axum::extract::Query;
use axum::response::Response;
use serde::Serialize;
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use crate::client::FarmApi;
use crate::handlers::{TenantQuery, require_console, scope};
use crate::models::{ScheduleOut, TenantRef};
use crate::views::{
    TenantSelect, format_time, opt_text, row_tenant, short_id, tenant_label, tenant_select,
};

const URL: &str = "/admin/farm/schedules";

#[derive(Debug, Serialize)]
pub struct ScheduleRowView {
    pub schedule_id: String,
    pub short_id: String,
    pub agent: String,
    pub is_prompt: bool,
    pub scope: Option<String>,
    pub tenant: String,
    pub owner: String,
    pub cron: String,
    pub timezone: String,
    pub window: Option<String>,
    pub overlap: String,
    pub state: String,
    pub state_tone: &'static str,
    pub next_fire: String,
    pub last_fired: String,
    pub last_state: String,
    pub last_tone: &'static str,
    pub last_error: Option<String>,
    pub runs: String,
}

#[derive(Debug, Serialize)]
pub struct SchedulesPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub select: TenantSelect,
    pub rows: Vec<ScheduleRowView>,
    pub has_rows: bool,
    pub loaded: bool,
    pub count_label: String,
}

pub fn state_tone(state: &str) -> &'static str {
    match state {
        "active" => "ok",
        "finished" => "muted",
        _ => "warn",
    }
}

pub fn last_fire_tone(state: &str) -> &'static str {
    match state {
        "ok" => "ok",
        "error" => "err",
        "skipped_overlap" | "skipped_quota" => "warn",
        _ => "muted",
    }
}

pub fn schedule_row(s: &ScheduleOut, filter: &str) -> ScheduleRowView {
    let rec = s.recurrence.clone().unwrap_or_default();
    let task = s.task.clone().unwrap_or_default();
    let agent = task.agent.clone().filter(|a| !a.is_empty());
    let window = match (rec.ends_at.as_deref(), rec.max_runs) {
        (Some(end), Some(max)) => Some(format!(
            "until {}, at most {max} runs",
            format_time(Some(end))
        )),
        (Some(end), None) => Some(format!("until {}", format_time(Some(end)))),
        (None, Some(max)) => Some(format!("at most {max} runs")),
        (None, None) => None,
    };
    let runs = match (s.runs_count, s.missed_runs) {
        (Some(r), Some(m)) if m > 0 => format!("{r} ({m} missed)"),
        (Some(r), _) => r.to_string(),
        (None, _) => "—".to_owned(),
    };
    let last_state = s.last_fire_state.clone().unwrap_or_default();
    ScheduleRowView {
        short_id: short_id(&s.schedule_id),
        schedule_id: s.schedule_id.clone(),
        is_prompt: agent.is_none(),
        agent: agent.unwrap_or_else(|| "free prompt".to_owned()),
        scope: task.scope.filter(|x| !x.is_empty()),
        tenant: tenant_label(row_tenant(s.tenant.as_deref(), filter).as_deref()),
        owner: opt_text(s.owner.as_deref()),
        cron: opt_text(Some(rec.cron.as_str())),
        timezone: opt_text(rec.timezone.as_deref()),
        window,
        overlap: opt_text(s.overlap.as_deref()),
        state_tone: state_tone(&s.state),
        state: s.state.clone(),
        next_fire: format_time(s.next_fire_at.as_deref()),
        last_fired: format_time(s.last_fired_at.as_deref()),
        last_tone: last_fire_tone(&last_state),
        last_state: if last_state.is_empty() {
            "never fired".to_owned()
        } else {
            last_state
        },
        last_error: s.last_fire_error.clone().filter(|e| !e.is_empty()),
        runs,
    }
}

pub fn schedules_view(
    tenant: &str,
    tenants: &[TenantRef],
    schedules: Option<&[ScheduleOut]>,
) -> SchedulesPageContext {
    let rows: Vec<ScheduleRowView> = schedules
        .map(|l| l.iter().map(|s| schedule_row(s, tenant)).collect())
        .unwrap_or_default();
    let count_label = match rows.len() {
        1 => "1 schedule".to_owned(),
        n => format!("{n} schedules"),
    };
    SchedulesPageContext {
        page: "farm-schedules",
        title: "Schedules",
        breadcrumbs: vec![
            BreadcrumbView::link("Farm", "/admin/farm"),
            BreadcrumbView::current("Schedules"),
        ],
        select: tenant_select(URL, tenants, tenant),
        has_rows: !rows.is_empty(),
        rows,
        loaded: schedules.is_some(),
        count_label,
    }
}

pub async fn schedules_page(
    shell: Page,
    Extension(api): Extension<FarmApi>,
    Query(query): Query<TenantQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let mut s = scope(&api, &query).await;
    let list = if s.ready {
        match api.list_schedules(&s.tenant).await {
            Ok(list) => Some(list.items),
            Err(e) => {
                s.faults.record("Schedules", &e);
                None
            },
        }
    } else {
        None
    };
    let ctx = schedules_view(&s.tenant, &s.tenants, list.as_deref());
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-schedules",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &s.faults,
    ))
}
