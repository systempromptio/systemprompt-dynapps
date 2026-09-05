//! The governance KPI strip.
//!
//! Nine tiles spanning both enforcement planes, because the whole reason the
//! three tabs share one page is that the chain and the scanners are configured
//! apart and have to be read together. Every tile is also the filter its number
//! describes, so a number and the rows behind it are one click apart and cannot
//! disagree.

use serde::Serialize;

use super::GovernanceQuery;
use super::data::GovernanceData;
use super::urls::filter_url;

// Why: One KPI tile, as `components/kpi` reads it.
#[derive(Debug, Serialize)]
pub(super) struct KpiView {
    label: &'static str,
    value: String,
    sub: String,
    tone: &'static str,
    href: String,
    active: bool,
    hint: &'static str,
}

// Why: nine tiles, and the four in the middle are the four chain stages in
// evaluation order. Reading them left to right is reading the chain, which is
// the only ordering that lets an operator see which stage is doing the work.
pub(super) fn kpis(query: &GovernanceQuery, data: &GovernanceData) -> Vec<KpiView> {
    let s = data.stats;
    let f = data.safety;
    let rate = percent(s.denied, s.evaluated);
    vec![
        tile(
            "Evaluated",
            s.evaluated,
            format!("{} callers", s.distinct_users),
            "",
            query,
            None,
            "Tool calls the chain decided in this window",
        ),
        tile(
            "Denied",
            s.denied,
            format!("{rate} of evaluated"),
            "err",
            query,
            Some(("decision", "deny")),
            "Calls the chain refused outright",
        ),
        tile(
            "Warned",
            s.warned,
            "would have been refused".to_owned(),
            "warn",
            query,
            Some(("decision", "warn")),
            "Warn mode absorbed these; enforcement would have refused them",
        ),
        tile(
            "Scope denies",
            s.scope_denied,
            "stage 1".to_owned(),
            "",
            query,
            Some(("policy", "agent_scope")),
            "Tool outside the scope granted to the agent",
        ),
        tile(
            "Secret denies",
            s.secret_denied,
            "stage 2".to_owned(),
            "",
            query,
            Some(("policy", "secret_scan")),
            "A credential pattern matched the tool input",
        ),
        tile(
            "Blocklist denies",
            s.blocklist_denied,
            "stage 3".to_owned(),
            "",
            query,
            Some(("policy", "tool_blocklist")),
            "Tool is blocked for this principal by policy",
        ),
        tile(
            "Rate denies",
            s.rate_denied,
            "stage 4".to_owned(),
            "",
            query,
            Some(("policy", "rate_limit")),
            "Call budget for the session was exhausted",
        ),
        tile(
            "Safety findings",
            f.findings,
            format!("{} categories", f.categories),
            "",
            query,
            Some(("tab", "safety")),
            "Scanner findings in both directions",
        ),
        tile(
            "Findings blocked",
            f.blocked,
            format!("{} audited only", f.audited),
            "err",
            query,
            Some(("tab", "safety")),
            "Findings with no blocks means the scanners are in warn mode",
        ),
    ]
}

fn tile(
    label: &'static str,
    value: i64,
    sub: String,
    tone: &'static str,
    query: &GovernanceQuery,
    filter: Option<(&'static str, &'static str)>,
    hint: &'static str,
) -> KpiView {
    let (href, active) = filter.map_or_else(
        || (filter_url(query, &[]), false),
        |(name, value)| {
            let current = match name {
                "decision" => query.decision.as_deref(),
                "policy" => query.policy.as_deref(),
                _ => query.tab.as_deref(),
            };
            (filter_url(query, &[(name, value)]), current == Some(value))
        },
    );
    KpiView {
        label,
        value: value.to_string(),
        sub,
        tone,
        href,
        active,
        hint,
    }
}

fn percent(part: i64, whole: i64) -> String {
    if whole <= 0 {
        return "0%".to_owned();
    }
    format!("{:.1}%", (part as f64 / whole as f64) * 100.0_f64)
}
