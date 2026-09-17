//! Classified conversations — the repository behind
//! `/admin/analysis/conversations`.
//!
//! A conversation is a context; `conversation_metrics_for` (schema 27) rolls
//! its gateway requests up, `conversation_analyses` (schema 37) carries the
//! classifier's verdict, and `conversation_skill_uses` the skills the harness
//! hooks reported. One statement (`page.sql`) narrows the classified set,
//! then reads its totals, one breakdown dimension and one page of rows from
//! the same filtered set, so every number above the table describes the rows
//! inside it. Filtering, paging and sorting are bound parameters chosen by
//! `CASE` arms, never interpolated text.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

use crate::repositories::analytics::conversation_rows::ConversationSort;

/// Narrowing applied to the rows, totals and breakdown alike.
#[derive(Debug, Clone, Default)]
pub struct ConversationAnalysisFilter {
    // Why: the caller's resolved `SubjectScope::as_sql()`; `None` = every user.
    pub subject_ids: Option<Vec<String>>,
    pub user_id: Option<UserId>,
    pub category: Option<String>,
    pub outcome: Option<String>,
    pub skill: Option<String>,
    pub free_text: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

impl ConversationAnalysisFilter {
    fn free_text_pattern(&self) -> Option<String> {
        self.free_text
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("%{}%", s.replace('\\', "\\\\").replace('%', "\\%")))
    }
}

/// The dimension the breakdown table groups the filtered set by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BreakdownBy {
    #[default]
    Category,
    Group,
    Project,
    User,
    Outcome,
}

impl BreakdownBy {
    pub const ALL: [Self; 5] = [
        Self::Category,
        Self::Group,
        Self::Project,
        Self::User,
        Self::Outcome,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Category => "category",
            Self::Group => "group",
            Self::Project => "project",
            Self::User => "user",
            Self::Outcome => "outcome",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Category => "By category",
            Self::Group => "By group",
            Self::Project => "By project",
            Self::User => "By person",
            Self::Outcome => "By outcome",
        }
    }

    #[must_use]
    pub fn from_breakdown_param(value: Option<&str>) -> Self {
        match value {
            Some("group") => Self::Group,
            Some("project") => Self::Project,
            Some("user") => Self::User,
            Some("outcome") => Self::Outcome,
            _ => Self::Category,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversationAnalysisPage {
    pub sort: ConversationSort,
    pub descending: bool,
    pub limit: i64,
    pub offset: i64,
    pub breakdown: BreakdownBy,
}

/// One classified conversation as the page shows it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationAnalysisRow<Id = ContextId> {
    pub context_id: Id,
    pub title: String,
    pub user_id: Option<UserId>,
    pub display_name: Option<String>,
    pub session_id: Option<SessionId>,
    pub client_session_id: Option<String>,
    pub group_name: Option<String>,
    pub project_name: Option<String>,
    pub model: Option<String>,
    pub client_kind: String,
    pub turn_count: i64,
    pub tool_call_count: i64,
    pub error_count: i64,
    pub total_tokens: i64,
    pub total_cost_microdollars: i64,
    pub first_at: Option<DateTime<Utc>>,
    pub last_at: Option<DateTime<Utc>>,
    pub category: String,
    pub summary: Option<String>,
    pub tags: Vec<String>,
    pub skills_used: Vec<String>,
    pub skills_hooked: Vec<String>,
    pub outcome: Option<String>,
    pub confidence: Option<f32>,
    pub classified_at: Option<DateTime<Utc>>,
    pub judge_model: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ConversationAnalysisTotals {
    pub conversations: i64,
    pub users: i64,
    pub turns: i64,
    pub tool_calls: i64,
    pub total_tokens: i64,
    pub total_cost_microdollars: i64,
    pub achieved: i64,
    // Why: pending or failed rows in scope — what the page has not yet seen.
    pub unclassified: i64,
}

/// One bucket of the chosen breakdown dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationBucketRow {
    pub label: String,
    pub user_id: Option<UserId>,
    pub conversations: i64,
    pub users: i64,
    pub turns: i64,
    pub total_cost_microdollars: i64,
    pub achieved: i64,
    pub top_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFacet {
    pub skill: String,
    pub conversations: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFacet {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub conversations: i64,
}

#[derive(Debug, Default)]
pub struct ConversationAnalysisResult {
    pub rows: Vec<ConversationAnalysisRow>,
    pub totals: ConversationAnalysisTotals,
    pub breakdown: Vec<ConversationBucketRow>,
    pub skills: Vec<SkillFacet>,
    pub users: Vec<UserFacet>,
}

// Why: IDs are decoded by SQLx from a companion array; the JSON payload keeps
// them as text so no unchecked constructor is needed on the way out.
#[derive(Debug, Deserialize)]
struct ConversationAnalysisWire {
    rows: Vec<ConversationAnalysisRow<String>>,
    totals: ConversationAnalysisTotals,
    breakdown: Vec<ConversationBucketRow>,
    skills: Vec<SkillFacet>,
    users: Vec<UserFacet>,
}

pub async fn load_conversation_analysis_page(
    pool: &PgPool,
    filter: &ConversationAnalysisFilter,
    page: ConversationAnalysisPage,
) -> Result<ConversationAnalysisResult, sqlx::Error> {
    let started = std::time::Instant::now();
    let mut connection = crate::repositories::dashboard_read::begin(pool).await?;
    let pattern = filter.free_text_pattern();
    let result = sqlx::query_file!(
        "src/repositories/analysis/conversations/page.sql",
        filter.subject_ids.as_deref(),
        filter.user_id.as_ref().map(UserId::as_str),
        filter.category,
        filter.outcome,
        filter.skill,
        pattern,
        filter.since,
        filter.until,
        page.sort.as_str(),
        page.descending,
        page.limit,
        page.offset,
        page.breakdown.as_str(),
    )
    .fetch_one(&mut *connection)
    .await?;
    connection.commit().await?;
    tracing::debug!(
        query_ms = started.elapsed().as_secs_f64() * 1000.0,
        "conversation analysis page loaded"
    );
    let ids = result.context_ids;
    let wire = result.payload.0;
    Ok(ConversationAnalysisResult {
        rows: wire
            .rows
            .into_iter()
            .map(|r| decode_row(r, &ids))
            .collect::<Result<_, _>>()?,
        totals: wire.totals,
        breakdown: wire.breakdown,
        skills: wire.skills,
        users: wire.users,
    })
}

fn decode_row(
    r: ConversationAnalysisRow<String>,
    ids: &[ContextId],
) -> Result<ConversationAnalysisRow, sqlx::Error> {
    Ok(ConversationAnalysisRow {
        context_id: crate::repositories::dashboard_read::context_id(&r.context_id, ids)?,
        title: r.title,
        user_id: r.user_id,
        display_name: r.display_name,
        session_id: r.session_id,
        client_session_id: r.client_session_id,
        group_name: r.group_name,
        project_name: r.project_name,
        model: r.model,
        client_kind: r.client_kind,
        turn_count: r.turn_count,
        tool_call_count: r.tool_call_count,
        error_count: r.error_count,
        total_tokens: r.total_tokens,
        total_cost_microdollars: r.total_cost_microdollars,
        first_at: r.first_at,
        last_at: r.last_at,
        category: r.category,
        summary: r.summary,
        tags: r.tags,
        skills_used: r.skills_used,
        skills_hooked: r.skills_hooked,
        outcome: r.outcome,
        confidence: r.confidence,
        classified_at: r.classified_at,
        judge_model: r.judge_model,
    })
}
