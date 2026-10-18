//! `/admin/analysis/conversations` breakdown — one row per bucket of the
//! breakdown the page is showing, over the same filtered set as the
//! conversations export.

use async_trait::async_trait;

use crate::handlers::ssr::analysis::conversations::export::export_rows;
use crate::repositories::analysis::conversations::ConversationBucketRow;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::export::model::{
    Cell, CellKind, Column, DataSet, ExportContext, Table, Window,
};
use systemprompt_web_admin_core::export::registry::DataSetRegistration;
use systemprompt_web_admin_core::export::window::time_range;
use systemprompt_web_admin_core::types::UserContext;

pub(crate) struct Breakdown;

inventory::submit! {
    DataSetRegistration {
        order: 28,
        dataset: &Breakdown,
    }
}

const BUCKET_COLUMNS: &[Column] = &[
    Column::new("bucket", "Bucket", CellKind::Text).group("Identity"),
    Column::new("user_id", "User", CellKind::Text)
        .optional()
        .group("Identity"),
    Column::new("conversations", "Conversations", CellKind::Integer).group("Volume"),
    Column::new("users", "Users", CellKind::Integer).group("Volume"),
    Column::new("turns", "Turns", CellKind::Integer).group("Volume"),
    Column::new("tool_calls", "Tool calls", CellKind::Integer).group("Tools & artifacts"),
    Column::new("tokens", "Tokens", CellKind::Integer).group("Tokens"),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money).group("Cost"),
    Column::new("errors", "Failed requests", CellKind::Integer).group("Volume"),
    Column::new("denied", "Denied", CellKind::Integer).group("Governance"),
    Column::new("achieved", "Achieved", CellKind::Integer).group("Judge"),
    Column::new("judged", "Judged", CellKind::Integer).group("Judge"),
    Column::new("completion_avg", "Mean completion", CellKind::Decimal)
        .optional()
        .group("Judge"),
    Column::new("top_category", "Top intent", CellKind::Text).group("Judge"),
];

fn bucket_row(r: &ConversationBucketRow) -> Vec<Cell> {
    vec![
        r.label.as_str().into(),
        Cell::opt_text(r.user_id.as_ref().map(UserId::as_str)),
        r.conversations.into(),
        r.users.into(),
        r.turns.into(),
        r.tool_calls.into(),
        r.total_tokens.into(),
        Cell::Money(r.total_cost_microdollars),
        r.errors.into(),
        r.denied.into(),
        r.achieved.into(),
        r.judged.into(),
        Cell::opt_decimal(r.completion_avg),
        Cell::opt_text(r.top_category.as_deref()),
    ]
}

#[async_trait]
impl DataSet for Breakdown {
    fn id(&self) -> &'static str {
        "analysis-conversation-breakdown"
    }
    fn title(&self) -> &'static str {
        "Conversation breakdown"
    }
    fn description(&self) -> &'static str {
        "One row per bucket of the breakdown the page is showing."
    }
    fn columns(&self) -> &'static [Column] {
        BUCKET_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    fn allows(&self, user: &UserContext) -> bool {
        user.has_scoped_console()
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let data = export_rows(ctx.pool, ctx.user, ctx.query()?, time_range(ctx)?, 1).await?;
        Ok(Table::complete(
            data.breakdown.iter().map(bucket_row).collect(),
        ))
    }
}
