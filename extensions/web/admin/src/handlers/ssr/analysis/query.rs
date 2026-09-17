//! The skill page's query: the UTC window, the optional narrowing to the
//! revision-verified evidence of one managed resource, and the page — with
//! the URL builder that renders them back onto `/admin/analysis/skills/{key}`.
//!
//! The skill itself is never a query parameter: its `plugin:skill` key is
//! the path, the same identity the invocation facts carry.

use chrono::{DateTime, Duration, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::{ManagedResourceId, PluginId};

use super::time::{self, WindowBound};
use crate::error::AdminError;
use crate::handlers::ssr::analysis_urls::analysis_skill_url;
use crate::repositories::analysis::retained::{RetainedSkillRef, RetainedWindow};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisQuery {
    #[serde(default)]
    pub start: WindowBound,
    #[serde(default)]
    pub end: WindowBound,
    pub resource: Option<String>,
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

    #[must_use]
    pub fn page(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }

    #[must_use]
    pub fn url(&self, page: u32, skill_key: &str) -> String {
        let mut pairs = Vec::new();
        for (key, value) in [
            ("start", self.start.map(time::render)),
            ("end", self.end.map(time::render)),
            ("resource", self.resource.clone()),
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
        let base = analysis_skill_url(skill_key);
        if query.is_empty() {
            base
        } else {
            format!("{base}?{query}")
        }
    }
}

/// The skill identity the facts key on, parsed from a `plugin:skill` path
/// segment: the plugin is the prefix, the key is kept whole.
pub fn parse_skill_key(raw: &str) -> Result<RetainedSkillRef, AdminError> {
    let key = raw.trim();
    let bad = key.is_empty()
        || key.len() > 200
        || key.chars().any(|c| c.is_control() || c.is_whitespace());
    let Some((plugin, skill)) = key.split_once(':') else {
        return Err(AdminError::BadRequest(
            "A skill is addressed as plugin:skill".into(),
        ));
    };
    if bad || plugin.is_empty() || skill.is_empty() {
        return Err(AdminError::BadRequest("Invalid skill reference".into()));
    }
    Ok(RetainedSkillRef {
        plugin_id: PluginId::new(plugin),
        skill: key.to_owned(),
    })
}

/// The link every other page uses to reach a skill's analysis: the skill's
/// key in the path, the window and, for a bound entry, the managed resource
/// that narrows the page to revision-verified evidence.
#[must_use]
pub fn drilldown_url(
    skill_key: &str,
    resource: Option<&ManagedResourceId>,
    window: (&str, &str),
) -> String {
    let mut url = format!(
        "{}?start={}&end={}",
        analysis_skill_url(skill_key),
        urlencoding::encode(window.0),
        urlencoding::encode(window.1)
    );
    if let Some(resource) = resource {
        url.push_str("&resource=");
        url.push_str(&urlencoding::encode(resource.as_str()));
    }
    url
}
