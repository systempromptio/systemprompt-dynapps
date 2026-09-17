//! View models for the evaluation pages: a criterion as the judge is
//! configured, and a run as the table and its drilldown show it.

use serde::Serialize;

use crate::handlers::ssr::entity_urls::request_detail_url;
use crate::handlers::ssr::format::{format_cost, format_duration_ms, local_time, short_id};
use crate::repositories::evaluation::runs::{EvaluationRunDetail, EvaluationRunRow};
use crate::repositories::evaluation::{LoadedCriterion, Scoring};
use crate::services::evaluation::{judge_output_schema, prompt};
use systemprompt::identifiers::{AiRequestId, SessionId, UserId};

// Why: judge scores are 0-100 and human ratings 1-5 (mapped ×20); both land
// on the same three tones so every score on the console reads on one scale.
#[must_use]
pub(crate) const fn score_tone(score: i16) -> &'static str {
    match score {
        s if s >= 80 => "ok",
        s if s >= 50 => "warn",
        _ => "err",
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct CriterionView {
    pub name: String,
    pub scoring: &'static str,
    pub is_metric: bool,
    pub provider: String,
    pub model: String,
    pub max_output_tokens: u32,
    pub instructions_path: String,
    pub inputs: Vec<&'static str>,
    pub thresholds: Vec<ThresholdView>,
    pub system_prompt: String,
    pub schema_json: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ThresholdView {
    pub label: &'static str,
    pub value: String,
}

impl CriterionView {
    pub(crate) fn of(criterion: &LoadedCriterion) -> Self {
        let schema = judge_output_schema(criterion.name, criterion.scoring);
        Self {
            name: criterion.name.to_string(),
            scoring: criterion.scoring.as_str(),
            is_metric: criterion.scoring == Scoring::Metric,
            provider: criterion.provider.clone(),
            model: criterion.model.clone(),
            max_output_tokens: criterion.max_output_tokens,
            instructions_path: format!("services/evaluation/{}", criterion.instructions_path),
            inputs: criterion.inputs.iter().map(|i| i.as_str()).collect(),
            thresholds: criterion
                .metric
                .map(|m| {
                    m.describe()
                        .into_iter()
                        .map(|(label, value)| ThresholdView { label, value })
                        .collect()
                })
                .unwrap_or_default(),
            system_prompt: prompt::build_system_prompt(criterion),
            // Why: discard-ok: a JSON value always serialises
            schema_json: serde_json::to_string_pretty(&schema).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct RunView {
    pub id: String,
    pub id_short: String,
    pub url: String,
    pub session_id: SessionId,
    pub session_short: String,
    pub session_url: String,
    pub user_id: UserId,
    pub host: String,
    pub criterion: String,
    pub attempt: i32,
    pub status: String,
    pub status_tone: &'static str,
    pub score_source: String,
    pub score_display: String,
    pub score_tone: &'static str,
    pub model: String,
    pub provider: String,
    pub tokens: String,
    pub cost: String,
    pub latency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub trigger: String,
    pub is_manual: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_by: Option<UserId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_by_url: Option<String>,
    pub created_at_local: String,
}

impl RunView {
    pub(crate) fn of(row: &EvaluationRunRow) -> Self {
        Self {
            id: row.id.clone(),
            id_short: short_id(&row.id),
            url: format!(
                "/admin/analysis/evaluation/runs/{}",
                urlencoding::encode(&row.id)
            ),
            session_short: short_id(row.session_id.as_str()),
            session_url: format!(
                "/admin/sessions/{}",
                urlencoding::encode(row.session_id.as_str())
            ),
            session_id: row.session_id.clone(),
            user_id: row.user_id.clone(),
            host: row.host.clone(),
            criterion: row.criterion.clone(),
            attempt: row.attempt,
            status_tone: if row.status == "succeeded" {
                "ok"
            } else {
                "err"
            },
            status: row.status.clone(),
            score_source: row.score_source.clone(),
            score_display: row.score.map_or_else(|| "—".to_owned(), |s| s.to_string()),
            score_tone: row.score.map_or("muted", score_tone),
            model: row.model.clone(),
            provider: row.provider.clone(),
            tokens: match (row.input_tokens, row.output_tokens) {
                (Some(i), Some(o)) => format!("{i} / {o}"),
                _ => "—".to_owned(),
            },
            cost: row
                .cost_microdollars
                .map_or_else(|| "—".to_owned(), format_cost),
            latency: row
                .latency_ms
                .map_or_else(|| "—".to_owned(), |ms| format_duration_ms(i64::from(ms))),
            error: row.error.clone().filter(|e| !e.is_empty()),
            trigger: row.trigger.clone(),
            is_manual: row.trigger == "manual",
            requested_by_url: row
                .requested_by
                .as_ref()
                .map(|u| format!("/admin/users/{}", urlencoding::encode(u.as_str()))),
            requested_by: row.requested_by.clone(),
            created_at_local: local_time(row.created_at),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct RunEvidenceView {
    pub quote: String,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct RunDetailView {
    pub run: RunView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
    pub evidence: Vec<RunEvidenceView>,
    pub has_evidence: bool,
    pub metrics_json: String,
    pub has_metrics: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_url: Option<String>,
    pub system_prompt: String,
    pub user_prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    pub config_hash: String,
    pub config_hash_short: String,
}

impl RunDetailView {
    pub(crate) fn of(detail: &EvaluationRunDetail) -> Self {
        let evidence: Vec<RunEvidenceView> = detail
            .evidence
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .map(|item| RunEvidenceView {
                        quote: item["quote"].as_str().unwrap_or_default().to_owned(),
                        note: item["note"].as_str().unwrap_or_default().to_owned(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let has_metrics = detail.metrics.as_object().is_some_and(|m| !m.is_empty());
        Self {
            run: RunView::of(&detail.row),
            rationale: detail.rationale.clone().filter(|r| !r.is_empty()),
            has_evidence: !evidence.is_empty(),
            evidence,
            // Why: discard-ok: a JSON value always serialises
            metrics_json: serde_json::to_string_pretty(&detail.metrics).unwrap_or_default(),
            has_metrics,
            request_url: detail
                .ai_request_id
                .as_deref()
                .map(|id| request_detail_url(&AiRequestId::new(id))),
            system_prompt: detail.system_prompt.clone(),
            user_prompt: detail.user_prompt.clone(),
            response: detail.response.clone(),
            config_hash_short: short_id(&detail.config_hash),
            config_hash: detail.config_hash.clone(),
        }
    }
}
