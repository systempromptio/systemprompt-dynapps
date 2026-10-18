//! The query string of `/admin/analysis/skills`: which tab is open, the
//! window and scope it narrows to, and the links that keep that scope while
//! changing one part of it.

use serde::Deserialize;

use crate::repositories::analysis::skills::SkillSort;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::ssr::analysis_urls::ANALYSIS_SKILLS_URL;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SkillsTab {
    Overview,
    Activity,
    Skills,
}

impl SkillsTab {
    pub(super) const ALL: [Self; 3] = [Self::Overview, Self::Activity, Self::Skills];

    pub(super) fn parse(value: Option<&str>) -> AdminResult<Self> {
        Ok(match value.unwrap_or("overview") {
            "overview" => Self::Overview,
            "activity" => Self::Activity,
            "skills" => Self::Skills,
            _ => return Err(AdminError::BadRequest("Unknown skills tab".to_owned())),
        })
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Activity => "activity",
            Self::Skills => "skills",
        }
    }

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Activity => "Activity",
            Self::Skills => "Skills",
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SkillsQuery {
    pub days: Option<i64>,
    pub tab: Option<String>,
    pub marketplace: Option<String>,
    pub client: Option<String>,
    pub search: Option<String>,
    pub skill: Option<String>,
    pub sort: Option<String>,
    pub group: Option<String>,
    pub project: Option<String>,
}

impl SkillsQuery {
    pub(super) const fn days(&self) -> i64 {
        match self.days {
            Some(7) => 7,
            Some(90) => 90,
            Some(365) => 365,
            _ => 30,
        }
    }

    pub(super) fn trimmed(value: Option<&str>) -> Option<String> {
        value
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_owned)
    }

    pub(super) fn marketplace(&self) -> Option<String> {
        Self::trimmed(self.marketplace.as_deref())
    }

    pub(super) fn search(&self) -> Option<String> {
        Self::trimmed(self.search.as_deref()).or_else(|| Self::trimmed(self.skill.as_deref()))
    }

    pub(super) fn sort(&self) -> SkillSort {
        SkillSort::parse_skill_sort(self.sort.as_deref())
    }

    pub(super) fn tab(&self) -> AdminResult<SkillsTab> {
        SkillsTab::parse(self.tab.as_deref())
    }

    pub(super) fn link_without(&self, tab: SkillsTab, drop: &[&str]) -> String {
        let keep = |name: &str, value: &Option<String>| {
            (!drop.contains(&name)).then(|| value.clone()).flatten()
        };
        let cleared = Self {
            days: self.days,
            tab: self.tab.clone(),
            marketplace: keep("marketplace", &self.marketplace),
            client: keep("client", &self.client),
            search: keep("search", &self.search),
            skill: keep("search", &self.skill),
            sort: keep("sort", &self.sort),
            group: self.group.clone(),
            project: self.project.clone(),
        };
        cleared.link(tab, None, None)
    }

    pub(super) fn link_marketplace(&self, tab: SkillsTab, marketplace: Option<&str>) -> String {
        let scoped = Self {
            days: self.days,
            tab: self.tab.clone(),
            marketplace: marketplace.map(str::to_owned),
            client: self.client.clone(),
            search: self.search.clone(),
            skill: self.skill.clone(),
            sort: self.sort.clone(),
            group: self.group.clone(),
            project: self.project.clone(),
        };
        scoped.link(tab, None, None)
    }

    pub(super) fn link(
        &self,
        tab: SkillsTab,
        days: Option<i64>,
        sort: Option<SkillSort>,
    ) -> String {
        let mut pairs: Vec<(&str, String)> = Vec::new();
        if tab != SkillsTab::Overview {
            pairs.push(("tab", tab.as_str().to_owned()));
        }
        let days = days.unwrap_or_else(|| self.days());
        if days != 30 {
            pairs.push(("days", days.to_string()));
        }
        let sort = sort.unwrap_or_else(|| self.sort());
        if sort != SkillSort::Invocations {
            pairs.push(("sort", sort.as_str().to_owned()));
        }
        for (key, value) in [
            ("marketplace", self.marketplace()),
            ("client", Self::trimmed(self.client.as_deref())),
            ("search", self.search()),
            ("group", Self::trimmed(self.group.as_deref())),
            ("project", Self::trimmed(self.project.as_deref())),
        ] {
            if let Some(value) = value {
                pairs.push((key, value));
            }
        }
        let qs = pairs
            .iter()
            .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        if qs.is_empty() {
            ANALYSIS_SKILLS_URL.to_owned()
        } else {
            format!("{ANALYSIS_SKILLS_URL}?{qs}")
        }
    }
}
