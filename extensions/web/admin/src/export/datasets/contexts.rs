//! `/admin/contexts` — activity by person: every conversation, or one row
//! per person with their totals.

use async_trait::async_trait;

use super::requests::time_range;
use super::sessions::conversation_row;
use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::ssr_skills_contexts::export::export_rows;
use crate::repositories::analytics::conversation_rows::{
    ConversationPageMode, UserConversationSummary,
};

pub(crate) struct Contexts;
pub(crate) struct People;

const PEOPLE_COLUMNS: &[Column] = &[
    Column::new("user_id", "User", CellKind::Text),
    Column::new("display_name", "Name", CellKind::Text),
    Column::new("conversations", "Conversations", CellKind::Integer),
    Column::new("turns", "Turns", CellKind::Integer),
    Column::new("side_calls", "Side calls", CellKind::Integer),
    Column::new("tokens", "Tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("last_at", "Last activity", CellKind::Timestamp),
    Column::new("models", "Models", CellKind::Text),
    Column::new("clients", "Clients", CellKind::Text).optional(),
    Column::new("latest_title", "Latest conversation", CellKind::Text).optional(),
];

fn person_row(r: &UserConversationSummary) -> Vec<Cell> {
    let models: Vec<String> = r.models.iter().map(|m| m.model.clone()).collect();
    let clients: Vec<String> = r.clients.iter().map(|c| c.client.clone()).collect();
    vec![
        r.user_id.as_str().into(),
        Cell::opt_text(r.display_name.as_deref()),
        r.conversation_count.into(),
        r.turn_count.into(),
        r.side_call_count.into(),
        r.total_tokens.into(),
        Cell::Money(r.total_cost_microdollars),
        Cell::opt_time(r.last_at),
        Cell::list(&models),
        Cell::list(&clients),
        Cell::opt_text(r.latest.as_ref().map(|c| c.title.as_str())),
    ]
}

#[async_trait]
impl DataSet for Contexts {
    fn id(&self) -> &'static str {
        "contexts"
    }
    fn title(&self) -> &'static str {
        "Conversations by person"
    }
    fn columns(&self) -> &'static [Column] {
        super::sessions::COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let result = export_rows(
            ctx.pool,
            ctx.user,
            ctx.query()?,
            time_range(ctx)?,
            ctx.limit,
            ConversationPageMode::All,
        )
        .await?;
        Ok(Table {
            rows: result.conversations.iter().map(conversation_row).collect(),
            total: result.totals.conversations,
        })
    }
}

#[async_trait]
impl DataSet for People {
    fn id(&self) -> &'static str {
        "people"
    }
    fn title(&self) -> &'static str {
        "People"
    }
    fn columns(&self) -> &'static [Column] {
        PEOPLE_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Live
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let result = export_rows(
            ctx.pool,
            ctx.user,
            ctx.query()?,
            time_range(ctx)?,
            ctx.limit,
            ConversationPageMode::Users,
        )
        .await?;
        Ok(Table {
            rows: result.user_summaries.iter().map(person_row).collect(),
            total: result.totals.users,
        })
    }
}
