//! Display figures read from one retained feedback snapshot.
//!
//! Every number comes from `analytics_feedback_snapshots`; nothing is
//! recomputed from raw events. A snapshot with suppressed days is treated as
//! absent so a privacy-suppressed cohort never leaks a partial.
use serde::Serialize;
use systemprompt::analytics::snapshots::FeedbackSnapshot;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct SnapshotFigures {
    pub measured: bool,
    pub invocations: i64,
    pub verified_invocations: i64,
    pub attributed_pct: Option<i64>,
    pub distinct_users: Option<i64>,
    pub distinct_sessions: Option<i64>,
    pub requests: i64,
    pub failed_requests: i64,
    pub priced_requests: i64,
    pub latency_measured_requests: i64,
    pub token_measured_requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_microdollars: i64,
    pub assessed_conversations: i64,
    pub assessment_conversations: i64,
    pub failed_assessments: i64,
    pub p95_micros: Option<u64>,
    pub generation: i64,
}

impl SnapshotFigures {
    pub fn of(
        snapshot: &FeedbackSnapshot,
    ) -> Result<Option<Self>, systemprompt::analytics::AnalyticsError> {
        if snapshot.suppressed_days > 0 {
            return Ok(None);
        }
        let m = &snapshot.metrics;
        let clamp = |v: i128| i64::try_from(v).unwrap_or(i64::MAX);
        Ok(Some(Self {
            measured: true,
            invocations: m.invocations,
            verified_invocations: m.verified_invocations,
            attributed_pct: (m.invocations > 0)
                .then(|| m.verified_invocations * 100 / m.invocations),
            distinct_users: snapshot.distinct_users,
            distinct_sessions: snapshot.distinct_sessions,
            requests: m.requests,
            failed_requests: m.failed_requests,
            priced_requests: m.priced_requests,
            latency_measured_requests: m.latency_measured_requests,
            token_measured_requests: m.token_measured_requests,
            input_tokens: clamp(m.input_tokens),
            output_tokens: clamp(m.output_tokens),
            cost_microdollars: snapshot
                .spend_by_currency
                .get("USD")
                .map_or(0, |micros| clamp(*micros)),
            assessed_conversations: m.assessed_conversations,
            assessment_conversations: m.assessment_conversations,
            failed_assessments: m.failed_assessments,
            p95_micros: snapshot.histogram.percentile_upper_bound_micros(95)?,
            generation: snapshot.generation,
        }))
    }
}
