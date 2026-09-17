//! Typed template-context structs for the session-detail page
//! (`session-detail.hbs`).

use serde::Serialize;

pub(super) use crate::handlers::ssr::types::BreadcrumbView;
use systemprompt::identifiers::{AiRequestId, ContextId, PluginId, SessionId, TraceId, UserId};

#[derive(Debug, Serialize)]
pub(super) struct SessionDetailPageContext {
    pub(super) page: &'static str,
    pub(super) title: String,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    pub(super) header: SessionHeaderView,
    pub(super) kpis: SessionKpisView,
    pub(super) contexts: Vec<SessionContextRowView>,
    pub(super) traces: Vec<SessionTraceRowView>,
    pub(super) requests: Vec<SessionRequestRowView>,
    pub(super) has_contexts: bool,
    pub(super) has_traces: bool,
    pub(super) has_requests: bool,
    pub(super) back_url: &'static str,
    // Why: empty until the session has closed and the evaluation job has run,
    // which is the normal state for a session still in flight — the page says
    // so rather than rendering an empty verdict panel.
    pub(super) scores: Vec<EvaluationScoreView>,
    pub(super) has_scores: bool,
    pub(super) ratings: Vec<RatingView>,
    pub(super) has_ratings: bool,
    pub(super) rating_count: usize,
    pub(super) rating_average: Option<String>,
    pub(super) evaluation: EvaluationActionsView,
}

// Why: the buttons are only drawn when a click can actually run — the
// engine is configured and the process carries an inference client.
#[derive(Debug, Serialize)]
pub(super) struct EvaluationActionsView {
    pub(super) url: String,
    pub(super) enabled: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct EvaluationScoreView {
    pub(super) criterion: String,
    pub(super) score_display: String,
    pub(super) tone: &'static str,
    pub(super) score_source: String,
    pub(super) model: String,
    pub(super) run_url: String,
    pub(super) evaluated_at_local: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) rationale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) error: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct RatingView {
    pub(super) user_id: UserId,
    pub(super) user_label: String,
    pub(super) user_url: String,
    pub(super) rating: i16,
    pub(super) stars: String,
    pub(super) tone: &'static str,
    pub(super) outcome: String,
    pub(super) notes: String,
    pub(super) created_at_local: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionHeaderView {
    pub(super) session_id: SessionId,
    pub(super) session_id_short: String,
    pub(super) user_id: Option<UserId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) user_url: Option<String>,
    pub(super) display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) groups_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) started_at_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) last_activity_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) last_activity_at_local: Option<String>,
    pub(super) duration_display: String,
    pub(super) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) plugin_id: Option<PluginId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) ai_title: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionKpisView {
    pub(super) request_count: i64,
    pub(super) context_count: i64,
    pub(super) trace_count: i64,
    pub(super) error_count: i64,
    // Why: `total_input_tokens` is the uncached part only; `total_tokens`
    // is the wire total (input + cache read + cache creation + output).
    pub(super) total_input_tokens: i64,
    pub(super) total_cache_read_tokens: i64,
    pub(super) total_cache_creation_tokens: i64,
    pub(super) total_output_tokens: i64,
    pub(super) total_tokens: i64,
    pub(super) tokens_display: String,
    pub(super) tokens_note: String,
    pub(super) total_cost_microdollars: i64,
    pub(super) total_cost_display: String,
    pub(super) tool_call_count: i64,
    pub(super) executed_count: i64,
    pub(super) artifact_count: i64,
    pub(super) artifacts_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionContextRowView {
    pub(super) context_id: ContextId,
    pub(super) context_id_short: String,
    pub(super) context_url: String,
    pub(super) name: String,
    pub(super) request_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) last_request_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) last_request_at_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) model: Option<String>,
    pub(super) total_tokens: i64,
    pub(super) total_input_tokens: i64,
    pub(super) total_cache_read_tokens: i64,
    pub(super) total_cache_creation_tokens: i64,
    pub(super) total_output_tokens: i64,
    pub(super) cost_microdollars: i64,
    pub(super) token_display: String,
    pub(super) cost_display: String,
    pub(super) error_count: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionTraceRowView {
    pub(super) trace_id: TraceId,
    pub(super) trace_id_short: String,
    pub(super) trace_url: String,
    pub(super) request_count: i64,
    pub(super) error_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) started_at_local: Option<String>,
    pub(super) duration_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionRequestRowView {
    pub(super) id: AiRequestId,
    pub(super) id_short: String,
    pub(super) request_url: String,
    pub(super) context_id: Option<ContextId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) context_id_short: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) context_url: Option<String>,
    pub(super) trace_id: Option<TraceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) trace_id_short: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) trace_url: Option<String>,
    pub(super) model: String,
    pub(super) status: String,
    pub(super) is_error: bool,
    pub(super) is_rejected: bool,
    pub(super) latency_ms: Option<i32>,
    pub(super) input_tokens: Option<i32>,
    pub(super) cache_read_tokens: Option<i32>,
    pub(super) cache_creation_tokens: Option<i32>,
    pub(super) output_tokens: Option<i32>,
    pub(super) cost_microdollars: i64,
    pub(super) created_at: String,
    pub(super) latency_display: String,
    pub(super) cost_display: String,
    pub(super) created_at_local: String,
}
