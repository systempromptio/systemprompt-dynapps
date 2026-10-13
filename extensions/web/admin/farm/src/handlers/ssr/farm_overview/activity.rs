//! The live half of the farms overview: the KPI strip and the "Now running"
//! board. The page embeds it and `GET /admin/api/farm/activity` returns it,
//! both rendered from [`load_activity`] through the same partial, so a figure
//! reads the same after a refresh as on first load.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::format::{format_cost, short_num};
use systemprompt_web_admin_core::ssr::types::sparkline::{SparklineView, sparkline_toned};

use crate::handlers::ssr::shared::{RunRowView, run_row_views};
use crate::repositories::agents::count_agents;
use crate::repositories::farms::count_farms;
use crate::repositories::filters::RunFilter;
use crate::repositories::runs::{get_run_stats, list_active_runs, list_daily_run_counts};
use crate::repositories::summaries::{DailyRunCount, RunStats};

pub(crate) const ACTIVITY_PARTIAL: &str = "farm/activity-live";
const SPARK_DAYS: i64 = 14;
const RUNS_URL: &str = "/admin/farm/runs";

#[derive(Debug, Serialize)]
pub(crate) struct KpiTile {
    pub(crate) key: &'static str,
    pub(crate) label: &'static str,
    pub(crate) icon: &'static str,
    pub(crate) value: String,
    pub(crate) note: String,
    pub(crate) hint: String,
    pub(crate) href: Option<String>,
    pub(crate) tone: &'static str,
    pub(crate) spark: Option<SparklineView>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ActivityView {
    pub(crate) kpis: Vec<KpiTile>,
    pub(crate) running: Vec<RunRowView>,
    pub(crate) running_count: usize,
    pub(crate) has_running: bool,
    pub(crate) unavailable: bool,
}

struct Figures {
    farms: i64,
    agents: i64,
    today: RunStats,
    week: RunStats,
    daily: Vec<i64>,
    active: usize,
    queued: usize,
}

pub(crate) async fn load_activity(pool: &PgPool, faults: &mut Faults) -> ActivityView {
    let before = faults.as_slice().len();
    let now = Utc::now();
    let today = RunFilter {
        since: Some(start_of_day(now)),
        ..RunFilter::default()
    };
    let week = RunFilter {
        since: Some(now - Duration::days(7)),
        ..RunFilter::default()
    };
    let spark_since = start_of_day(now) - Duration::days(SPARK_DAYS - 1);
    let (farms, agents, today_stats, week_stats, daily, active) = tokio::join!(
        count_farms(pool),
        count_agents(pool, None),
        get_run_stats(pool, &today),
        get_run_stats(pool, &week),
        list_daily_run_counts(pool, spark_since),
        list_active_runs(pool),
    );
    let active = faults.take("running farm runs", active);
    let running = run_row_views(&active);
    let queued = active.iter().filter(|r| r.status == "queued").count();
    let figures = Figures {
        farms: faults.take("farm count", farms),
        agents: faults.take("agent count", agents),
        today: faults.take("today's run stats", today_stats),
        week: faults.take("7-day run stats", week_stats),
        daily: fill_days(
            &faults.take("daily run counts", daily),
            spark_since.date_naive(),
        ),
        active: running.len(),
        queued,
    };
    ActivityView {
        kpis: kpi_tiles(&figures),
        running_count: running.len(),
        has_running: !running.is_empty(),
        running,
        unavailable: faults.as_slice().len() > before,
    }
}

fn start_of_day(now: DateTime<Utc>) -> DateTime<Utc> {
    now.date_naive()
        .and_hms_opt(0, 0, 0)
        .map_or(now, |d| d.and_utc())
}

pub(crate) fn fill_days(rows: &[DailyRunCount], first: NaiveDate) -> Vec<i64> {
    (0..SPARK_DAYS)
        .filter_map(|offset| first.checked_add_signed(Duration::days(offset)))
        .map(|day| rows.iter().find(|r| r.day == day).map_or(0, |r| r.runs))
        .collect()
}

pub(crate) fn success_rate(stats: &RunStats) -> Option<f64> {
    let finished = stats.succeeded + stats.failed + stats.timed_out;
    (finished > 0).then(|| stats.succeeded as f64 / finished as f64)
}

fn success_tone(rate: Option<f64>) -> &'static str {
    match rate {
        Some(r) if r >= 0.9 => "ok",
        Some(r) if r >= 0.7 => "warn",
        Some(_) => "err",
        None => "",
    }
}

fn kpi_tiles(f: &Figures) -> Vec<KpiTile> {
    let rate = success_rate(&f.week);
    let finished = f.week.succeeded + f.week.failed + f.week.timed_out;
    vec![
        tile(
            "farms",
            "Farms",
            "layers",
            f.farms.to_string(),
            "declared and synced".to_owned(),
        ),
        tile(
            "agents",
            "Agents",
            "skill",
            f.agents.to_string(),
            "across every farm".to_owned(),
        ),
        KpiTile {
            href: Some(RUNS_URL.to_owned()),
            spark: Some(sparkline_toned(
                &f.daily,
                "accent",
                format!("Runs per day, last {SPARK_DAYS} days"),
            )),
            ..tile(
                "runs-today",
                "Runs today",
                "bolt",
                short_num(f.today.total),
                format!("{} failed", f.today.failed + f.today.timed_out),
            )
        },
        KpiTile {
            href: Some(format!("{RUNS_URL}?status=running")),
            tone: if f.active > 0 { "accent" } else { "" },
            ..tile(
                "running",
                "Running now",
                "clock",
                (f.active - f.queued).to_string(),
                format!("{} queued", f.queued),
            )
        },
        KpiTile {
            tone: success_tone(rate),
            hint: format!("{} of {finished} finished runs succeeded", f.week.succeeded),
            ..tile(
                "success-7d",
                "Success 7d",
                "check",
                rate.map_or_else(|| "—".to_owned(), |r| format!("{:.0}%", r * 100.0)),
                format!("{finished} finished"),
            )
        },
        tile(
            "cost-7d",
            "Cost 7d",
            "coins",
            format_cost(f.week.cost_microdollars),
            format!("{} runs", short_num(f.week.total)),
        ),
    ]
}

const fn tile(
    key: &'static str,
    label: &'static str,
    icon: &'static str,
    value: String,
    note: String,
) -> KpiTile {
    KpiTile {
        key,
        label,
        icon,
        value,
        note,
        hint: String::new(),
        href: None,
        tone: "",
        spark: None,
    }
}
