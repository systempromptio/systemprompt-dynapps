//! `/admin/analysis/skills` — the resource inventory with its retained
//! usage figures.

use async_trait::async_trait;
use serde::Deserialize;

use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::analysis::portfolio::PortfolioQuery;
use crate::handlers::ssr::analysis::portfolio::export::export_rows;
use crate::handlers::ssr::analysis::portfolio::rows::InventoryRow;
use crate::types::UserContext;
use systemprompt::identifiers::{ManagedResourceId, PluginId};
use systemprompt::models::feedback::inventory::InventoryAvailability;

pub(crate) struct Resources;

// Why: the page's own query refuses unknown keys, and an export request
// carries `format=` and `columns=` beside the filters.
#[derive(Debug, Default, Deserialize)]
struct Filters {
    kind: Option<String>,
    search: Option<String>,
    marketplace: Option<String>,
}

const COLUMNS: &[Column] = &[
    Column::new("resource_key", "Resource", CellKind::Text),
    Column::new("kind", "Kind", CellKind::Text),
    Column::new("plugin_id", "Plugin", CellKind::Text),
    Column::new("plugin_name", "Plugin name", CellKind::Text).optional(),
    Column::new("marketplaces", "Marketplaces", CellKind::Text),
    Column::new("version", "Version", CellKind::Text),
    Column::new("availability", "Availability", CellKind::Text),
    Column::new("health", "Health", CellKind::Text),
    Column::new("health_reason", "Health reason", CellKind::Text),
    Column::new("usage", "Usage status", CellKind::Text).optional(),
    Column::new("users_with_access", "Users with access", CellKind::Integer),
    Column::new("users", "Users", CellKind::Integer),
    Column::new("invocations", "Invocations", CellKind::Integer),
    Column::new(
        "verified_invocations",
        "Verified invocations",
        CellKind::Integer,
    ),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("failed_requests", "Failed requests", CellKind::Integer),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("p95_ms", "p95 (ms)", CellKind::Decimal),
    Column::new("resource_id", "Resource id", CellKind::Text).optional(),
    Column::new("diagnostic", "Diagnostic", CellKind::Text).optional(),
];

const fn availability(value: InventoryAvailability) -> &'static str {
    match value {
        InventoryAvailability::Available => "available",
        InventoryAvailability::Unavailable => "unavailable",
        InventoryAvailability::Withdrawn => "withdrawn",
        InventoryAvailability::Conflicting => "conflicting",
    }
}

fn row(r: &InventoryRow) -> Vec<Cell> {
    let marketplaces: Vec<String> = r.marketplaces.iter().map(|m| m.id.clone()).collect();
    let f = &r.figures;
    vec![
        r.resource_key.as_str().into(),
        r.entry.kind.as_str().into(),
        Cell::opt_text(r.plugin_id.as_ref().map(PluginId::as_str)),
        Cell::opt_text(r.plugin_name.as_deref()),
        Cell::list(&marketplaces),
        Cell::opt_text(r.version.as_ref().map(|v| v.content_hash.as_str())),
        availability(r.entry.availability).into(),
        r.health.tone().into(),
        r.health_reason.into(),
        r.usage_status.into(),
        i64::try_from(r.users_with_access)
            .unwrap_or(i64::MAX)
            .into(),
        Cell::opt_int(r.users),
        f.invocations.into(),
        f.verified_invocations.into(),
        f.requests.into(),
        f.failed_requests.into(),
        f.input_tokens.into(),
        f.output_tokens.into(),
        Cell::Money(f.cost_microdollars),
        Cell::opt_decimal(
            f.p95_micros
                .map(|m| crate::numeric::to_f64(i64::try_from(m).unwrap_or(i64::MAX)) / 1000.0),
        ),
        Cell::opt_text(r.entry.resource_id.as_ref().map(ManagedResourceId::as_str)),
        Cell::opt_text(r.entry.diagnostic.as_deref()),
    ]
}

#[async_trait]
impl DataSet for Resources {
    fn id(&self) -> &'static str {
        "analysis-resources"
    }
    fn title(&self) -> &'static str {
        "Resources"
    }
    fn columns(&self) -> &'static [Column] {
        COLUMNS
    }
    fn window(&self) -> Window {
        Window::Days
    }
    fn allows(&self, user: &UserContext) -> bool {
        user.is_admin
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let filters: Filters = ctx.query()?;
        let query = PortfolioQuery {
            days: Some(ctx.window()?.days),
            kind: filters.kind,
            search: filters.search,
            marketplace: filters.marketplace,
            ..PortfolioQuery::default()
        };
        let rows = export_rows(ctx.managed, ctx.pool, ctx.user, query).await?;
        Ok(Table::complete(rows.iter().map(row).collect()))
    }
}
