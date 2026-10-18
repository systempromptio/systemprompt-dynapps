//! The page frame around the leaderboard: the time-range picker that keeps
//! the farm filter, the farm select, and the KPI strip.

use serde::Serialize;
use systemprompt_web_admin_core::ssr::list_view::SelectOptionView;
use systemprompt_web_admin_core::util::time_range::TimeRange;
use systemprompt_web_shared::format::format_cost;

use super::rows::{PersonRowView, share_pct};
use crate::repositories::rows::FarmRow;
use crate::repositories::summaries::FarmRunStats;

pub(super) const BASE_URL: &str = "/admin/farm/people";

#[derive(Debug, Serialize)]
pub(super) struct PeopleTimeRange {
    pub preset: String,
    pub from: String,
    pub to: String,
    pub base_url: &'static str,
    pub query: String,
    pub rejected: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct PeopleKpi {
    pub label: &'static str,
    pub value: String,
    pub note: String,
    pub tone: &'static str,
}

pub(super) fn time_range(
    preset: Option<&str>,
    farm: Option<&str>,
    range: TimeRange,
) -> PeopleTimeRange {
    PeopleTimeRange {
        preset: preset.unwrap_or("7d").to_owned(),
        from: range.from.format("%Y-%m-%dT%H:%M").to_string(),
        to: range.to.format("%Y-%m-%dT%H:%M").to_string(),
        base_url: BASE_URL,
        query: farm.map(|f| format!("&farm={f}")).unwrap_or_default(),
        rejected: range.rejected_bounds,
    }
}

pub(super) fn farm_options(farms: &[FarmRow], selected: Option<&str>) -> Vec<SelectOptionView> {
    let mut out = vec![SelectOptionView {
        value: String::new(),
        label: "All farms".to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(farms.iter().map(|f| SelectOptionView {
        value: f.id.as_str().to_owned(),
        label: f.name.clone(),
        selected: selected == Some(f.id.as_str()),
    }));
    out
}

pub(super) fn kpis(stats: &FarmRunStats, people: &[PersonRowView]) -> Vec<PeopleKpi> {
    let finished = stats.succeeded + stats.failed + stats.timed_out;
    let success = if finished > 0 {
        format!("{}%", share_pct(stats.succeeded, finished).round())
    } else {
        "—".to_owned()
    };
    let success_tone = match finished {
        0 => "",
        _ if stats.succeeded * 10 >= finished * 9 => "ok",
        _ if stats.succeeded * 2 >= finished => "warn",
        _ => "err",
    };
    let top = people
        .first()
        .map(|p| format!("Most active: {}", p.display_name))
        .unwrap_or_default();
    vec![
        PeopleKpi {
            label: "People",
            value: people.len().to_string(),
            note: top,
            tone: "info",
        },
        PeopleKpi {
            label: "Runs",
            value: stats.total.to_string(),
            note: format!("{} running, {} queued", stats.running, stats.queued),
            tone: "accent",
        },
        PeopleKpi {
            label: "Cost",
            value: format_cost(stats.cost_microdollars),
            note: "Spent by runs in the window".to_owned(),
            tone: "",
        },
        PeopleKpi {
            label: "Success rate",
            value: success,
            note: format!("{} succeeded of {finished} finished", stats.succeeded),
            tone: success_tone,
        },
    ]
}
