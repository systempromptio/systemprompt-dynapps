//! The skills page's query: the UTC window, the optional drill-down (a
//! managed resource by id, or a skill by the hook's plugin and skill), and
//! the page — with the URL builder that renders them back.

use chrono::{DateTime, Duration, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::{ManagedResourceId, PluginId};

use super::time::{self, WindowBound};
use crate::error::AdminError;
use crate::repositories::analysis::retained::{RetainedSkillRef, RetainedWindow};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisQuery {
    #[serde(default)]
    pub start: WindowBound,
    #[serde(default)]
    pub end: WindowBound,
    pub resource: Option<String>,
    pub plugin: Option<String>,
    pub skill: Option<String>,
    pub page: Option<u32>,
}

impl AnalysisQuery {
    // Why: the default window ends at the next UTC midnight so today's drained
    // facts are inside it, and starts 30 days earlier — the same day
    // boundaries the daily snapshot buckets facts under.
    pub fn window(&self, now: DateTime<Utc>) -> Result<RetainedWindow, AdminError> {
        let end = self.end.unwrap_or_else(|| {
            (now.date_naive() + Duration::days(1))
                .and_time(NaiveTime::MIN)
                .and_utc()
        });
        let start = self.start.unwrap_or(end - Duration::days(30));
        if start >= end || end - start > Duration::days(366) {
            return Err(AdminError::BadRequest(
                "Choose a window between 1 day and 366 days".into(),
            ));
        }
        Ok(RetainedWindow { start, end })
    }

    pub fn resource(&self) -> Result<Option<ManagedResourceId>, AdminError> {
        let Some(raw) = self.resource.as_deref().map(str::trim) else {
            return Ok(None);
        };
        if raw.is_empty() {
            return Ok(None);
        }
        if raw.len() > 200 || raw.chars().any(|c| c.is_control() || c.is_whitespace()) {
            return Err(AdminError::BadRequest("Invalid resource id".into()));
        }
        Ok(Some(ManagedResourceId::new(raw)))
    }

    // Why: the skill drill-down is keyed the way the facts are — the hook's
    // plugin id plus its `<plugin>:<skill>` string — so a configured or
    // bundled skill with no managed resource still has a page.
    pub fn skill(&self) -> Result<Option<RetainedSkillRef>, AdminError> {
        let plugin = self.plugin.as_deref().map(str::trim).unwrap_or_default();
        let skill = self.skill.as_deref().map(str::trim).unwrap_or_default();
        if plugin.is_empty() && skill.is_empty() {
            return Ok(None);
        }
        let bad = |s: &str| {
            s.is_empty() || s.len() > 200 || s.chars().any(|c| c.is_control() || c.is_whitespace())
        };
        if bad(plugin) || bad(skill) {
            return Err(AdminError::BadRequest("Invalid skill reference".into()));
        }
        Ok(Some(RetainedSkillRef {
            plugin_id: PluginId::new(plugin),
            skill: skill.to_owned(),
        }))
    }

    #[must_use]
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    #[must_use]
    pub fn url(&self, page: u32, target: Option<&Drilldown>) -> String {
        let mut pairs = Vec::new();
        let (resource, plugin, skill) = match target {
            Some(Drilldown::Resource(id)) => (Some(id.clone()), None, None),
            Some(Drilldown::Skill(s)) => (
                None,
                Some(s.plugin_id.as_str().to_owned()),
                Some(s.skill.clone()),
            ),
            None => (None, None, None),
        };
        for (key, value) in [
            ("start", self.start.map(time::render)),
            ("end", self.end.map(time::render)),
            ("resource", resource),
            ("plugin", plugin),
            ("skill", skill),
        ] {
            if let Some(value) = value {
                pairs.push((key, value));
            }
        }
        if page > 1 {
            pairs.push(("page", page.to_string()));
        }
        let query = pairs
            .into_iter()
            .map(|(k, v)| format!("{k}={}", urlencoding::encode(&v)))
            .collect::<Vec<_>>()
            .join("&");
        if query.is_empty() {
            "/admin/analysis/skills".to_owned()
        } else {
            format!("/admin/analysis/skills?{query}")
        }
    }
}

#[must_use]
pub fn drilldown_url(resource: &ManagedResourceId, window: (&str, &str)) -> String {
    format!(
        "/admin/analysis/skills?resource={}&start={}&end={}",
        urlencoding::encode(resource.as_str()),
        urlencoding::encode(window.0),
        urlencoding::encode(window.1)
    )
}

/// What a page is narrowed to: a managed resource (the revision-keyed link
/// other pages use) or a skill identity (this page's own rows).
#[derive(Debug, Clone)]
pub enum Drilldown {
    Resource(String),
    Skill(RetainedSkillRef),
}
