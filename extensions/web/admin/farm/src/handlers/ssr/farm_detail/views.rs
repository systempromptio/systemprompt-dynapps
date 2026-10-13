//! The rows each farm-page tab renders: the farm's agents, its eval suites
//! and its recent eval runs. Runs reuse the shared run row.

use serde::Serialize;
use systemprompt_web_admin_core::ssr::format::{format_cost, local_time, relative_time};

use crate::handlers::ssr::shared::BadgeView;
use crate::handlers::ssr::sync_card::short_sha;
use crate::repositories::rows::{AgentRow, EvalRunRow, EvalSuiteRow};
use crate::types::{EvalRunStatus, FarmAgentId};

#[derive(Debug, Serialize)]
pub(super) struct AgentView {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) description: String,
    pub(super) url: String,
    pub(super) model: String,
    pub(super) tool_count: usize,
    pub(super) skills: String,
    pub(super) synced_sha_short: String,
    pub(super) is_stale: bool,
}

pub(super) fn agent_view(agent: &AgentRow, farm_sha: Option<&str>) -> AgentView {
    AgentView {
        id: agent.id.as_str().to_owned(),
        name: agent.name.clone(),
        description: agent.description.clone(),
        url: format!(
            "/admin/farm/{}/agents/{}",
            agent.farm_id.as_str(),
            agent.id.as_str()
        ),
        model: agent
            .model
            .clone()
            .unwrap_or_else(|| "farm default".to_owned()),
        tool_count: agent.tools.len(),
        skills: agent.skills.join(", "),
        synced_sha_short: short_sha(&agent.synced_commit_sha),
        is_stale: farm_sha.is_some_and(|sha| sha != agent.synced_commit_sha),
    }
}

#[derive(Debug, Serialize)]
pub(super) struct FarmSuiteTabView {
    pub(super) id: String,
    pub(super) agent_id: FarmAgentId,
    pub(super) agent_url: String,
    pub(super) case_count: i32,
    pub(super) source_path: String,
    pub(super) synced_sha_short: String,
}

pub(super) fn suite_view(suite: &EvalSuiteRow) -> FarmSuiteTabView {
    FarmSuiteTabView {
        id: suite.id.as_str().to_owned(),
        agent_id: suite.agent_id.clone(),
        agent_url: format!(
            "/admin/farm/{}/agents/{}",
            suite.farm_id.as_str(),
            suite.agent_id.as_str()
        ),
        case_count: suite.case_count,
        source_path: suite.source_path.clone(),
        synced_sha_short: short_sha(&suite.synced_commit_sha),
    }
}

#[derive(Debug, Serialize)]
pub(super) struct EvalRunView {
    pub(super) id: String,
    pub(super) url: String,
    pub(super) agent_id: FarmAgentId,
    pub(super) status: BadgeView,
    pub(super) passed: i32,
    pub(super) failed: i32,
    pub(super) errored: i32,
    pub(super) score: String,
    pub(super) cost_display: String,
    pub(super) created_local: String,
    pub(super) created_relative: String,
}

fn eval_status_badge(status: &str) -> BadgeView {
    let (tone, text) = match EvalRunStatus::parse(status) {
        Some(EvalRunStatus::Queued) => ("muted", "Queued"),
        Some(EvalRunStatus::Running) => ("accent", "Running"),
        Some(EvalRunStatus::Completed) => ("ok", "Completed"),
        Some(EvalRunStatus::Failed) => ("err", "Failed"),
        Some(EvalRunStatus::Cancelled) => ("muted", "Cancelled"),
        None => ("muted", "Unknown"),
    };
    BadgeView::new(tone, text, status)
}

pub(super) fn eval_run_view(run: &EvalRunRow) -> EvalRunView {
    EvalRunView {
        id: run.id.as_str().to_owned(),
        url: format!("/admin/farm/evals/runs/{}", run.id.as_str()),
        agent_id: run.agent_id.clone(),
        status: eval_status_badge(&run.status),
        passed: run.passed,
        failed: run.failed,
        errored: run.errored,
        score: run
            .mean_score
            .map_or_else(|| "—".to_owned(), |s| format!("{s:.2}")),
        cost_display: format_cost(run.cost_microdollars),
        created_local: local_time(run.created_at),
        created_relative: relative_time(run.created_at),
    }
}
