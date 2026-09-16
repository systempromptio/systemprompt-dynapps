//! Row views for the retained skill drill-down: one skill as the facts key
//! it, resolved to the plugin name and marketplaces the inventory knows, and
//! one conversation with its session link, spend and worded assessment.
use crate::repositories::analysis::inventory_index::InventoryIndex;
use crate::repositories::analysis::retained::{RetainedConversationRow, RetainedSkillRow};
use serde::Serialize;

pub(crate) fn dollars(microdollars: i64) -> String {
    crate::handlers::ssr::format::format_cost(microdollars)
}

#[derive(Debug, Clone, Serialize)]
pub struct RetainedSkillView {
    pub row: RetainedSkillRow,
    pub href: String,
    pub cost: String,
    pub skill_name: String,
    pub plugin_name: String,
    pub marketplaces: Vec<String>,
    pub source_label: String,
    pub source_hash_short: Option<String>,
    pub inventory_href: String,
}

impl RetainedSkillView {
    // Why: the fact carries the hook's ids; the inventory index resolves the
    // plugin's display name and the marketplaces that include it, with the
    // fact's own marketplace as the fallback when the index (this tree) does
    // not know a bundled plugin.
    #[must_use]
    pub fn new(row: RetainedSkillRow, href: String, index: &InventoryIndex) -> Self {
        let skill_name = row
            .skill
            .split_once(':')
            .map_or_else(|| row.skill.clone(), |(_, s)| s.to_owned());
        let plugin_name = index
            .plugins_by_skill
            .get(&skill_name.replace('-', "_"))
            .and_then(|list| list.iter().find(|p| p.0 == row.plugin_id.as_str()))
            .map_or_else(|| row.plugin_id.as_str().to_owned(), |p| p.1.clone());
        let mut marketplaces: Vec<String> = index
            .marketplaces_by_plugin
            .get(row.plugin_id.as_str())
            .map(|list| list.iter().map(|m| m.name.clone()).collect())
            .unwrap_or_default();
        if marketplaces.is_empty()
            && let Some(m) = row.marketplace_id.as_ref()
        {
            marketplaces.push(m.as_str().to_owned());
        }
        Self {
            inventory_href: format!(
                "/admin/analysis/resources?search={}",
                urlencoding::encode(&skill_name)
            ),
            source_label: row
                .source
                .clone()
                .unwrap_or_else(|| "unrecorded".to_owned()),
            source_hash_short: row
                .source_hash
                .as_ref()
                .map(|h| h.chars().take(12).collect()),
            skill_name,
            plugin_name,
            marketplaces,
            cost: dollars(row.cost),
            href,
            row,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RetainedConversationView {
    pub row: RetainedConversationRow,
    pub href: String,
    pub cost: String,
    pub host: String,
    pub first: String,
    pub last: String,
    pub assessment: String,
}

impl RetainedConversationView {
    #[must_use]
    pub fn new(row: RetainedConversationRow) -> Self {
        let assessment = match (row.assessment_status.as_deref(), row.score_millionths) {
            (Some("scored"), Some(score)) => format!("scored {}%", score / 10_000),
            (Some("scored"), None) => "scored".to_owned(),
            (Some(status), _) => status.to_owned(),
            (None, _) => "not assessed".to_owned(),
        };
        Self {
            href: format!(
                "/admin/sessions/{}",
                urlencoding::encode(row.session_id.as_str())
            ),
            cost: dollars(row.cost),
            host: row
                .host
                .clone()
                .unwrap_or_else(|| "unknown host".to_owned()),
            first: row.first_at.format("%Y-%m-%d %H:%M").to_string(),
            last: row.last_at.format("%Y-%m-%d %H:%M").to_string(),
            assessment,
            row,
        }
    }
}
