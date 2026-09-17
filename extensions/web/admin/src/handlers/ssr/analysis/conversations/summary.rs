//! The figures above the conversations table: KPI tiles, the breakdown
//! rows and the filter state, with the classifier's fixed vocabularies.

use serde::Serialize;

use crate::handlers::ssr::analysis::evaluation::views::score_tone;
use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::handlers::ssr::scorecard::{ScorecardView, Scores};
use crate::repositories::analysis::conversations::{
    ConversationAnalysisTotals, ConversationBucketRow,
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

pub(super) fn outcome_label(value: &str) -> &'static str {
    OUTCOMES
        .iter()
        .find(|(v, _)| *v == value)
        .map_or("Unclear", |(_, label)| label)
}

// Why: the badge tones the console already defines; an outcome reads at a
// glance without a legend.
pub(super) fn outcome_tone(value: &str) -> &'static str {
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
    pub scored: i64,
    pub scored_display: String,
    pub quality_display: String,
    pub quality_tone: &'static str,
    pub scorecard: ScorecardView,
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
            scored: t.scored,
            scored_display: percent(t.scored, t.conversations),
            quality_display: t
                .quality_avg
                .map_or_else(|| "—".to_owned(), |q| format!("{q:.0}")),
            quality_tone: t
                .quality_avg
                .map_or("muted", |q| score_tone(q.round() as i16)),
            scorecard: ScorecardView::new(
                Scores {
                    intent: t.intent_avg,
                    quality: t.quality_avg,
                    speed: t.speed_avg,
                    cost: t.cost_avg,
                },
                None,
                format!(
                    "Mean judge scores over the {} scored conversations in this set",
                    t.scored
                ),
            ),
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
    pub scored: String,
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
    pub quality_display: String,
    pub quality_tone: &'static str,
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
            quality_display: row
                .quality_avg
                .map_or_else(|| "—".to_owned(), |q| format!("{q:.0}")),
            quality_tone: row
                .quality_avg
                .map_or("muted", |q| score_tone(q.round() as i16)),
            top_category: row.top_category.as_deref().map_or("—", category_label),
        }
    }
}
