//! Resource-wide retained metrics; current catalog placement is not a
//! historical dimension.

use serde::Serialize;
use systemprompt::analytics::snapshots::FeedbackSnapshot;

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotFigures {
    pub invocations: i64,
    pub distinct_users: Option<i64>,
    pub conversations: Option<i64>,
    pub linked_requests: i64,
    pub failed_requests: i64,
    pub priced_requests: i64,
    pub latency_measured: i64,
    pub token_measured: i64,
    pub assessed_conversations: i64,
    pub assessment_conversations: i64,
    pub failed_assessments: i64,
}
impl SnapshotFigures {
    pub fn from_snapshot(s: &FeedbackSnapshot) -> Option<Self> {
        (s.suppressed_days == 0).then_some(Self {
            invocations: s.metrics.invocations,
            distinct_users: s.distinct_users,
            conversations: s.distinct_sessions,
            linked_requests: s.metrics.requests,
            failed_requests: s.metrics.failed_requests,
            priced_requests: s.metrics.priced_requests,
            latency_measured: s.metrics.latency_measured_requests,
            token_measured: s.metrics.token_measured_requests,
            assessed_conversations: s.metrics.assessed_conversations,
            assessment_conversations: s.metrics.assessment_conversations,
            failed_assessments: s.metrics.failed_assessments,
        })
    }
}
pub fn cost(s: &FeedbackSnapshot) -> String {
    if s.suppressed_days > 0 || s.metrics.priced_requests == 0 || s.spend_by_currency.is_empty() {
        return "Unknown".to_owned();
    }
    s.spend_by_currency
        .iter()
        .map(|(currency, micros)| format!("{currency} {micros} micro-units"))
        .collect::<Vec<_>>()
        .join(", ")
}
