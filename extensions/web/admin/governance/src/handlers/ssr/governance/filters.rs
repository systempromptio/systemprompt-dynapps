//! The filter bar of `/admin/governance`: the select options, the time-range
//! picker and the export menu, each derived from the URL's query alone.

use super::urls::url_with;
use super::{BASE_URL, GovernanceQuery, GovernanceTab};
use systemprompt_web_admin_core::ssr::list_view::{SelectOptionView, TimeRangeContext};
use systemprompt_web_admin_core::util::time_range::TimeRange;

pub(super) fn options(
    all_label: &str,
    selected: Option<&str>,
    values: &[String],
) -> Vec<SelectOptionView> {
    let mut out = vec![SelectOptionView {
        value: String::new(),
        label: all_label.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(values.iter().map(|value| SelectOptionView {
        selected: selected == Some(value.as_str()),
        value: value.clone(),
        label: value.clone(),
    }));
    out
}

pub(super) fn decision_options(query: &GovernanceQuery) -> Vec<SelectOptionView> {
    ["", "deny", "warn", "allow"]
        .into_iter()
        .map(|value| SelectOptionView {
            value: value.to_owned(),
            label: if value.is_empty() {
                "All outcomes".to_owned()
            } else {
                value.to_owned()
            },
            selected: query.decision.as_deref().unwrap_or("") == value,
        })
        .collect()
}

pub(super) fn outcome_options(query: &GovernanceQuery) -> Vec<SelectOptionView> {
    ["", "blocked", "audited"]
        .into_iter()
        .map(|value| SelectOptionView {
            value: value.to_owned(),
            label: if value.is_empty() {
                "All outcomes".to_owned()
            } else {
                value.to_owned()
            },
            selected: query.blocked.as_deref().unwrap_or("") == value,
        })
        .collect()
}

pub(super) fn time_range_context(query: &GovernanceQuery, range: TimeRange) -> TimeRangeContext {
    TimeRangeContext {
        preset: query.preset.clone().unwrap_or_else(|| "7d".to_owned()),
        from: range.from.format("%Y-%m-%dT%H:%M").to_string(),
        to: range.to.format("%Y-%m-%dT%H:%M").to_string(),
        base_url: BASE_URL,
        query: "",
        rejected: range.rejected_bounds,
    }
}

pub(super) fn export_view(
    query: &GovernanceQuery,
    tab: GovernanceTab,
) -> systemprompt_web_admin_core::export::view::ExportView {
    let query = url_with(query, &[("page", ""), ("tab", "")]);
    let query = query.split_once('?').map_or("", |(_, q)| q);
    let ids: &[&str] = match tab {
        GovernanceTab::Safety => &["governance-findings", "governance-decisions"],
        GovernanceTab::Decisions | GovernanceTab::Hooks => {
            &["governance-decisions", "governance-findings"]
        },
    };
    systemprompt_web_admin_core::export::view::ExportView::new(ids, query)
}
