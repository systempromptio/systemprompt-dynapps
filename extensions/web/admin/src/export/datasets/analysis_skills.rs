//! `/admin/analysis/skills` — retained skill usage, and the conversations
//! behind one skill or managed resource.

use async_trait::async_trait;
use serde::Deserialize;

use super::paged::collect_pages;
use crate::error::AdminResult;
use crate::export::model::{Cell, CellKind, Column, DataSet, ExportContext, Table, Window};
use crate::handlers::ssr::analysis::{AnalysisQuery, parse_skill_key};
use crate::repositories::analysis::retained::{
    self, RetainedConversationRow, RetainedSkillRow, RetainedWindow,
};

pub(crate) struct RetainedSkills;
pub(crate) struct SkillConversations;

// Why: the page's own query refuses unknown keys, and an export request
// carries `format=`, `columns=` and the window beside the drill-down — so
// the drill-down keys are read here and validated through the page's types.
// `skill` is the `plugin:skill` key the page carries in its path.
#[derive(Debug, Default, Deserialize)]
struct Drilldown {
    resource: Option<String>,
    skill: Option<String>,
}

impl Drilldown {
    fn page_query(&self) -> AnalysisQuery {
        AnalysisQuery {
            resource: self.resource.clone(),
            ..AnalysisQuery::default()
        }
    }
}

fn retained_window(ctx: &ExportContext<'_>) -> AdminResult<RetainedWindow> {
    let w = ctx.window()?;
    Ok(RetainedWindow {
        start: w.from,
        end: w.to,
    })
}

const SKILL_COLUMNS: &[Column] = &[
    Column::new("plugin_id", "Plugin", CellKind::Text),
    Column::new("skill", "Skill", CellKind::Text),
    Column::new("marketplace_id", "Marketplace", CellKind::Text),
    Column::new("resource_id", "Resource", CellKind::Text).optional(),
    Column::new("resource_key", "Resource key", CellKind::Text).optional(),
    Column::new("kind", "Kind", CellKind::Text).optional(),
    Column::new("source", "Source", CellKind::Text).optional(),
    Column::new("source_hash", "Source hash", CellKind::Text).optional(),
    Column::new("invocations", "Invocations", CellKind::Integer),
    Column::new("revision_verified", "Revision verified", CellKind::Integer),
    Column::new("conversations", "Conversations", CellKind::Integer),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("failed", "Failed", CellKind::Integer),
    Column::new("tokens", "Tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("artifacts", "Artifacts", CellKind::Integer),
    Column::new(
        "structured_artifacts",
        "Structured artifacts",
        CellKind::Integer,
    )
    .optional(),
];

fn skill_row(r: &RetainedSkillRow) -> Vec<Cell> {
    vec![
        r.plugin_id.as_str().into(),
        r.skill.as_str().into(),
        Cell::opt_text(r.marketplace_id.as_ref().map(|m| m.as_str())),
        Cell::opt_text(r.resource_id.as_ref().map(|m| m.as_str())),
        Cell::opt_text(r.resource_key.as_deref()),
        Cell::opt_text(r.kind.as_deref()),
        Cell::opt_text(r.source.as_deref()),
        Cell::opt_text(r.source_hash.as_deref()),
        r.invocations.into(),
        r.revision_verified.into(),
        r.conversations.into(),
        r.requests.into(),
        r.failed.into(),
        r.tokens.into(),
        Cell::Money(r.cost),
        r.artifacts.into(),
        r.structured_artifacts.into(),
    ]
}

const CONVERSATION_COLUMNS: &[Column] = &[
    Column::new("first_at", "Started", CellKind::Timestamp),
    Column::new("last_at", "Last activity", CellKind::Timestamp),
    Column::new("session_id", "Session", CellKind::Text),
    Column::new("user_id", "User", CellKind::Text),
    Column::new("host", "Host", CellKind::Text),
    Column::new("invocations", "Invocations", CellKind::Integer),
    Column::new("requests", "Requests", CellKind::Integer),
    Column::new("failed", "Failed", CellKind::Integer),
    Column::new("tokens", "Tokens", CellKind::Integer),
    Column::new("cost_usd", "Cost (USD)", CellKind::Money),
    Column::new("assessment", "Assessment", CellKind::Text),
    Column::new("score", "Score", CellKind::Decimal),
    Column::new("artifacts", "Artifacts", CellKind::Integer),
];

fn conversation_row(r: &RetainedConversationRow) -> Vec<Cell> {
    vec![
        r.first_at.into(),
        r.last_at.into(),
        r.session_id.as_str().into(),
        r.user_id.as_str().into(),
        Cell::opt_text(r.host.as_deref()),
        r.invocations.into(),
        r.requests.into(),
        r.failed.into(),
        r.tokens.into(),
        Cell::Money(r.cost),
        Cell::opt_text(r.assessment_status.as_deref()),
        Cell::opt_decimal(
            r.score_millionths
                .map(|s| crate::numeric::to_f64(s) / 1_000_000.0),
        ),
        r.artifacts.into(),
    ]
}

#[async_trait]
impl DataSet for RetainedSkills {
    fn id(&self) -> &'static str {
        "analysis-skills"
    }
    fn title(&self) -> &'static str {
        "Skill usage"
    }
    fn columns(&self) -> &'static [Column] {
        SKILL_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Retained
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let window = retained_window(ctx)?;
        let owner = &ctx.managed.owner;
        let (rows, total) = collect_pages(ctx.limit, retained::PAGE_SIZE, |page| {
            retained::list_retained_skills(ctx.pool, owner, window, page)
        })
        .await?;
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
        "Conversations using the skill"
    }
    fn columns(&self) -> &'static [Column] {
        CONVERSATION_COLUMNS
    }
    fn window(&self) -> Window {
        Window::Retained
    }
    async fn load(&self, ctx: &ExportContext<'_>) -> AdminResult<Table> {
        let window = retained_window(ctx)?;
        let owner = &ctx.managed.owner;
        let drilldown = ctx.query::<Drilldown>()?;
        let query = drilldown.page_query();
        let (rows, total) = if let Some(resource) = query.resource()? {
            collect_pages(ctx.limit, retained::PAGE_SIZE, |page| {
                retained::list_retained_conversations(ctx.pool, owner, &resource, window, page)
            })
            .await?
        } else if let Some(key) = drilldown.skill.as_deref() {
            let skill = parse_skill_key(key)?;
            collect_pages(ctx.limit, retained::PAGE_SIZE, |page| {
                retained::list_retained_skill_conversations(ctx.pool, owner, &skill, window, page)
            })
            .await?
        } else {
            return Err(crate::error::AdminError::BadRequest(
                "Name a resource or a plugin:skill key to export its conversations".to_owned(),
            ));
        };
        Ok(Table {
            rows: rows.iter().map(conversation_row).collect(),
            total,
        })
    }
}
