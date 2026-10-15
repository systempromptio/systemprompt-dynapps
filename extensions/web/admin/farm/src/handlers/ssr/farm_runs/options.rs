//! The runs explorer's filter form: the options each select offers, the
//! hidden fields that carry the window and sort through it, and a removable
//! chip per filter in force.

use systemprompt_web_admin_core::ssr::list_view::{AnnotatedOption, Chip, Preserved};
use systemprompt_web_admin_core::util::time_range::TimeRange;

use super::query::RunsQuery;
use crate::handlers::ssr::shared::{status_badge, trigger_badge};
use crate::repositories::rows::{AgentRow, FarmRow, RunsByUserRow};
use crate::types::{RunStatus, TriggeredVia};

const STATUSES: [RunStatus; 7] = [
    RunStatus::Queued,
    RunStatus::Preparing,
    RunStatus::Running,
    RunStatus::Succeeded,
    RunStatus::Failed,
    RunStatus::Cancelled,
    RunStatus::TimedOut,
];
const TRIGGERS: [TriggeredVia; 7] = TriggeredVia::ALL;

pub(super) fn preserved(query: &RunsQuery, range: TimeRange, preset: &str) -> Vec<Preserved> {
    let mut out = vec![
        Preserved {
            name: "preset",
            value: preset.to_owned(),
        },
        Preserved {
            name: "from",
            value: range.from.to_rfc3339(),
        },
        Preserved {
            name: "to",
            value: range.to.to_rfc3339(),
        },
    ];
    for (name, value) in [
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
    ] {
        if let Some(v) = value.filter(|v| !v.is_empty()) {
            out.push(Preserved {
                name,
                value: v.to_owned(),
            });
        }
    }
    out
}

fn option(id: &str, label: String, count: i64, selected: Option<&str>) -> AnnotatedOption {
    AnnotatedOption {
        id: id.to_owned(),
        label,
        count,
        selected: selected == Some(id),
    }
}

pub(super) fn farm_options(farms: &[FarmRow], selected: Option<&str>) -> Vec<AnnotatedOption> {
    farms
        .iter()
        .map(|f| option(f.id.as_str(), f.name.clone(), 0, selected))
        .collect()
}

pub(super) fn agent_options(agents: &[AgentRow], selected: Option<&str>) -> Vec<AnnotatedOption> {
    let mut out: Vec<AnnotatedOption> = Vec::with_capacity(agents.len());
    for a in agents {
        if out.iter().any(|o| o.id == a.id.as_str()) {
            continue;
        }
        out.push(option(
            a.id.as_str(),
            format!("{} ({})", a.name, a.farm_id.as_str()),
            0,
            selected,
        ));
    }
    out
}

pub(super) fn user_options(
    users: &[RunsByUserRow],
    selected: Option<&str>,
) -> Vec<AnnotatedOption> {
    users
        .iter()
        .map(|u| {
            let label = u
                .display_name
                .clone()
                .unwrap_or_else(|| u.user_id.as_str().to_owned());
            option(u.user_id.as_str(), label, u.runs.unwrap_or(0), selected)
        })
        .collect()
}

pub(super) fn status_options(selected: Option<RunStatus>) -> Vec<AnnotatedOption> {
    let selected = selected.map(RunStatus::as_str);
    STATUSES
        .iter()
        .map(|s| option(s.as_str(), status_badge(s.as_str()).text, 0, selected))
        .collect()
}

pub(super) fn trigger_options(selected: Option<TriggeredVia>) -> Vec<AnnotatedOption> {
    let selected = selected.map(TriggeredVia::as_str);
    TRIGGERS
        .iter()
        .map(|t| option(t.as_str(), trigger_badge(t.as_str()).text, 0, selected))
        .collect()
}

pub(super) fn chips(query: &RunsQuery) -> Vec<Chip> {
    let status = query.status_value().map(|s| status_badge(s.as_str()).text);
    let trigger = query
        .trigger_value()
        .map(|t| trigger_badge(t.as_str()).text);
    let groups: [(&str, &'static str, Option<String>); 5] = [
        ("farm_id", "Farm", query.farm().map(str::to_owned)),
        ("agent_id", "Agent", query.agent().map(str::to_owned)),
        ("user_id", "User", query.user().map(str::to_owned)),
        ("status", "Status", status),
        ("trigger", "Trigger", trigger),
    ];
    groups
        .into_iter()
        .filter_map(|(param, group_label, label)| {
            label.map(|label| Chip {
                group_label,
                value: label.clone(),
                label,
                remove_url: query.url_without(&[param, "page"]),
            })
        })
        .collect()
}

pub(super) fn clear_url(query: &RunsQuery) -> String {
    query.url_without(&[
        "farm_id", "agent_id", "user_id", "status", "trigger", "page",
    ])
}
