//! `/admin/analysis/skills` and `/admin/analysis/skills/{key}` — every
//! skill's record over the window, and the conversations that invoked one
//! skill.

use async_trait::async_trait;
use serde::Deserialize;

use crate::error::{AdminError, AdminResult};
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::analysis::parse_skill_key;
use crate::repositories;
use crate::repositories::analysis::skills::{
    SkillConversationRow, SkillFactRow, SkillListFilter, SkillWindow,
    list_skill_conversations_paged, list_skill_facts,
};
use crate::repositories::scope::ScopeRequest;
use crate::types::UserContext;
use systemprompt::identifiers::{MarketplaceId, PluginId};

pub(crate) struct Skills;
pub(crate) struct SkillConversations;

// Why: the export dialog is opened from the page, so the request's query
// carries `format=`, `columns=` and the window beside the drill-down;
// `skill` is the `plugin:skill` key the page carries in its path.
#[derive(Debug, Default, Deserialize)]
struct Drilldown {
    skill: Option<String>,
}

async fn window(ctx: &ExportContext<'_>) -> AdminResult<SkillWindow> {
    let w = ctx.window()?;
    let request = ScopeRequest::from_query(ctx.user, None, None);
    let scope = repositories::scope::membership::get_subject_scope(ctx.pool, &request).await?;
    Ok(SkillWindow {
        start: w.from,
        end: w.to,
        subject_ids: scope.as_sql().map(<[String]>::to_vec),
    })
}

const SKILL_COLUMNS: &[Column] = &[
    Column::new("skill", "Skill", CellKind::Text),
    Column::new("plugin_id", "Plugin", CellKind::Text),
    Column::new("marketplace_id", "Marketplace", CellKind::Text),
    Column::new("invocations", "Invocations", CellKind::Integer),
    Column::new("slash", "Slash", CellKind::Integer).optional(),
    Column::new("tool", "Tool", CellKind::Integer).optional(),
    Column::new("attributed", "Attributed", CellKind::Integer).optional(),
    Column::new("users", "People", CellKind::Integer),
    Column::new("sessions", "Sessions", CellKind::Integer).optional(),
    Column::new("installs", "Installs", CellKind::Integer),
    Column::new("conversations", "Conversations", CellKind::Integer),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("turns", "Turns", CellKind::Integer).optional(),
    Column::new("input_tokens", "Input tokens", CellKind::Integer),
    Column::new("output_tokens", "Output tokens", CellKind::Integer),
    Column::new("cache_tokens", "Cache tokens", CellKind::Integer).optional(),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("errors", "Failed requests", CellKind::Integer),
    Column::new("denied", "Denied", CellKind::Integer),
    Column::new("tool_calls", "Tool calls", CellKind::Integer),
    Column::new("tool_calls_failed", "Tools failed", CellKind::Integer).optional(),
    Column::new("artifacts", "Artifacts", CellKind::Integer).optional(),
    Column::new("p95_latency_ms", "p95 latency (ms)", CellKind::Decimal).optional(),
    Column::new("models", "Models", CellKind::Text).optional(),
    Column::new("clients", "Clients", CellKind::Text).optional(),
    Column::new("judged", "Judged", CellKind::Integer),
    Column::new("completion_avg", "Mean completion", CellKind::Decimal),
    Column::new("achieved", "Achieved", CellKind::Integer).optional(),
    Column::new("first_used", "First used", CellKind::Timestamp),
    Column::new("last_used", "Last used", CellKind::Timestamp),
];

fn skill_row(r: &SkillFactRow) -> Vec<Cell> {
    vec![
        r.skill.as_str().into(),
        Cell::opt_text(r.plugin_id.as_ref().map(PluginId::as_str)),
        Cell::opt_text(r.marketplace_id.as_ref().map(MarketplaceId::as_str)),
        r.invocations.into(),
        r.slash.into(),
        r.tool.into(),
        r.attributed.into(),
        r.users.into(),
        r.sessions.into(),
        r.installs.into(),
        r.conversations.into(),
        r.requests.into(),
        r.turns.into(),
        r.input_tokens.into(),
        r.output_tokens.into(),
        r.cache_tokens.into(),
        Cell::Money(r.cost_microdollars),
        r.errors.into(),
        r.denied.into(),
        r.tool_calls.into(),
        r.tool_calls_failed.into(),
        r.artifacts.into(),
        Cell::opt_decimal(r.p95_latency_ms),
        Cell::list(&r.models),
        Cell::list(&r.clients),
        r.judged.into(),
        Cell::opt_decimal(r.completion_avg),
        r.achieved.into(),
        Cell::opt_time(Some(r.first_used)),
        Cell::opt_time(Some(r.last_used)),
    ]
}

const CONVERSATION_COLUMNS: &[Column] = &[
    Column::new("context_id", "Context", CellKind::Text),
    Column::new("client_session_id", "Harness session", CellKind::Text).optional(),
    Column::new("title", "Title", CellKind::Text),
    Column::new("user_id", "User", CellKind::Text),
    Column::new("display_name", "Name", CellKind::Text),
    Column::new("group_name", "Group", CellKind::Text),
    Column::new("project_name", "Project", CellKind::Text),
    Column::new("client", "Client", CellKind::Text),
    Column::new("model", "Model", CellKind::Text),
    Column::new("invocations", "Invocations", CellKind::Integer),
    Column::new("turns", "Turns", CellKind::Integer),
    Column::new("tool_calls", "Tool calls", CellKind::Integer),
    Column::new("errors", "Failed requests", CellKind::Integer),
    Column::new("denied", "Denied", CellKind::Integer),
    Column::new("tokens", "Tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("p95_latency_ms", "p95 latency (ms)", CellKind::Integer).optional(),
    Column::new("duration_seconds", "Duration (s)", CellKind::Integer),
    Column::new("skills", "Skills", CellKind::Text).optional(),
    Column::new("category", "Intent", CellKind::Text),
    Column::new("outcome", "Outcome", CellKind::Text),
    Column::new("completion", "Completion", CellKind::Integer),
    Column::new("first_at", "Started", CellKind::Timestamp),
    Column::new("last_at", "Last activity", CellKind::Timestamp),
];

fn conversation_row(r: &SkillConversationRow) -> Vec<Cell> {
    vec![
        r.context_id.as_str().into(),
        Cell::opt_text(r.client_session_id.as_deref()),
        Cell::opt_text(r.judge_title.as_deref().or(Some(r.title.as_str()))),
        r.user_id.as_str().into(),
        Cell::opt_text(r.display_name.as_deref()),
        Cell::opt_text(r.group_name.as_deref()),
        Cell::opt_text(r.project_name.as_deref()),
        r.client_kind.as_str().into(),
        Cell::opt_text(r.model.as_deref()),
        r.invocations.into(),
        r.turn_count.into(),
        r.tool_calls.into(),
        r.error_count.into(),
        r.gov_deny.into(),
        r.total_tokens.into(),
        Cell::Money(r.cost_microdollars),
        Cell::opt_int(r.p95_latency_ms),
        r.duration_seconds.into(),
        Cell::list(&r.skills),
        Cell::opt_text(r.category.as_deref()),
        Cell::opt_text(r.outcome.as_deref()),
        Cell::opt_int(r.completion),
        Cell::opt_time(Some(r.first_at)),
        Cell::opt_time(Some(r.last_at)),
    ]
}

#[async_trait]
impl DataSet for Skills {
    fn id(&self) -> &'static str {
        "analysis-skills"
    }
    fn title(&self) -> &'static str {
        "Skills"
    }
    fn columns(&self) -> &'static [Column] {
        SKILL_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Retained
    }
    fn allows(&self, user: &UserContext) -> bool {
        user.is_console
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let rows = list_skill_facts(
            ctx.pool,
            &window(ctx).await?,
            &SkillListFilter {
                limit: ctx.limit,
                ..SkillListFilter::default()
            },
        )
        .await?;
        let total = rows.first().map_or(0, |r| r.total);
        Ok(Table {
            rows: rows.iter().map(skill_row).collect(),
            total,
        })
    }
}

#[async_trait]
impl DataSet for SkillConversations {
    fn id(&self) -> &'static str {
        "analysis-skill-conversations"
    }
    fn title(&self) -> &'static str {
        "Skill conversations"
    }
    fn columns(&self) -> &'static [Column] {
        CONVERSATION_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Retained
    }
    fn allows(&self, user: &UserContext) -> bool {
        user.is_console
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let drill: Drilldown = ctx.query()?;
        let skill = drill
            .skill
            .as_deref()
            .ok_or_else(|| AdminError::BadRequest("A skill is required".into()))?;
        let skill = parse_skill_key(skill)?;
        let (rows, total) = list_skill_conversations_paged(
            ctx.pool,
            &window(ctx).await?,
            &skill.skill,
            ctx.limit,
            0,
        )
        .await?;
        Ok(Table {
            rows: rows.iter().map(conversation_row).collect(),
            total,
        })
    }
}
