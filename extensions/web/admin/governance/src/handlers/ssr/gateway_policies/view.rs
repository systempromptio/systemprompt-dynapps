//! View types for `/admin/gateway/policies`.
//!
//! The form renders reserved blank window rows in place of an add control.

use chrono::NaiveDate;
use serde::Serialize;
use systemprompt::gateway::{QuotaWindow, SafetyHistoryMode};

use crate::repositories::gateway_policies::form::{CATEGORIES, MAX_WINDOWS, SCANNERS, SUBJECTS};
use crate::repositories::gateway_policies::rows::PolicyRow;
use crate::repositories::gateway_policies::windows::{is_calendar_month, window_label};
use systemprompt_web_admin_core::ssr::core_plane::CorePlaneCardView;
use systemprompt_web_admin_core::ssr::list_view::SelectOptionView;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToggleView {
    pub value: &'static str,
    pub checked: bool,
}

fn checks(all: &[&'static str], on: &[String]) -> Vec<ToggleView> {
    all.iter()
        .map(|v| ToggleView {
            value: v,
            checked: on.iter().any(|o| o == v),
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WindowView {
    pub index: usize,
    pub subjects: Vec<SelectOptionView>,
    pub choice: &'static str,
    pub custom_seconds: String,
    pub label: String,
    pub is_month: bool,
    pub max_requests: String,
    pub max_input_tokens: String,
    pub max_output_tokens: String,
    pub max_cost: String,
}

fn subjects(selected: Option<&str>) -> Vec<SelectOptionView> {
    let mut out = vec![SelectOptionView {
        value: String::new(),
        label: "— none —".to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(SUBJECTS.iter().map(|s| SelectOptionView {
        value: (*s).to_owned(),
        label: (*s).to_owned(),
        selected: selected == Some(s),
    }));
    out
}

const fn choice(w: &QuotaWindow) -> &'static str {
    if is_calendar_month(w) {
        return "month";
    }
    match w.window_seconds {
        3_600 => "hour",
        86_400 => "day",
        604_800 => "week",
        _ => "custom",
    }
}

fn optional(n: Option<i64>) -> String {
    n.map(|n| n.to_string()).unwrap_or_default()
}

fn dollars(n: Option<i64>) -> String {
    n.map(|n| format!("{:.2}", n as f64 / 1_000_000.0))
        .unwrap_or_default()
}

pub(crate) fn window_view(index: usize, w: &QuotaWindow, today: NaiveDate) -> WindowView {
    let choice = choice(w);
    WindowView {
        index,
        subjects: subjects(Some(&w.subject)),
        choice,
        custom_seconds: if choice == "custom" {
            w.window_seconds.to_string()
        } else {
            String::new()
        },
        label: window_label(w, today),
        is_month: is_calendar_month(w),
        max_requests: optional(w.max_requests),
        max_input_tokens: optional(w.max_input_tokens),
        max_output_tokens: optional(w.max_output_tokens),
        max_cost: dollars(w.max_cost_microdollars),
    }
}

pub(crate) fn blank_window(index: usize) -> WindowView {
    WindowView {
        index,
        subjects: subjects(None),
        choice: "day",
        custom_seconds: String::new(),
        label: String::new(),
        is_month: false,
        max_requests: String::new(),
        max_input_tokens: String::new(),
        max_output_tokens: String::new(),
        max_cost: String::new(),
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PolicyView {
    pub name: String,
    pub is_new: bool,
    pub enabled: bool,
    pub priority: i32,
    pub quota_warn: bool,
    pub windows: Vec<WindowView>,
    pub safety_warn: bool,
    pub scanners: Vec<ToggleView>,
    pub block: Vec<ToggleView>,
    pub block_response: Vec<ToggleView>,
    pub history: &'static str,
    pub heuristic_phrases: String,
    pub has_month_window: bool,
    pub updated_at: String,
    pub in_force: InForce,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub(crate) struct InForce {
    pub windows: bool,
    pub safety: bool,
}

const fn history(mode: SafetyHistoryMode) -> &'static str {
    match mode {
        SafetyHistoryMode::Off => "off",
        SafetyHistoryMode::Audit => "audit",
        SafetyHistoryMode::Block => "block",
    }
}

const RESERVED_WINDOW_ROWS: usize = 2;

fn with_blanks(mut windows: Vec<WindowView>) -> Vec<WindowView> {
    let target = (windows.len() + RESERVED_WINDOW_ROWS).min(MAX_WINDOWS);
    while windows.len() < target {
        windows.push(blank_window(windows.len()));
    }
    windows
}

pub(crate) fn policy_view(row: &PolicyRow, today: NaiveDate, in_force: InForce) -> PolicyView {
    let s = &row.spec;
    PolicyView {
        name: row.name.clone(),
        is_new: false,
        enabled: row.enabled,
        priority: row.priority,
        quota_warn: s.quota_mode.is_warn(),
        windows: with_blanks(
            s.quota_windows
                .iter()
                .enumerate()
                .map(|(i, w)| window_view(i, w, today))
                .collect(),
        ),
        safety_warn: s.safety.mode.is_warn(),
        scanners: checks(&SCANNERS, &s.safety.scanners),
        block: checks(&CATEGORIES, &s.safety.block_categories),
        block_response: checks(&CATEGORIES, &s.safety.block_response_categories),
        history: history(s.safety.history),
        heuristic_phrases: s
            .safety
            .heuristic
            .phrases
            .clone()
            .unwrap_or_default()
            .join("\n"),
        has_month_window: s.quota_windows.iter().any(is_calendar_month),
        updated_at: row.updated_at.to_rfc3339(),
        in_force,
    }
}

pub(crate) fn new_policy_view() -> PolicyView {
    PolicyView {
        name: String::new(),
        is_new: true,
        enabled: true,
        priority: 0,
        quota_warn: true,
        windows: with_blanks(Vec::new()),
        safety_warn: true,
        scanners: checks(&SCANNERS, &[]),
        block: checks(&CATEGORIES, &[]),
        block_response: checks(&CATEGORIES, &[]),
        history: "off",
        heuristic_phrases: String::new(),
        has_month_window: false,
        updated_at: String::new(),
        in_force: InForce::default(),
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct GatewayPoliciesPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub tabs: Vec<TabLinkView>,
    pub sync: Option<CorePlaneCardView>,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub can_write: bool,
    pub policies: Vec<PolicyView>,
    pub new_policy: PolicyView,
    pub quotas_url: &'static str,
    pub docs_url: &'static str,
}
