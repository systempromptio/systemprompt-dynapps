//! Row-to-view-model formatting for the customer usage report.
//!
//! Every table on the page shares one share-bar rule: a row's bar scales
//! against the heaviest row in that table, so the three breakdowns read the
//! same way without the reader having to learn three scales.

use crate::handlers::ssr::format::format_token_total;
use crate::handlers::ssr::types::bar_pct;
use crate::repositories::reports::customer::{
    CustomerModelUsage, CustomerMonthSummary, CustomerProjectUsage, CustomerUserUsage,
};

use super::context::{CustomerSummaryView, ModelUsageView, ProjectUsageView, UserUsageView};

pub(super) fn summary_view(summary: &CustomerMonthSummary) -> CustomerSummaryView {
    let total = summary.total_tokens;
    CustomerSummaryView {
        active_users: summary.active_users,
        requests: summary.requests,
        total_tokens: total,
        total_tokens_display: format_token_total(total),
        input_tokens: summary.input_tokens,
        input_tokens_display: format_token_total(summary.input_tokens),
        output_tokens: summary.output_tokens,
        output_tokens_display: format_token_total(summary.output_tokens),
        cache_read_tokens: summary.cache_read_tokens,
        cache_read_tokens_display: format_token_total(summary.cache_read_tokens),
        reasoning_tokens: summary.reasoning_tokens,
        reasoning_tokens_display: format_token_total(summary.reasoning_tokens),
        reasoning_share_display: share_display(summary.reasoning_tokens, summary.output_tokens),
        error_count: summary.error_count,
        success_rate_display: success_rate(summary.requests, summary.error_count),
    }
}

pub(super) fn user_views(rows: &[CustomerUserUsage]) -> Vec<UserUsageView> {
    let totals: Vec<i64> = rows.iter().map(|r| r.total_tokens).collect();
    let grand: i64 = totals.iter().sum();
    let max = totals.iter().copied().max().unwrap_or(0);
    rows.iter()
        .zip(totals)
        .map(|(r, total)| UserUsageView {
            display_name: r.display_name.clone(),
            email: r.email.clone(),
            project: project_label(r.project.as_deref()),
            requests: r.requests,
            input_tokens_display: format_token_total(r.input_tokens),
            output_tokens_display: format_token_total(r.output_tokens),
            reasoning_tokens_display: format_token_total(r.reasoning_tokens),
            total_tokens: total,
            total_tokens_display: format_token_total(total),
            distinct_models: r.distinct_models,
            share_pct: bar_pct(total, max),
            share_display: share_display(total, grand),
        })
        .collect()
}

pub(super) fn project_views(rows: &[CustomerProjectUsage]) -> Vec<ProjectUsageView> {
    let totals: Vec<i64> = rows.iter().map(|r| r.total_tokens).collect();
    let grand: i64 = totals.iter().sum();
    let max = totals.iter().copied().max().unwrap_or(0);
    rows.iter()
        .zip(totals)
        .map(|(r, total)| ProjectUsageView {
            project: project_label(Some(r.project.as_str())),
            members: r.members,
            requests: r.requests,
            input_tokens_display: format_token_total(r.input_tokens),
            output_tokens_display: format_token_total(r.output_tokens),
            reasoning_tokens_display: format_token_total(r.reasoning_tokens),
            total_tokens: total,
            total_tokens_display: format_token_total(total),
            share_pct: bar_pct(total, max),
            share_display: share_display(total, grand),
        })
        .collect()
}

pub(super) fn model_views(rows: &[CustomerModelUsage]) -> Vec<ModelUsageView> {
    let totals: Vec<i64> = rows.iter().map(|r| r.total_tokens).collect();
    let grand: i64 = totals.iter().sum();
    let max = totals.iter().copied().max().unwrap_or(0);
    rows.iter()
        .zip(totals)
        .map(|(r, total)| ModelUsageView {
            provider: r.provider.clone(),
            model: r.model.clone(),
            requests: r.requests,
            input_tokens_display: format_token_total(r.input_tokens),
            output_tokens_display: format_token_total(r.output_tokens),
            cache_read_tokens_display: format_token_total(r.cache_read_tokens),
            reasoning_tokens_display: format_token_total(r.reasoning_tokens),
            total_tokens: total,
            total_tokens_display: format_token_total(total),
            share_pct: bar_pct(total, max),
            share_display: share_display(total, grand),
        })
        .collect()
}

fn project_label(project: Option<&str>) -> String {
    project.map_or_else(|| "Unassigned".to_owned(), ToOwned::to_owned)
}

// Why: the share column states a percentage of the whole, while the bar next
// to it scales against the largest row — the number answers "how much of our
// usage is this", the bar answers "how does this compare to the top".
fn share_display(value: i64, whole: i64) -> String {
    if whole <= 0 {
        return "—".to_owned();
    }
    format!("{}%", value.saturating_mul(100) / whole)
}

fn success_rate(requests: i64, errors: i64) -> String {
    if requests <= 0 {
        return "—".to_owned();
    }
    format!("{}%", (requests - errors).saturating_mul(100) / requests)
}
