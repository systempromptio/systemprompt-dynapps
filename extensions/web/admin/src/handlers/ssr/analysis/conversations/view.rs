//! Template rows for the Analysis conversation tables: one conversation's
//! deterministic record shaped for Handlebars, every number pre-formatted
//! and toned, with the judge's label beside it. Built from `RowFacts`, which
//! both the conversations page rows and the skill page's rows provide, so
//! the two tables read the same.

use serde::Serialize;

use super::kpis::KpisView;
use super::row_facts::RowFacts;
use super::summary::{
    ConversationAnalysisFilterView, ConversationBucketView, category_label, outcome_label,
    outcome_tone,
};
use crate::handlers::ssr::analysis::tone::{
    cache_tone, deny_tone, error_rate_tone, latency_tone, safety_tone, score_tone,
};
use crate::handlers::ssr::analysis_urls::{analysis_conversation_url, analysis_skill_url};
use crate::handlers::ssr::format::{
    ClientChipView, client_chip, format_cost, format_duration_ms, format_token_total,
};
use crate::handlers::ssr::list_view::{Pagination, ScopeFilterView, SelectOptionView};
use crate::handlers::ssr::types::{
    SortHeaderView, SparklineView, SvgLineChartView, TabLinkView, sparkline_toned,
};

#[derive(Debug, Serialize)]
pub(crate) struct SkillChipView {
    pub name: String,
    pub href: String,
    // Why: a skill the harness hook reported is evidence; one the judge
    // inferred from the transcript alone is a reading.
    pub hooked: bool,
}

// Why: the judge's label as a row shows it.
#[derive(Debug, Serialize)]
pub(crate) struct JudgeCellView {
    pub judged: bool,
    pub pending: bool,
    pub completion: Option<i16>,
    pub completion_display: String,
    pub tone: &'static str,
    pub category: String,
    pub category_label: &'static str,
    pub outcome: &'static str,
    pub outcome_tone: &'static str,
    pub summary: String,
    pub rationale: String,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConversationRowView {
    pub context_key: String,
    pub href: String,
    pub session_href: Option<String>,
    pub title: String,
    pub user_key: String,
    pub user_href: String,
    pub display_name: String,
    pub group_name: Option<String>,
    pub project_name: Option<String>,
    pub client: ClientChipView,
    pub model: String,
    pub extra_models: Vec<String>,
    pub turns: i64,
    pub tools_display: String,
    pub tools_tone: &'static str,
    pub tools_title: String,
    pub errors: i64,
    pub errors_tone: &'static str,
    pub denied: i64,
    pub denied_tone: &'static str,
    pub safety: i64,
    pub safety_tone: &'static str,
    pub tokens_display: String,
    pub tokens_title: String,
    pub cache_pct: i64,
    pub cache_tone: &'static str,
    pub cost_display: String,
    pub latency_display: String,
    pub latency_tone: &'static str,
    pub duration_display: String,
    pub skills: Vec<SkillChipView>,
    pub spark: SparklineView,
    pub judge: JudgeCellView,
    pub first_display: String,
    pub last_display: String,
    pub last_full: String,
}

fn skill_chips(hooked: &[String], inferred: &[String]) -> Vec<SkillChipView> {
    let mut skills: Vec<SkillChipView> = hooked
        .iter()
        .map(|s| SkillChipView {
            href: analysis_skill_url(s),
            name: s.clone(),
            hooked: true,
        })
        .collect();
    for s in inferred {
        if !skills.iter().any(|k| k.name.eq_ignore_ascii_case(s)) {
            skills.push(SkillChipView {
                href: analysis_skill_url(s),
                name: s.clone(),
                hooked: false,
            });
        }
    }
    skills
}

// Why: the tools cell reads "executed/asked" only when the two differ, so a
// clean row shows one number.
fn tools_cell(f: &RowFacts<'_>) -> (String, &'static str, String) {
    let asked = f.tool_calls_intended.max(f.tool_calls_executed);
    let display = if f.tool_calls_executed > 0 && f.tool_calls_executed != f.tool_calls_intended {
        format!("{}/{}", f.tool_calls_executed, asked)
    } else {
        asked.to_string()
    };
    let tone = if f.tool_calls_failed > 0 {
        "err"
    } else if asked > 0 {
        "accent"
    } else {
        "muted"
    };
    let title = format!(
        "{} tool calls requested by the model · {} executed · {} failed",
        f.tool_calls_intended, f.tool_calls_executed, f.tool_calls_failed
    );
    (display, tone, title)
}

fn tokens_cell(f: &RowFacts<'_>) -> (String, String, i64) {
    let read = f.input_tokens + f.cache_tokens;
    (
        format_token_total(f.input_tokens + f.output_tokens),
        format!(
            "{} in · {} out · {} cached",
            f.input_tokens, f.output_tokens, f.cache_tokens
        ),
        if read > 0 {
            f.cache_tokens * 100 / read
        } else {
            0
        },
    )
}

pub(crate) fn judge_cell(f: &RowFacts<'_>) -> JudgeCellView {
    let judged = f.completion.is_some();
    JudgeCellView {
        judged,
        pending: !judged && f.judge_status == Some("pending"),
        completion: f.completion,
        completion_display: f
            .completion
            .map_or_else(|| "—".to_owned(), |c| c.to_string()),
        tone: f.completion.map_or("muted", score_tone),
        category: f.category.unwrap_or_default().to_owned(),
        category_label: f.category.map_or("Unjudged", category_label),
        outcome: f.outcome.map_or("—", outcome_label),
        outcome_tone: f.outcome.map_or("muted", outcome_tone),
        summary: f.summary.unwrap_or_default().to_owned(),
        rationale: f.rationale.unwrap_or_default().to_owned(),
        title: f.judge_title.unwrap_or_default().to_owned(),
    }
}

impl ConversationRowView {
    pub(crate) fn new(f: &RowFacts<'_>) -> Self {
        let tools = tools_cell(f);
        let tokens = tokens_cell(f);
        Self {
            context_key: f.context_id.as_str().to_owned(),
            href: analysis_conversation_url(f.context_id),
            session_href: f
                .client_session_id
                .map(|s| format!("/admin/sessions/{}", urlencoding::encode(s))),
            title: f
                .judge_title
                .filter(|t| !t.is_empty())
                .unwrap_or(f.title)
                .to_owned(),
            user_key: f.user_id.to_owned(),
            user_href: format!("/admin/users/{}", urlencoding::encode(f.user_id)),
            display_name: f
                .display_name
                .filter(|n| !n.is_empty())
                .unwrap_or(f.user_id)
                .to_owned(),
            group_name: f.group_name.map(str::to_owned),
            project_name: f.project_name.map(str::to_owned),
            client: client_chip(f.client_kind, f.client_attestation),
            model: f.model.unwrap_or("—").to_owned(),
            extra_models: f
                .models
                .iter()
                .filter(|m| Some(m.as_str()) != f.model)
                .cloned()
                .collect(),
            turns: f.turn_count,
            tools_display: tools.0,
            tools_tone: tools.1,
            tools_title: tools.2,
            errors: f.error_count,
            errors_tone: error_rate_tone(f.error_count, f.request_count),
            denied: f.gov_deny,
            denied_tone: deny_tone(f.gov_deny),
            safety: f.safety_findings,
            safety_tone: safety_tone(f.safety_findings, f.safety_blocked),
            tokens_display: tokens.0,
            tokens_title: tokens.1,
            cache_pct: tokens.2,
            cache_tone: cache_tone(f.cache_tokens, f.input_tokens),
            cost_display: format_cost(f.cost_microdollars),
            latency_display: f
                .p95_latency_ms
                .map_or_else(|| "—".to_owned(), |ms| format_duration_ms(i64::from(ms))),
            latency_tone: latency_tone(f.p95_latency_ms.map(f64::from)),
            duration_display: format_duration_ms(f.duration_seconds * 1000),
            skills: skill_chips(f.skills, f.skills_used),
            spark: sparkline_toned(
                f.turn_tokens,
                "accent",
                format!("Tokens per turn, last {} turns", f.turn_tokens.len()),
            ),
            judge: judge_cell(f),
            first_display: f.first_at.format("%b %-d, %H:%M").to_string(),
            last_display: f.last_at.format("%b %-d, %H:%M").to_string(),
            last_full: f.last_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct SortHeaders {
    pub activity: SortHeaderView,
    pub turns: SortHeaderView,
    pub tools: SortHeaderView,
    pub errors: SortHeaderView,
    pub tokens: SortHeaderView,
    pub cost: SortHeaderView,
    pub latency: SortHeaderView,
    pub duration: SortHeaderView,
    pub completion: SortHeaderView,
}

#[derive(Debug, Serialize)]
pub(crate) struct FacetLinkView {
    pub label: String,
    pub href: String,
    pub count: i64,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConversationsPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub kpis: KpisView,
    pub charts: Vec<SvgLineChartView>,
    pub range_links: Vec<TabLinkView>,
    pub filter: ConversationAnalysisFilterView,
    pub categories: Vec<SelectOptionView>,
    pub outcomes: Vec<SelectOptionView>,
    pub judged_options: Vec<SelectOptionView>,
    pub flag_options: Vec<SelectOptionView>,
    pub skills: Vec<SelectOptionView>,
    pub users: Vec<SelectOptionView>,
    pub models: Vec<SelectOptionView>,
    pub clients: Vec<SelectOptionView>,
    pub model_facets: Vec<FacetLinkView>,
    pub client_facets: Vec<FacetLinkView>,
    pub scope_filter: ScopeFilterView,
    pub breakdown_tabs: Vec<TabLinkView>,
    pub breakdown_label: &'static str,
    pub breakdown: Vec<ConversationBucketView>,
    pub rows: Vec<ConversationRowView>,
    pub has_rows: bool,
    pub count_label: String,
    pub pagination: Pagination,
    pub sort_headers: SortHeaders,
    pub export: crate::export::ExportView,
}
