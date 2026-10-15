//! Table rows for the clients pages: one client, one agent's outcomes for a
//! client, and one recent run with its completion report and eval verdict.

use serde::Serialize;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::ssr::format::{local_time, relative_time, short_id};
use systemprompt_web_shared::format::format_cost;

use crate::handlers::ssr::shared::outcomes::{pct_display, rate_tone, score_display};
use crate::handlers::ssr::shared::{BadgeView, status_badge, trigger_badge};
use crate::repositories::clients::{AgentOutcomeRow, ClientSummaryRow, ReportRunRow};

#[derive(Debug, Serialize)]
pub(crate) struct ClientRowView {
    pub name: String,
    pub slug: String,
    pub url: String,
    pub runs: i64,
    pub share_pct: String,
    pub share_label: String,
    pub succeeded: i64,
    pub failed: i64,
    pub active: i64,
    pub people: i64,
    pub success: String,
    pub success_tone: &'static str,
    pub completion: String,
    pub eval: String,
    pub infra_failures: i64,
    pub cost: String,
    pub last_run: String,
    pub last_run_local: String,
}

fn ratio(part: i64, whole: i64) -> Option<f64> {
    (whole > 0).then(|| part as f64 * 100.0 / whole as f64)
}

pub(crate) fn client_rows(rows: &[ClientSummaryRow]) -> Vec<ClientRowView> {
    let total: i64 = rows.iter().map(|r| r.runs).sum();
    rows.iter()
        .map(|r| {
            let share = ratio(r.runs, total).unwrap_or(0.0);
            let success = ratio(r.succeeded, r.finished);
            ClientRowView {
                name: r.name.clone(),
                slug: r.slug.clone(),
                url: format!("/admin/farm/clients/{}", r.client_id),
                runs: r.runs,
                share_pct: format!("{share:.1}"),
                share_label: format!("{share:.0}%"),
                succeeded: r.succeeded,
                failed: r.finished - r.succeeded,
                active: r.active,
                people: r.people,
                success: pct_display(success),
                success_tone: badge_tone(success),
                completion: pct_display(ratio(r.completed, r.reported)),
                eval: score_display(r.eval_mean),
                infra_failures: r.infra_failures,
                cost: format_cost(r.cost_microdollars),
                last_run: r.last_run_at.map(relative_time).unwrap_or_default(),
                last_run_local: r.last_run_at.map(local_time).unwrap_or_default(),
            }
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub(crate) struct AgentOutcomeView {
    pub label: String,
    pub url: String,
    pub runs: i64,
    pub success: String,
    pub success_tone: &'static str,
    pub share_pct: String,
    pub completed: i64,
    pub eval: String,
    pub infra_failures: i64,
    pub cost: String,
    pub last_run: String,
}

pub(crate) fn agent_rows(rows: &[AgentOutcomeRow]) -> Vec<AgentOutcomeView> {
    rows.iter()
        .map(|r| {
            let success = ratio(r.succeeded, r.finished);
            AgentOutcomeView {
                label: format!("{} › {}", r.farm_id, r.agent_id),
                url: format!("/admin/farm/{}/agents/{}", r.farm_id, r.agent_id),
                runs: r.runs,
                success: pct_display(success),
                success_tone: badge_tone(success),
                share_pct: format!("{:.1}", success.unwrap_or(0.0)),
                completed: r.completed,
                eval: score_display(r.eval_mean),
                infra_failures: r.infra_failures,
                cost: format_cost(r.cost_microdollars),
                last_run: relative_time(r.last_run_at),
            }
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub(crate) struct ReportRunView {
    pub run_id_short: String,
    pub url: String,
    pub agent_label: String,
    pub user_id: UserId,
    pub status: BadgeView,
    pub trigger: BadgeView,
    pub completion: BadgeView,
    pub eval: BadgeView,
    pub summary: String,
    pub cost: String,
    pub created: String,
    pub created_local: String,
}

pub(crate) fn completion_badge(completed: Option<bool>) -> BadgeView {
    match completed {
        Some(true) => BadgeView::new("ok", "Completed", "completed"),
        Some(false) => BadgeView::new("err", "Incomplete", "incomplete"),
        None => BadgeView::new("muted", "No report", "none"),
    }
}

pub(crate) fn eval_badge(passed: Option<bool>, score: Option<f64>) -> BadgeView {
    let score = score.map(|s| format!(" · {s:.0}")).unwrap_or_default();
    match passed {
        Some(true) => BadgeView::new("ok", &format!("Passed{score}"), "passed"),
        Some(false) => BadgeView::new("err", &format!("Failed{score}"), "failed"),
        None => BadgeView::new("muted", "Not scored", "none"),
    }
}

pub(crate) fn report_run_rows(rows: &[ReportRunRow]) -> Vec<ReportRunView> {
    rows.iter()
        .map(|r| ReportRunView {
            run_id_short: short_id(r.run_id.as_str()),
            url: format!("/admin/farm/runs/{}", r.run_id),
            agent_label: format!("{} › {}", r.farm_id, r.agent_id),
            user_id: r.user_id.clone(),
            status: status_badge(&r.status),
            trigger: trigger_badge(&r.triggered_via),
            completion: completion_badge(r.completed),
            eval: eval_badge(r.eval_passed, r.eval_score),
            summary: r.summary.clone().unwrap_or_default(),
            cost: format_cost(r.cost_microdollars),
            created: relative_time(r.created_at),
            created_local: local_time(r.created_at),
        })
        .collect()
}

fn badge_tone(pct: Option<f64>) -> &'static str {
    match rate_tone(pct) {
        "" => "muted",
        tone => tone,
    }
}
