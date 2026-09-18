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

// Why: the ticked rows' keys, comma-joined by the export dialog.
#[derive(Debug, Default, serde::Deserialize)]
struct SkillIdsQuery {
    ids: Option<String>,
}

impl SkillIdsQuery {
    fn skills(&self) -> Option<Vec<String>> {
        let keys: Vec<String> = self
            .ids
            .as_deref()?
            .split(',')
            .map(str::trim)
            .filter(|k| !k.is_empty())
            .map(str::to_owned)
            .collect();
        (!keys.is_empty()).then_some(keys)
    }
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
    Column::new("skill", "Skill", CellKind::Text).group("Identity"),
    Column::new("plugin_id", "Plugin", CellKind::Text).group("Identity"),
    Column::new("marketplace_id", "Marketplace", CellKind::Text).group("Identity"),
    Column::new("invocations", "Invocations", CellKind::Integer).group("Volume"),
    Column::new("slash", "Slash", CellKind::Integer)
        .optional()
        .group("Volume"),
    Column::new("tool", "Tool", CellKind::Integer)
        .optional()
        .group("Volume"),
    Column::new("attributed", "Attributed", CellKind::Integer)
        .optional()
        .group("Volume"),
    Column::new("users", "People", CellKind::Integer).group("Volume"),
    Column::new("sessions", "Sessions", CellKind::Integer)
        .optional()
        .group("Volume"),
    Column::new("installs", "Installs", CellKind::Integer).group("Volume"),
    Column::new("conversations", "Conversations", CellKind::Integer).group("Volume"),
    Column::new("requests", "Requests", CellKind::Integer).group("Volume"),
    Column::new("turns", "Turns", CellKind::Integer)
        .optional()
        .group("Volume"),
    Column::new("input_tokens", "Input tokens", CellKind::Integer).group("Tokens"),
    Column::new("output_tokens", "Output tokens", CellKind::Integer).group("Tokens"),
    Column::new("cache_tokens", "Cache tokens", CellKind::Integer)
        .optional()
        .group("Tokens"),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money).group("Cost"),
    Column::new("errors", "Failed requests", CellKind::Integer).group("Volume"),
    Column::new("denied", "Denied", CellKind::Integer).group("Governance"),
    Column::new("tool_calls", "Tool calls", CellKind::Integer).group("Tools & artifacts"),
    Column::new("tool_calls_failed", "Tools failed", CellKind::Integer)
        .optional()
        .group("Tools & artifacts"),
    Column::new("artifacts", "Artifacts", CellKind::Integer)
        .optional()
        .group("Tools & artifacts"),
    Column::new("p95_latency_ms", "p95 latency (ms)", CellKind::Decimal)
        .optional()
        .group("Volume"),
    Column::new("models", "Models", CellKind::Text)
        .optional()
        .group("Identity"),
    Column::new("clients", "Clients", CellKind::Text)
        .optional()
        .group("Identity"),
    Column::new("judged", "Judged", CellKind::Integer).group("Judge"),
    Column::new("completion_avg", "Mean completion", CellKind::Decimal).group("Judge"),
    Column::new("achieved", "Achieved", CellKind::Integer)
        .optional()
        .group("Judge"),
    Column::new("first_used", "First used", CellKind::Timestamp).group("Identity"),
    Column::new("last_used", "Last used", CellKind::Timestamp).group("Identity"),
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
    Column::new("context_id", "Context", CellKind::Text).group("Identity"),
    Column::new("client_session_id", "Harness session", CellKind::Text)
        .optional()
        .group("Identity"),
    Column::new("title", "Title", CellKind::Text).group("Identity"),
    Column::new("user_id", "User", CellKind::Text).group("Identity"),
    Column::new("display_name", "Name", CellKind::Text).group("Identity"),
    Column::new("group_name", "Group", CellKind::Text).group("Identity"),
    Column::new("project_name", "Project", CellKind::Text).group("Identity"),
    Column::new("client", "Client", CellKind::Text).group("Identity"),
    Column::new("model", "Model", CellKind::Text).group("Identity"),
    Column::new("invocations", "Invocations", CellKind::Integer).group("Volume"),
    Column::new("turns", "Turns", CellKind::Integer).group("Volume"),
    Column::new("tool_calls", "Tool calls", CellKind::Integer).group("Tools & artifacts"),
    Column::new("errors", "Failed requests", CellKind::Integer).group("Volume"),
    Column::new("denied", "Denied", CellKind::Integer).group("Governance"),
    Column::new("tokens", "Tokens", CellKind::Integer).group("Tokens"),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money).group("Cost"),
    Column::new("p95_latency_ms", "p95 latency (ms)", CellKind::Integer)
        .optional()
        .group("Volume"),
    Column::new("duration_seconds", "Duration (s)", CellKind::Integer).group("Volume"),
    Column::new("skills", "Skills", CellKind::Text)
        .optional()
        .group("Tools & artifacts"),
    Column::new("category", "Intent", CellKind::Text).group("Judge"),
    Column::new("outcome", "Outcome", CellKind::Text).group("Judge"),
    Column::new("completion", "Completion", CellKind::Integer).group("Judge"),
    Column::new("first_at", "Started", CellKind::Timestamp).group("Identity"),
    Column::new("last_at", "Last activity", CellKind::Timestamp).group("Identity"),
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
    fn description(&self) -> &'static str {
        "One row per skill invoked in the window, with its conversations' figures."
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
        let picked: SkillIdsQuery = ctx.query()?;
        let rows = list_skill_facts(
            ctx.pool,
            &window(ctx).await?,
            &SkillListFilter {
                limit: ctx.limit,
                skills: picked.skills(),
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
    fn description(&self) -> &'static str {
        "One row per conversation that invoked the skill."
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
