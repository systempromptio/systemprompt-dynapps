//! Bounded request plan: the validated window, kind and filters, the YAML
//! index for the request, and the full inventory walk the table renders.
use super::{PortfolioQuery, range};
use crate::error::{AdminError, AdminResult};
use crate::repositories::analysis::inventory_index::InventoryIndex;
use crate::repositories::analysis::portfolio_snapshots::{PortfolioEntry, PortfolioFilter};
use crate::routes::managed_state::ManagedState;
use chrono::Utc;
use systemprompt::identifiers::InventoryEntryId;

pub(crate) struct Plan {
    pub(super) range: range::Range,
    pub(super) kind: String,
    pub(super) inventory_kind: &'static str,
    pub(super) search: String,
    // Why: empty when the reader pinned no marketplace, which is the grouped
    // every-marketplace view; the picker is a filter, never a guess.
    pub(super) filter_marketplace: String,
    pub(super) all_marketplaces: bool,
    pub(super) columns_all: bool,
    pub(super) index: InventoryIndex,
}

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

// Why: the one `marketplace` value that is not an id — it asks for the
// grouped, every-marketplace view.
pub(super) const ALL_MARKETPLACES: &str = "all";

fn clean(value: Option<String>) -> AdminResult<String> {
    let value = value.unwrap_or_default();
    if value.len() > 200 || value.chars().any(char::is_control) {
        return Err(AdminError::BadRequest(
            "Invalid inventory filter".to_owned(),
        ));
    }
    Ok(value.trim().to_owned())
}

pub(super) fn plan(query: PortfolioQuery, now: chrono::DateTime<Utc>) -> AdminResult<Plan> {
    let range = range::Range::parse(&query, now.date_naive())?;
    let kind = query.kind.unwrap_or_else(|| "skills".to_owned());
    let inventory_kind = inventory_kind(&kind)?;
    let requested = clean(query.marketplace)?;
    let all_marketplaces = requested.is_empty() || requested == ALL_MARKETPLACES;
    Ok(Plan {
        range,
        kind,
        inventory_kind,
        search: clean(query.search)?,
        filter_marketplace: if all_marketplaces {
            String::new()
        } else {
            requested
        },
        all_marketplaces,
        columns_all: clean(query.columns)? == "all",
        index: InventoryIndex::build(&crate::handlers::shared::get_services_path()?),
    })
}

// Why: the dense table renders every skill at once; the repository pages by
// canonical entry id, so the pages are walked to exhaustion here.
pub(super) async fn list_inventory(
    managed: &ManagedState,
    plan: &Plan,
) -> AdminResult<Vec<PortfolioEntry>> {
    let mut all = Vec::new();
    let mut after: Option<InventoryEntryId> = None;
    loop {
        let page = managed
            .portfolio
            .list(
                &managed.owner,
                &PortfolioFilter {
                    after: after.as_ref(),
                    kind: Some(plan.inventory_kind),
                    search: "",
                    window: plan.range.window,
                    limit: 100,
                },
            )
            .await?;
        let full = page.len() == 100;
        after = page.last().map(|item| item.entry.entry_id.clone());
        all.extend(page);
        if !full {
            return Ok(all);
        }
    }
}
