//! The page's query string and what it is read into: the decision and
//! finding filters, the sort, and the time window.

use serde::Deserialize;

use crate::repositories::governance::decision_log::{DecisionFilter, DecisionSort};
use crate::repositories::governance::findings::FindingFilter;
use systemprompt_web_admin_core::util::time_range::{TimeRange, TimeRangeQuery, parse_time_range};

#[derive(Debug, Default, Deserialize)]
pub(crate) struct GovernanceQuery {
    pub tab: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
    pub policy: Option<String>,
    pub decision: Option<String>,
    pub category: Option<String>,
    pub blocked: Option<String>,
    pub q: Option<String>,
    pub attention: Option<String>,
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
}

pub(super) fn sort_from(query: &GovernanceQuery) -> DecisionSort {
    let key = match query.sort.as_deref() {
        Some("policy") => "policy",
        Some("decision") => "decision",
        Some("tool") => "tool",
        Some("user") => "user",
        _ => "when",
    };
    DecisionSort {
        key,
        ascending: query.dir.as_deref() == Some("asc"),
    }
}

pub(super) fn decision_filter(query: &GovernanceQuery) -> DecisionFilter {
    DecisionFilter {
        policy: non_empty(query.policy.as_deref()),
        decision: non_empty(query.decision.as_deref()),
        search: non_empty(query.q.as_deref()),
        attention: attention_on(query),
    }
}

pub(super) fn attention_on(query: &GovernanceQuery) -> bool {
    non_empty(query.attention.as_deref()).is_some_and(|v| v != "0")
}

pub(super) fn finding_filter(query: &GovernanceQuery) -> FindingFilter {
    FindingFilter {
        category: non_empty(query.category.as_deref()),
        blocked: match query.blocked.as_deref() {
            Some("blocked") => Some(true),
            Some("audited") => Some(false),
            _ => None,
        },
        search: non_empty(query.q.as_deref()),
    }
}

fn non_empty(value: Option<&str>) -> Option<String> {
    value.filter(|v| !v.trim().is_empty()).map(str::to_owned)
}

pub(super) fn range_of(query: &GovernanceQuery) -> TimeRange {
    parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    })
}
