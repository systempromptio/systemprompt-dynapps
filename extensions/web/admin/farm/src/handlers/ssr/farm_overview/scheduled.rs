//! The scheduled-flow strip on `/admin/farm`: the runs the scheduler started
//! over the window (the nightly accounting audit) and what their reports
//! recorded. A write count above zero is the one figure that is never
//! expected here: the farm denies every dynapps-odoo write tool.

use systemprompt_web_admin_core::ssr::format::{format_cost, local_time, short_num};

use super::activity::KpiTile;
use crate::handlers::ssr::shared::outcomes::{pct_display, rate_tone, score_display, tile};
use crate::repositories::reports::ScheduledStatsRow;

fn pct(part: i64, whole: i64) -> Option<f64> {
    (whole > 0).then(|| part as f64 * 100.0 / whole as f64)
}

pub(crate) fn scheduled_kpis(stats: &ScheduledStatsRow) -> Vec<KpiTile> {
    let last = stats
        .last_run_at
        .map_or_else(|| "never".to_owned(), local_time);
    let success = pct(stats.succeeded, stats.runs);
    let delivered = pct(stats.delivered, stats.reported);
    vec![
        KpiTile {
            href: stats
                .last_run_id
                .as_ref()
                .map(|id| format!("/admin/farm/runs/{id}")),
            ..tile(
                "scheduled-last",
                "Last scheduled run",
                "bolt",
                stats.last_status.clone().unwrap_or_else(|| "—".to_owned()),
                last,
            )
        },
        KpiTile {
            tone: rate_tone(success),
            ..tile(
                "scheduled-success",
                "Succeeded",
                "check",
                pct_display(success),
                format!("{} of {} runs", stats.succeeded, stats.runs),
            )
        },
        tile(
            "scheduled-cost",
            "Cost",
            "coins",
            format_cost(stats.cost_microdollars),
            format!("{} runs", short_num(stats.runs)),
        ),
        KpiTile {
            tone: if stats.mcp_writes > 0 { "err" } else { "ok" },
            hint: "Unattended runs are denied every dynapps-odoo write tool".to_owned(),
            ..tile(
                "scheduled-mcp",
                "MCP reads / writes",
                "report",
                format!("{} / {}", short_num(stats.mcp_reads), stats.mcp_writes),
                format!("{} governance warnings", stats.governance_warnings),
            )
        },
        KpiTile {
            tone: rate_tone(delivered),
            ..tile(
                "scheduled-delivery",
                "Delivered to DynBrain",
                "check",
                pct_display(delivered),
                format!("{} of {} reported runs", stats.delivered, stats.reported),
            )
        },
        KpiTile {
            tone: rate_tone(stats.eval_mean),
            ..tile(
                "scheduled-eval",
                "Eval score",
                "judge",
                score_display(stats.eval_mean),
                "mean over scored runs".to_owned(),
            )
        },
    ]
}
