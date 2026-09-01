//! Typed view-model for the customer-facing month-end usage report.
//!
//! There is deliberately no cost, margin, or unit-economics field on this
//! struct: the report is usage only. The repository behind it selects no cost
//! column either, so the two halves of the guarantee are independent.

use serde::Serialize;

use crate::handlers::ssr::list_view::ProjectFilterView;
use crate::util::month_range::MonthOption;

#[derive(Debug, Serialize)]
pub(super) struct ReportCustomerContext {
    pub page: &'static str,
    pub title: String,
    pub scope_name: String,
    pub project_key: Option<String>,
    pub month_key: String,
    pub month_label: String,
    pub month_complete: bool,
    pub months: Vec<MonthOption>,
    pub prev_url: String,
    pub next_url: Option<String>,
    pub has_next: bool,
    pub base_url: &'static str,
    pub generated_at: String,
    pub project_filter: ProjectFilterView,
    pub summary: CustomerSummaryView,
    // Why: Handlebars reads an empty array as falsy, so each table gates on the
    // collection itself rather than on a parallel `has_` flag that could
    // disagree with it.
    pub users: Vec<UserUsageView>,
    pub projects: Vec<ProjectUsageView>,
    pub models: Vec<ModelUsageView>,
}

#[derive(Debug, Serialize)]
pub(super) struct CustomerSummaryView {
    pub active_users: i64,
    pub requests: i64,
    pub total_tokens: i64,
    pub total_tokens_display: String,
    pub input_tokens: i64,
    pub input_tokens_display: String,
    pub output_tokens: i64,
    pub output_tokens_display: String,
    pub cache_read_tokens: i64,
    pub cache_read_tokens_display: String,
    pub error_count: i64,
    pub success_rate_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct UserUsageView {
    pub display_name: String,
    pub email: String,
    pub project: String,
    pub requests: i64,
    pub input_tokens_display: String,
    pub output_tokens_display: String,
    pub total_tokens: i64,
    pub total_tokens_display: String,
    pub distinct_models: i64,
    pub share_pct: i64,
    pub share_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ProjectUsageView {
    pub project: String,
    pub members: i64,
    pub requests: i64,
    pub input_tokens_display: String,
    pub output_tokens_display: String,
    pub total_tokens: i64,
    pub total_tokens_display: String,
    pub share_pct: i64,
    pub share_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ModelUsageView {
    pub provider: String,
    pub model: String,
    pub requests: i64,
    pub input_tokens_display: String,
    pub output_tokens_display: String,
    pub cache_read_tokens_display: String,
    pub total_tokens: i64,
    pub total_tokens_display: String,
    pub share_pct: i64,
    pub share_display: String,
}
