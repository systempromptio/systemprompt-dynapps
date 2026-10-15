//! The Governance tab: every `governance_decisions` row the run produced,
//! joined on its gateway and host sessions or its conversation context, with
//! a count per decision so a refused call stands out without scrolling.

use serde::Serialize;
use systemprompt_web_admin_core::ssr::format::local_time;

use super::RunData;
use crate::handlers::ssr::shared::BadgeView;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DecisionView {
    pub url: String,
    pub when: String,
    pub tool_name: String,
    pub decision: BadgeView,
    pub policy: String,
    pub reason: String,
    pub actor: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RunGovernanceView {
    pub rows: Vec<DecisionView>,
    pub has_rows: bool,
    pub total: usize,
    pub allowed: usize,
    pub warned: usize,
    pub denied: usize,
}

fn decision_badge(decision: &str) -> BadgeView {
    let (tone, text) = match decision {
        "allow" => ("ok", "Allow"),
        "warn" => ("warn", "Warn"),
        "deny" => ("err", "Deny"),
        "pending" => ("info", "Pending"),
        _ => ("muted", decision),
    };
    BadgeView::new(tone, text, decision)
}

pub(crate) fn governance_view(data: &RunData) -> RunGovernanceView {
    let count = |d: &str| data.decisions.iter().filter(|r| r.decision == d).count();
    let rows: Vec<DecisionView> = data
        .decisions
        .iter()
        .map(|r| DecisionView {
            url: format!("/admin/governance/decisions/{}", r.id),
            when: local_time(r.created_at),
            tool_name: r.tool_name.clone(),
            decision: decision_badge(&r.decision),
            policy: r.policy.clone(),
            reason: r.reason.clone(),
            actor: format!("{}:{}", r.actor_kind, r.actor_id),
        })
        .collect();
    RunGovernanceView {
        has_rows: !rows.is_empty(),
        total: rows.len(),
        allowed: count("allow"),
        warned: count("warn"),
        denied: count("deny"),
        rows,
    }
}
