//! Template rows for `/admin/analysis/conversations`: repository rows shaped
//! for Handlebars, with every number pre-formatted.

use crate::handlers::ssr::entity_urls::session_evaluate_url;
use serde::Serialize;

use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::handlers::ssr::list_view::{Pagination, ScopeFilterView, SelectOptionView};
use crate::handlers::ssr::types::{SortHeaderView, TabLinkView};
use crate::repositories::analysis::conversations::{
    ConversationAnalysisRow, ConversationAnalysisTotals, ConversationBucketRow,
};

pub(super) const CATEGORIES: [(&str, &str); 7] = [
    ("development", "Development"),
    ("business-analysis", "Business analysis"),
    ("operations", "Operations"),
    ("admin-config", "Admin & config"),
    ("writing-comms", "Writing & comms"),
    ("research-learning", "Research & learning"),
    ("other", "Other"),
];

pub(super) const OUTCOMES: [(&str, &str); 4] = [
    ("achieved", "Achieved"),
    ("partial", "Partial"),
    ("abandoned", "Abandoned"),
    ("unclear", "Unclear"),
];

#[must_use]
pub(super) fn category_label(value: &str) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(v, _)| *v == value)
        .map_or("Other", |(_, label)| label)
}

fn outcome_label(value: &str) -> &'static str {
    OUTCOMES
        .iter()
        .find(|(v, _)| *v == value)
        .map_or("Unclear", |(_, label)| label)
}

// Why: the badge tones the console already defines; an outcome reads at a
// glance without a legend.
fn outcome_tone(value: &str) -> &'static str {
    match value {
        "achieved" => "ok",
        "partial" => "warn",
        "abandoned" => "err",
        _ => "muted",
    }
}

fn percent(part: i64, whole: i64) -> String {
    if whole == 0 {
        "—".to_owned()
    } else {
        format!("{}%", (part * 100) / whole)
    }
}

#[derive(Debug, Serialize)]
pub(super) struct KpisView {
    pub conversations: i64,
    pub unclassified: i64,
    pub users: i64,
    pub turns: i64,
    pub tool_calls: i64,
    pub tokens_display: String,
    pub cost_display: String,
    pub achieved_display: String,
}

impl KpisView {
    pub(super) fn new(t: &ConversationAnalysisTotals) -> Self {
        Self {
            conversations: t.conversations,
            unclassified: t.unclassified,
            users: t.users,
            turns: t.turns,
            tool_calls: t.tool_calls,
            tokens_display: format_token_total(t.total_tokens),
            cost_display: format_cost(t.total_cost_microdollars),
            achieved_display: percent(t.achieved, t.conversations),
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct ConversationAnalysisFilterView {
    pub since: &'static str,
    pub q: String,
    pub category: String,
    pub outcome: String,
    pub skill: String,
    pub user_key: String,
    pub group: String,
    pub project: String,
    pub by: &'static str,
    pub sort: &'static str,
    pub dir: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct ConversationBucketView {
    pub label: String,
    pub href: Option<String>,
    pub conversations: i64,
    pub users: i64,
    pub turns: i64,
    pub cost_display: String,
    pub achieved_display: String,
    pub top_category: &'static str,
}

impl ConversationBucketView {
    pub(super) fn new(
        row: &ConversationBucketRow,
        href: Option<String>,
        label: Option<&str>,
    ) -> Self {
        Self {
            label: label.map_or_else(|| row.label.clone(), str::to_owned),
            href,
            conversations: row.conversations,
            users: row.users,
            turns: row.turns,
            cost_display: format_cost(row.total_cost_microdollars),
            achieved_display: percent(row.achieved, row.conversations),
            top_category: row.top_category.as_deref().map_or("—", category_label),
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct SkillChipView {
    pub name: String,
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
                name: s.clone(),
                hooked: true,
            })
            .collect();
        for s in &row.skills_used {
            if !skills.iter().any(|k| k.name.eq_ignore_ascii_case(s)) {
                skills.push(SkillChipView {
                    name: s.clone(),
                    hooked: false,
                });
            }
        }
        let user_id = row.user_id.as_ref().map(|u| u.as_str().to_owned());
        Self {
            context_key: row.context_id.as_str().to_owned(),
            href: format!("/admin/contexts/{}", row.context_id.as_str()),
            session_href: row
                .session_id
                .as_ref()
                .map(|s| format!("/admin/sessions/{}", s.as_str())),
            evaluate_url: row.session_id.as_ref().map(session_evaluate_url),
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
}

#[derive(Debug, Serialize)]
pub(super) struct PageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub kpis: KpisView,
    pub filter: ConversationAnalysisFilterView,
    pub categories: Vec<SelectOptionView>,
    pub outcomes: Vec<SelectOptionView>,
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
}
