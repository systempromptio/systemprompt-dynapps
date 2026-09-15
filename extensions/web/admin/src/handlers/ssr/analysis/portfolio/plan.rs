//! Bounded filters retain UTC ranges and never infer historical audience.
use super::{PortfolioQuery, range};
use crate::error::{AdminError, AdminResult};
use crate::repositories::analysis::inventory_index::InventoryIndex;
use chrono::Utc;
use systemprompt::identifiers::InventoryEntryId;

fn inventory_kind(kind: &str) -> AdminResult<&'static str> {
    Ok(match kind {
        "skills" => "skill",
        "plugins" => "plugin",
        "marketplaces" => "marketplace",
        "rules" => "rule",
        "hooks" => "hook",
        "artifacts" => "artifact",
        "agents" => "agent",
        "mcp" => "mcp",
        _ => return Err(AdminError::BadRequest("Invalid inventory kind".to_owned())),
    })
}

fn clean(value: Option<String>) -> AdminResult<String> {
    let value = value.unwrap_or_default();
    if value.len() > 200 || value.chars().any(char::is_control) {
        return Err(AdminError::BadRequest(
            "Invalid inventory filter".to_owned(),
        ));
    }
    Ok(value.trim().to_owned())
}

pub(super) struct Plan {
    pub(super) range: range::Range,
    pub(super) kind: String,
    pub(super) inventory_kind: &'static str,
    pub(super) search: String,
    pub(super) filter_marketplace: String,
    pub(super) filter_group: String,
    pub(super) after: Option<InventoryEntryId>,
    pub(super) index: InventoryIndex,
}

pub(super) fn plan(query: PortfolioQuery, now: chrono::DateTime<Utc>) -> AdminResult<Plan> {
    let range = range::Range::parse(&query, now.date_naive())?;
    let kind = query.kind.unwrap_or_else(|| "skills".to_owned());
    let inventory_kind = inventory_kind(&kind)?;
    let filter_marketplace = clean(query.marketplace)?;
    let filter_group = clean(query.group)?;
    if query.after.as_ref().is_some_and(|cursor| {
        cursor.as_str().len() > 200 || cursor.as_str().chars().any(char::is_control)
    }) {
        return Err(AdminError::BadRequest(
            "Invalid inventory cursor".to_owned(),
        ));
    }
    let index = InventoryIndex::build(&crate::handlers::shared::get_services_path()?);
    Ok(Plan {
        range,
        kind,
        inventory_kind,
        search: clean(query.search)?,
        filter_marketplace,
        filter_group,
        after: query.after,
        index,
    })
}
