//! Small view types and formatters the pages share.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

pub const NOT_MOUNTED: &str =
    "The Brain is not running on this instance: its configuration could not be used. The log says why.";
pub const AT_SOURCE: &str = "This instance is a proxy. Chunks, embeddings and the ingest queue live at the source, and this console only sees what the source's operations return.";
pub const NONE: &str = "—";

/// One KPI tile (`components/kpi`).
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Tile {
    pub label: String,
    pub value: String,
    pub note: Option<String>,
    pub tone: Option<&'static str>,
}

pub fn tile(label: &str, value: impl ToString, tone: Option<&'static str>) -> Tile {
    Tile {
        label: label.to_owned(),
        value: value.to_string(),
        note: None,
        tone,
    }
}

pub fn crumbs(current: &str) -> Vec<BreadcrumbView> {
    vec![
        BreadcrumbView::link("Brain", "/admin/brain"),
        BreadcrumbView::current(current),
    ]
}

pub fn when(time: DateTime<Utc>) -> String {
    time.format("%Y-%m-%d %H:%M").to_string()
}

pub fn usd(micros: i64) -> String {
    format!("{}.{:02}", micros / 1_000_000, (micros % 1_000_000) / 10_000)
}

pub fn words(text: &str) -> usize {
    text.split_whitespace().count()
}

pub fn clip(text: &str, chars: usize) -> String {
    text.chars().take(chars).collect()
}

pub fn state_tone(state: &str) -> &'static str {
    match state {
        "done" | "ok" => "ok",
        "failed" | "error" => "err",
        "pending" | "running" => "warn",
        _ => "muted",
    }
}

pub fn keep<T, E: std::fmt::Display>(
    faults: &mut systemprompt_web_admin_core::ssr::faults::Faults,
    what: &'static str,
    result: Result<T, E>,
) -> Option<T> {
    result.map_err(|error| faults.record(what, &error)).ok()
}
