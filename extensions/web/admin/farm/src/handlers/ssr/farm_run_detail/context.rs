//! The page context: header, identifiers, actions, the live block the
//! fragment re-renders, the identity badges, and the six tabs.

use serde::Serialize;
use systemprompt::identifiers::{ContextId, UserId};
use systemprompt_web_admin_core::ssr::entity_urls::context_detail_url;
use systemprompt_web_admin_core::ssr::format::{local_time, relative_time, short_id};
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use super::RunData;
use super::artifacts::{RunArtifactsView, artifacts_view};
use super::cost::{RunCostView, cost_view};
use super::governance::{RunGovernanceView, governance_view};
use super::report::{IdentityView, RunReportView, identity_view, report_view};
use super::summary::{RunSummaryView, run_duration_ms, run_summary};
use crate::handlers::ssr::run_events_view::{
    RunEventView, RunTimelineView, timeline_view, transcript_views,
};
use crate::handlers::ssr::shared::{BadgeView, status_badge, trigger_badge};

#[derive(Debug, Serialize)]
pub(crate) struct RunDetailContext {
    pub page: &'static str,
    pub title: String,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub header: RunHeaderView,
    pub live: RunLiveView,
    pub transcript: Vec<RunEventView>,
    pub cost: RunCostView,
    pub artifacts: RunArtifactsView,
    pub report: RunReportView,
    pub governance: RunGovernanceView,
    pub identity: IdentityView,
}

#[derive(Debug, Serialize)]
pub(crate) struct RunHeaderView {
    pub run_id: String,
    pub run_id_short: String,
    pub poll_url: String,
    pub cancel_url: String,
    pub rerun_url: String,
    pub agent_label: String,
    pub agent_url: String,
    pub farm_id: String,
    pub farm_url: String,
    pub user_id: UserId,
    pub user_url: String,
    pub trigger: BadgeView,
    pub started_local: String,
    pub started_relative: String,
    pub model: String,
    pub prompt: String,
    pub permission_mode: String,
    pub limits: String,
    pub source_commit: Option<String>,
    pub context_id: Option<ContextId>,
    pub context_url: Option<String>,
    pub gateway_session_id: Option<String>,
    pub session_url: Option<String>,
    pub host: String,
    pub host_session_id: Option<String>,
    pub attempt: i32,
    pub claimed_by: Option<String>,
    pub lease_until_local: Option<String>,
    pub error_class: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct RunLiveView {
    pub status: BadgeView,
    pub status_key: String,
    pub is_live: bool,
    pub last_seq: i64,
    pub summary: RunSummaryView,
    pub timeline: RunTimelineView,
    pub events: Vec<RunEventView>,
}

pub(crate) fn live_view(data: &RunData, events: Vec<RunEventView>) -> RunLiveView {
    let summary = run_summary(data);
    RunLiveView {
        status: status_badge(&data.run.status),
        status_key: data.run.status.clone(),
        is_live: summary.is_live,
        last_seq: data.events.last().map_or(0, |e| e.seq),
        timeline: timeline_view(&data.events, run_duration_ms(data)),
        summary,
        events,
    }
}

pub(crate) fn detail_context(data: &RunData) -> RunDetailContext {
    let short = short_id(data.run.id.as_str());
    let header = header_view(data);
    RunDetailContext {
        page: "farm-run-detail",
        title: format!("Run · {short}"),
        breadcrumbs: vec![
            BreadcrumbView::link("Agent farms", "/admin/farm"),
            BreadcrumbView::link("Runs", "/admin/farm/runs"),
            BreadcrumbView::current(short),
        ],
        transcript: transcript_views(&data.events),
        cost: cost_view(&data.requests),
        artifacts: artifacts_view(&data.run, &data.agent_run),
        report: report_view(data),
        governance: governance_view(data),
        identity: identity_view(data),
        live: live_view(data, data.events.clone()),
        header,
    }
}

fn header_view(data: &RunData) -> RunHeaderView {
    let run = &data.run;
    let agent_run = &data.agent_run;
    let agent_name = data.agent_name.as_deref();
    let context = data.context_id.as_ref();
    let id = run.id.as_str();
    let agent_url = format!(
        "/admin/farm/{}/agents/{}",
        run.farm_id.as_str(),
        run.agent_id.as_str()
    );
    let started = run.started_at.unwrap_or(run.created_at);
    RunHeaderView {
        run_id: id.to_owned(),
        run_id_short: short_id(id),
        poll_url: format!("/admin/api/farm/runs/{id}"),
        cancel_url: format!("/admin/farm/runs/{id}/cancel"),
        rerun_url: format!("{agent_url}?launch=1&prompt={}", encode_query(&run.prompt)),
        agent_label: agent_name.unwrap_or(run.agent_id.as_str()).to_owned(),
        agent_url,
        farm_id: run.farm_id.as_str().to_owned(),
        farm_url: format!("/admin/farm/{}", run.farm_id.as_str()),
        user_id: run.user_id.clone(),
        user_url: format!("/admin/users/{}", run.user_id.as_str()),
        trigger: trigger_badge(&run.triggered_via),
        started_local: local_time(started),
        started_relative: relative_time(started),
        model: run
            .model
            .clone()
            .unwrap_or_else(|| "default model".to_owned()),
        prompt: run.prompt.clone(),
        permission_mode: run.permission_mode.clone(),
        limits: format!(
            "{} turns · ${:.2} · {} s",
            run.max_turns, run.max_budget_usd, run.timeout_seconds
        ),
        source_commit: run.source_commit_sha.clone(),
        context_id: context.cloned(),
        context_url: context.map(context_detail_url),
        gateway_session_id: agent_run
            .gateway_session_id
            .as_ref()
            .map(|s| s.as_str().to_owned()),
        session_url: agent_run
            .gateway_session_id
            .as_ref()
            .map(|s| format!("/admin/sessions/{}", s.as_str())),
        host: run.host.clone(),
        host_session_id: agent_run
            .host_session_id
            .as_ref()
            .map(|s| s.as_str().to_owned()),
        attempt: run.attempt,
        claimed_by: agent_run.claimed_by.clone(),
        lease_until_local: agent_run.lease_until.map(local_time),
        error_class: run
            .error_class
            .clone()
            .or_else(|| agent_run.error_class.clone()),
    }
}

const HEX: &[u8; 16] = b"0123456789ABCDEF";

pub(crate) fn encode_query(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(b));
        } else {
            out.push('%');
            out.push(char::from(HEX[usize::from(b >> 4)]));
            out.push(char::from(HEX[usize::from(b & 0x0F)]));
        }
    }
    out
}
