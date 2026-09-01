//! Row-to-view-model formatting for the internal provider cost report.
//!
//! Pure functions over what the repository returned. Keeping them here means
//! the handler reads as "fetch, assemble, render" and every display rule —
//! how a share bar scales — has one home.

use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::handlers::ssr::types::{ChartBarView, ChartView, bar_pct};
use crate::repositories::reports::internal::{PlatformMonthPoint, SupplierMonthCost};

use super::context::{InternalTotals, SupplierView};

// Why: folded from the provider rows the table prints, so the header and
// the body cannot disagree.
pub(super) fn totals(rows: &[SupplierMonthCost]) -> InternalTotals {
    let cost: i64 = rows.iter().map(|r| r.cost_microdollars).sum();
    let tokens: i64 = rows.iter().map(|r| r.tokens).sum();
    let requests: i64 = rows.iter().map(|r| r.requests).sum();
    InternalTotals {
        requests,
        tokens,
        tokens_display: format_token_total(tokens),
        cost_microdollars: cost,
        cost_per_1m_display: per_unit_display(cost, tokens, 1_000_000),
        cost_per_request_display: per_unit_display(cost, requests, 1),
    }
}

pub(super) fn supplier_views(rows: &[SupplierMonthCost]) -> Vec<SupplierView> {
    let max = rows.iter().map(|r| r.cost_microdollars).max().unwrap_or(0);
    rows.iter()
        .map(|r| SupplierView {
            key: r.key.clone(),
            requests: r.requests,
            tokens_display: format_token_total(r.tokens),
            cost_microdollars: r.cost_microdollars,
            share_pct: bar_pct(r.cost_microdollars, max),
            share_display: format_cost(r.cost_microdollars),
        })
        .collect()
}

pub(super) fn trend_chart(points: &[PlatformMonthPoint]) -> ChartView {
    let max = points
        .iter()
        .map(|p| p.cost_microdollars)
        .max()
        .unwrap_or(0);
    let total: i64 = points.iter().map(|p| p.cost_microdollars).sum();
    let (first, last) = (points.first(), points.last());
    ChartView {
        title: "Provider cost by month",
        subtitle: format!("{} across the window", format_cost(total)),
        tone: "success",
        series: points
            .iter()
            .map(|p| ChartBarView {
                pct: bar_pct(p.cost_microdollars, max),
                tooltip: format!(
                    "{}: {} over {} requests",
                    month_label(p),
                    format_cost(p.cost_microdollars),
                    p.requests
                ),
            })
            .collect(),
        has_data: max > 0,
        y_max_display: format_cost(max),
        y_mid_display: format_cost(max / 2),
        x_start_display: first.map(month_label).unwrap_or_default(),
        x_mid_display: points
            .get(points.len() / 2)
            .map(month_label)
            .unwrap_or_default(),
        x_end_display: last.map(month_label).unwrap_or_default(),
        empty_message: "No billed requests in these months.",
    }
}

fn month_label(point: &PlatformMonthPoint) -> String {
    point.month_start.format("%b %Y").to_string()
}

fn per_unit_display(cost: i64, units: i64, per: i64) -> String {
    if units <= 0 {
        return "—".to_owned();
    }
    format_cost(cost.saturating_mul(per) / units)
}
