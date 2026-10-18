//! The finance half of the month-end reports: provider and model cost for the
//! month. These are the only report tables that carry a cost column, so they
//! never leave the platform team.

use async_trait::async_trait;

use super::{InternalModels, InternalProviders, month};
use crate::repositories::reports::internal;
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::export::model::{
    Cell, CellKind, Column, DataSet, ExportContext, Table, Window,
};

const SUPPLIER_COLUMNS: &[Column] = &[
    Column::new("key", "Supplier", CellKind::Text),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("tokens", "Tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
];

fn supplier_rows(rows: &[internal::SupplierMonthCost]) -> Table {
    Table::complete(
        rows.iter()
            .map(|r| {
                vec![
                    r.key.as_str().into(),
                    r.requests.into(),
                    r.tokens.into(),
                    Cell::Money(r.cost_microdollars),
                ]
            })
            .collect(),
    )
}

dataset!(
    InternalProviders,
    "report-internal-providers",
    "Provider cost for the month",
    "One row per provider for the month: requests, tokens and cost.",
    SUPPLIER_COLUMNS,
    |ctx| {
        let m = month(ctx)?;
        Ok(supplier_rows(
            &internal::list_provider_month_costs(ctx.pool, m.from, m.to).await?,
        ))
    }
);

dataset!(
    InternalModels,
    "report-internal-models",
    "Model cost for the month",
    "One row per model for the month: requests, tokens and cost.",
    SUPPLIER_COLUMNS,
    |ctx| {
        let m = month(ctx)?;
        Ok(supplier_rows(
            &internal::list_model_month_costs(ctx.pool, m.from, m.to).await?,
        ))
    }
);
