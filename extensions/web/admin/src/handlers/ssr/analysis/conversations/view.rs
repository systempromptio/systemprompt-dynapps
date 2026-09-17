//! Template rows for `/admin/analysis/conversations`: one classified
//! conversation shaped for Handlebars, with every number pre-formatted, and
//! the page context that carries them.

use crate::handlers::ssr::analysis_urls::analysis_skill_url;
use crate::handlers::ssr::entity_urls::{session_detail_url, session_evaluate_url};
use crate::handlers::ssr::scorecard::{ScorecardView, Scores};
use serde::Serialize;
use systemprompt::identifiers::SessionId;

use super::summary::{
    ConversationAnalysisFilterView, ConversationBucketView, KpisView, category_label,
    outcome_label, outcome_tone,
};
use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::handlers::ssr::list_view::{Pagination, ScopeFilterView, SelectOptionView};
use crate::handlers::ssr::types::{SortHeaderView, TabLinkView};
use crate::repositories::analysis::conversations::ConversationAnalysisRow;

#[derive(Debug, Serialize)]
pub(super) struct SkillChipView {
    pub name: String,
    pub href: String,
    // Why: a skill the harness hook reported is evidence; one the judge
    // inferred from the transcript alone is a reading.
    pub hooked: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct ConversationRowView {
    pub context_key: String,
    pub href: String,
    pub session_href: Option<String>,
    pub evaluate_url: Option<String>,
    pub title: String,
    pub user_key: String,
    pub user_href: Option<String>,
    pub display_name: String,
    pub group_name: Option<String>,
    pub project_name: Option<String>,
    pub client_kind: String,
    pub model: Option<String>,
    pub category: String,
    pub category_label: &'static str,
    pub summary: String,
    pub tags: Vec<String>,
    pub outcome: &'static str,
    pub outcome_tone: &'static str,
    pub skills: Vec<SkillChipView>,
    pub scorecard: ScorecardView,
    pub confidence_display: String,
    pub turns: i64,
    pub tool_calls: i64,
    pub errors: i64,
    pub tokens_display: String,
    pub cost_display: String,
    pub last_display: String,
    pub last_full: String,
}

impl ConversationRowView {
    pub(super) fn new(row: &ConversationAnalysisRow) -> Self {
        let mut skills: Vec<SkillChipView> = row
            .skills_hooked
            .iter()
            .map(|s| SkillChipView {
                href: analysis_skill_url(s),
                name: s.clone(),
                hooked: true,
            })
            .collect();
        for s in &row.skills_used {
            if !skills.iter().any(|k| k.name.eq_ignore_ascii_case(s)) {
                skills.push(SkillChipView {
                    href: analysis_skill_url(s),
                    name: s.clone(),
                    hooked: false,
                });
            }
        }
        let user_id = row.user_id.as_ref().map(|u| u.as_str().to_owned());
        // Why: the judge scores the Claude Code session, keyed by the client
        // session id; the gateway `sess_…` id has no summary row to evaluate.
        let client_session = row.client_session_id.as_deref().map(SessionId::new);
        let scorecard = ScorecardView::new(
            Scores::from_points(
                row.intent_score,
                row.quality_score,
                row.speed_score,
                row.cost_score,
            ),
            client_session
                .as_ref()
                .map(|s| format!("{}#evaluation", session_detail_url(s))),
            if row.scored_criteria > 0 {
                format!(
                    "Judge scores: {} of 4 criteria scored — open the session for the rationale",
                    row.scored_criteria
                )
            } else if client_session.is_some() {
                "Not judged yet".to_owned()
            } else {
                "No hook session behind this conversation, so nothing to judge".to_owned()
            },
        );
        Self {
            context_key: row.context_id.as_str().to_owned(),
            href: format!("/admin/contexts/{}", row.context_id.as_str()),
            session_href: row
                .session_id
                .as_ref()
                .map(|s| format!("/admin/sessions/{}", s.as_str())),
            evaluate_url: client_session.as_ref().map(session_evaluate_url),
            scorecard,
            user_href: user_id.as_ref().map(|u| format!("/admin/users/{u}")),
            display_name: row
                .display_name
                .clone()
                .or_else(|| user_id.clone())
                .unwrap_or_else(|| "Unknown".to_owned()),
            user_key: user_id.unwrap_or_default(),
            title: row.title.clone(),
            group_name: row.group_name.clone(),
            project_name: row.project_name.clone(),
            client_kind: row.client_kind.clone(),
            model: row.model.clone(),
            category_label: category_label(&row.category),
            category: row.category.clone(),
            summary: row.summary.clone().unwrap_or_default(),
            tags: row.tags.clone(),
            outcome: row.outcome.as_deref().map_or("Unclear", outcome_label),
            outcome_tone: row.outcome.as_deref().map_or("muted", outcome_tone),
            skills,
            confidence_display: row.confidence.map_or_else(
                || "—".to_owned(),
                |c| format!("{}%", (c * 100.0).round() as i64),
            ),
            turns: row.turn_count,
            tool_calls: row.tool_call_count,
            errors: row.error_count,
            tokens_display: format_token_total(row.total_tokens),
            cost_display: format_cost(row.total_cost_microdollars),
            last_display: row
                .last_at
                .map(|t| t.format("%b %-d, %H:%M").to_string())
                .unwrap_or_default(),
            last_full: row.last_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct SortHeaders {
    pub activity: SortHeaderView,
    pub turns: SortHeaderView,
    pub tokens: SortHeaderView,
    pub cost: SortHeaderView,
    pub quality: SortHeaderView,
}

#[derive(Debug, Serialize)]
pub(super) struct PageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub kpis: KpisView,
    pub filter: ConversationAnalysisFilterView,
    pub categories: Vec<SelectOptionView>,
    pub outcomes: Vec<SelectOptionView>,
    pub scored_options: Vec<SelectOptionView>,
    pub skills: Vec<SelectOptionView>,
    pub users: Vec<SelectOptionView>,
    pub scope_filter: ScopeFilterView,
    pub breakdown_tabs: Vec<TabLinkView>,
    pub breakdown_label: &'static str,
    pub breakdown: Vec<ConversationBucketView>,
    pub rows: Vec<ConversationRowView>,
    pub has_rows: bool,
    pub can_evaluate: bool,
    pub back_url: String,
    pub count_label: String,
    pub pagination: Pagination,
    pub sort_headers: SortHeaders,
    pub export: crate::export::ExportView,
}
