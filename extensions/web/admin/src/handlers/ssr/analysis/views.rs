//! Row views for the retained skill drill-down: a managed resource resolved
//! to the plugin and marketplaces that deliver it, and one conversation with
//! its session link, spend and worded assessment.
use crate::repositories::analysis::inventory_index::InventoryIndex;
use crate::repositories::analysis::retained::{RetainedConversationRow, RetainedResourceRow};
use serde::Serialize;
use systemprompt::identifiers::PluginId;

pub(crate) fn dollars(microdollars: i64) -> String {
    crate::handlers::ssr::format::format_cost(microdollars)
}

#[derive(Debug, Clone, Serialize)]
pub struct RetainedResourceView {
    pub row: RetainedResourceRow,
    pub href: String,
    pub cost: String,
    pub plugin_id: Option<PluginId>,
    pub plugin_name: Option<String>,
    pub marketplaces: Vec<String>,
    pub inventory_href: String,
}

impl RetainedResourceView {
    // Why: the inventory index keys plugins by the skill's resource key, so a
    // managed resource resolves to its plugin and marketplaces by that key;
    // the inventory page filters by the same key.
    #[must_use]
    pub fn new(row: RetainedResourceRow, href: String, index: &InventoryIndex) -> Self {
        let plugin = index
            .plugins_by_skill
            .get(&row.resource_key)
            .and_then(|list| list.first())
            .map(|p| (p.0.clone(), p.1.clone()));
        let marketplaces = plugin
            .as_ref()
            .and_then(|p| index.marketplaces_by_plugin.get(&p.0))
            .map(|list| list.iter().map(|m| m.name.clone()).collect())
            .unwrap_or_default();
        Self {
            inventory_href: format!(
                "/admin/analysis/resources?search={}",
                urlencoding::encode(&row.resource_key)
            ),
            plugin_id: plugin.as_ref().map(|p| PluginId::new(&p.0)),
            plugin_name: plugin.map(|p| p.1),
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
