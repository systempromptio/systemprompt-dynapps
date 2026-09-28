//! The skill detail page's KPI tiles: five usage tiles (invocations, people,
//! conversations, tokens, cost) then four health tiles (tools, errors, p95, AI
//! score).

use crate::handlers::ssr::analysis::skill_detail::views::SkillKpiView;
use crate::handlers::ssr::analysis::tone::{
    completion_tone, deny_tone, error_rate_tone, latency_tone, percent, score_display,
};
use crate::repositories::analysis::skills::SkillFactRow;
use systemprompt_web_admin_core::ssr::format::{
    format_cost, format_duration_ms, format_token_total,
};

fn usage_kpis(f: &SkillFactRow, entitled: usize) -> Vec<SkillKpiView> {
    let kpi = |label, icon, value, note, tone, hint| SkillKpiView {
        label,
        icon,
        value,
        note,
        tone,
        hint,
    };
    let tokens = f.input_tokens + f.output_tokens;
    vec![
        kpi(
            "Invocations",
            "bolt",
            f.invocations.to_string(),
            format!(
                "{} slash · {} tool · {} attributed",
                f.slash,
                f.tool,
                percent(f.attributed, f.invocations)
            ),
            "accent",
            "Hook-reported invocations in the window",
        ),
        kpi(
            "People",
            "people",
            f.users.to_string(),
            format!("of {entitled} entitled · {} installed", f.installs),
            "accent",
            "Distinct people who invoked the skill; entitlement from the access-control rules, installs from verified receipts",
        ),
        kpi(
            "Conversations",
            "chat",
            f.conversations.to_string(),
            format!("{} turns · {} sessions", f.turns, f.sessions),
            "accent",
            "Gateway conversations whose harness session invoked the skill",
        ),
        kpi(
            "Tokens",
            "token",
            format_token_total(tokens),
            format!(
                "{} in · {} out · {} cached",
                format_token_total(f.input_tokens),
                format_token_total(f.output_tokens),
                format_token_total(f.cache_tokens)
            ),
            "accent",
            "Tokens those conversations ran up",
        ),
        kpi(
            "Cost",
            "coins",
            format_cost(f.cost_microdollars),
            format!(
                "{} per invocation · {} per conversation",
                format_cost(f.cost_microdollars / f.invocations.max(1)),
                format_cost(f.cost_microdollars / f.conversations.max(1))
            ),
            "accent",
            "Priced spend of those conversations",
        ),
    ]
}

fn health_kpis(f: &SkillFactRow) -> Vec<SkillKpiView> {
    let kpi = |label, icon, value, note, tone, hint| SkillKpiView {
        label,
        icon,
        value,
        note,
        tone,
        hint,
    };
    vec![
        kpi(
            "Tools",
            "wrench",
            f.tool_calls.to_string(),
            format!("{} failed · {} artifacts", f.tool_calls_failed, f.artifacts),
            if f.tool_calls_failed > 0 {
                "err"
            } else {
                "accent"
            },
            "Tool calls the model asked for inside those conversations",
        ),
        kpi(
            "Errors",
            "alert",
            f.errors.to_string(),
            format!("failed requests · {} denied", f.denied),
            if f.denied > 0 {
                deny_tone(f.denied)
            } else {
                error_rate_tone(f.errors, f.requests)
            },
            "Failed requests and governance denials",
        ),
        kpi(
            "p95 latency",
            "gauge",
            f.p95_latency_ms.map_or_else(
                || "—".to_owned(),
                |ms| format_duration_ms(ms.round() as i64),
            ),
            "over the conversations' own p95".to_owned(),
            latency_tone(f.p95_latency_ms),
            "95th percentile of turn latency",
        ),
        kpi(
            "AI score",
            "sparkle",
            score_display(f.completion_avg),
            format!(
                "{} judged · {} achieved",
                f.judged,
                percent(f.achieved, f.conversations.max(1))
            ),
            completion_tone(f.completion_avg),
            "Mean of the judge's one completion score over judged conversations that used this skill",
        ),
    ]
}

pub(in crate::handlers::ssr::analysis::skill_detail) fn kpis(
    f: Option<&SkillFactRow>,
    entitled: usize,
) -> Vec<SkillKpiView> {
    let Some(f) = f else {
        return Vec::new();
    };
    let mut tiles = usage_kpis(f, entitled);
    tiles.extend(health_kpis(f));
    tiles
}
