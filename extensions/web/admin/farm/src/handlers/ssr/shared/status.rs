//! Badges for run status and trigger, keyed by the stored text values.

use serde::Serialize;

use crate::types::{RunStatus, TriggeredVia};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BadgeView {
    pub tone: &'static str,
    pub text: String,
    pub key: String,
}

impl BadgeView {
    pub fn new(tone: &'static str, text: &str, key: &str) -> Self {
        Self {
            tone,
            text: text.to_owned(),
            key: key.to_owned(),
        }
    }
}

pub fn status_badge(status: &str) -> BadgeView {
    let (tone, text) = match RunStatus::parse(status) {
        Some(RunStatus::Queued) => ("muted", "Queued"),
        Some(RunStatus::Preparing) => ("info", "Preparing"),
        Some(RunStatus::Running) => ("accent", "Running"),
        Some(RunStatus::Succeeded) => ("ok", "Succeeded"),
        Some(RunStatus::Failed) => ("err", "Failed"),
        Some(RunStatus::Cancelled) => ("muted", "Cancelled"),
        Some(RunStatus::TimedOut) => ("warn", "Timed out"),
        None => ("muted", "Unknown"),
    };
    BadgeView::new(tone, text, status)
}

pub fn trigger_badge(via: &str) -> BadgeView {
    let (tone, text) = match TriggeredVia::parse(via) {
        Some(TriggeredVia::Dashboard) => ("info", "Dashboard"),
        Some(TriggeredVia::Api) => ("accent", "API"),
        Some(TriggeredVia::Eval) => ("muted", "Eval"),
        Some(TriggeredVia::Mcp) => ("accent", "MCP"),
        Some(TriggeredVia::Console) => ("info", "Console"),
        Some(TriggeredVia::Demo) => ("warn", "Demo"),
        Some(TriggeredVia::Scheduler) => ("accent", "Scheduler"),
        None => ("muted", "Unknown"),
    };
    BadgeView::new(tone, text, via)
}

pub fn is_live(status: &str) -> bool {
    RunStatus::parse(status).is_some_and(|s| !s.is_terminal())
}
