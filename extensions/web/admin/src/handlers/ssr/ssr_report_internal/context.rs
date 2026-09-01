//! Typed view-model for the internal month-end provider cost report.
//!
//! Mirrors every field `storage/files/admin/templates/report-internal.hbs`
//! references. Handlebars runs in strict mode, so a field the template reads
//! must exist here even when its value is null.
//!
//! Money stays in microdollars all the way to the template, where `formatUsd`
//! renders it once. Rounding in Rust and again in Handlebars is how a total
//! stops matching the rows above it.

use serde::Serialize;

use crate::handlers::ssr::types::ChartView;
use crate::util::month_range::MonthOption;

#[derive(Debug, Serialize)]
pub(super) struct ReportInternalContext {
    pub page: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub month_key: String,
    pub month_label: String,
    // Why: False while the month is still running. The template turns this into a
    // banner, because a partial month's margin is not a result.
    pub month_complete: bool,
    pub months: Vec<MonthOption>,
    pub prev_url: String,
    pub next_url: Option<String>,
    pub has_next: bool,
    pub base_url: &'static str,
    // Why: Always null here. The shared month-selector partial preserves a
    // project across a month change on the usage report, and Handlebars runs
    // in strict mode: the key has to exist even where the page has no project
    // to carry.
    pub project_key: Option<String>,
    pub generated_at: String,
    pub totals: InternalTotals,
    // Why: Handlebars reads an empty array as falsy, so each table gates on the
    // collection itself rather than on a parallel `has_` flag that could
    // disagree with it.
    pub providers: Vec<SupplierView>,
    pub models: Vec<SupplierView>,
    pub trend: ChartView,
}

// Why: every figure is folded from the same provider rows the table prints,
// so the header and the body cannot disagree.
#[derive(Debug, Serialize)]
pub(super) struct InternalTotals {
    pub requests: i64,
    pub tokens: i64,
    pub tokens_display: String,
    pub cost_microdollars: i64,
    pub cost_per_1m_display: String,
    pub cost_per_request_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SupplierView {
    pub key: String,
    pub requests: i64,
    pub tokens_display: String,
    pub cost_microdollars: i64,
    pub share_pct: i64,
    pub share_display: String,
}
