//! What the report generator reads for one scope and window.
//!
//! The judge's aggregate figures, every skill's means, conversation
//! categories with their token load, and the worst and best judged sessions
//! with the judge's own words. Each read is scoped the same way — the
//! instance, or the sessions that invoked one of a marketplace's skills.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, PluginId, SessionId};

use super::reports::ReportScope;

#[derive(Debug, Clone, Copy)]
pub struct ReportWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Runs and means for the window, with the equal window before it.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ReportScopeTotals {
    pub runs: i64,
    pub failed_runs: i64,
    pub scored_sessions: i64,
    pub intent: Option<f64>,
    pub quality: Option<f64>,
    pub speed: Option<f64>,
    pub cost: Option<f64>,
    pub prior_scored_sessions: i64,
    pub prior_intent: Option<f64>,
    pub prior_quality: Option<f64>,
    pub prior_speed: Option<f64>,
    pub prior_cost: Option<f64>,
    pub last_run_at: Option<DateTime<Utc>>,
}

pub async fn get_report_scope_totals(
    pool: &PgPool,
    scope: &ReportScope,
    window: ReportWindow,
) -> Result<ReportScopeTotals, sqlx::Error> {
    let prior_start = window.start - (window.end - window.start);
    let marketplace = match scope {
        ReportScope::Global => None,
        ReportScope::Marketplace(id) => Some(id.as_str()),
    };
    sqlx::query_as!(
        ReportScopeTotals,
        r#"WITH runs AS (
               SELECT r.session_id, r.status, r.created_at
                 FROM evaluation_runs r
                WHERE r.created_at >= $2 AND r.created_at < $3
                  AND ($1::text IS NULL OR EXISTS (
                        SELECT 1 FROM evaluation_session_skills k
                         WHERE k.session_id = r.session_id AND k.marketplace_id = $1))
           ), cards AS (
               SELECT c.*
                 FROM evaluation_session_scorecard c
                WHERE c.scored_criteria > 0 AND c.last_scored_at >= $4 AND c.last_scored_at < $3
                  AND ($1::text IS NULL OR EXISTS (
                        SELECT 1 FROM evaluation_session_skills k
                         WHERE k.session_id = c.session_id AND k.marketplace_id = $1))
           )
           SELECT (SELECT COUNT(*) FROM runs)::bigint AS "runs!",
                  (SELECT COUNT(*) FROM runs WHERE status = 'failed')::bigint AS "failed_runs!",
                  (SELECT MAX(created_at) FROM runs WHERE status = 'succeeded') AS last_run_at,
                  COUNT(*) FILTER (WHERE last_scored_at >= $2)::bigint AS "scored_sessions!",
                  AVG(intent) FILTER (WHERE last_scored_at >= $2)::float8 AS intent,
                  AVG(quality) FILTER (WHERE last_scored_at >= $2)::float8 AS quality,
                  AVG(speed) FILTER (WHERE last_scored_at >= $2)::float8 AS speed,
                  AVG(cost) FILTER (WHERE last_scored_at >= $2)::float8 AS cost,
                  COUNT(*) FILTER (WHERE last_scored_at < $2)::bigint AS "prior_scored_sessions!",
                  AVG(intent) FILTER (WHERE last_scored_at < $2)::float8 AS prior_intent,
                  AVG(quality) FILTER (WHERE last_scored_at < $2)::float8 AS prior_quality,
                  AVG(speed) FILTER (WHERE last_scored_at < $2)::float8 AS prior_speed,
                  AVG(cost) FILTER (WHERE last_scored_at < $2)::float8 AS prior_cost
             FROM cards"#,
        marketplace,
        window.start,
        window.end,
        prior_start,
    )
    .fetch_one(pool)
    .await
}

/// One skill's means in the scope, ordered worst quality first.
#[derive(Debug, Clone, Serialize)]
pub struct ReportSkillFigure {
    pub plugin_id: PluginId,
    pub skill: String,
    pub sessions: i64,
    pub scored_sessions: i64,
    pub invocations: i64,
    pub intent: Option<f64>,
    pub quality: Option<f64>,
    pub speed: Option<f64>,
    pub cost: Option<f64>,
    pub quality_prior: Option<f64>,
}

pub async fn list_report_skill_figures(
    pool: &PgPool,
    scope: &ReportScope,
    window: ReportWindow,
    limit: i64,
) -> Result<Vec<ReportSkillFigure>, sqlx::Error> {
    let prior_start = window.start - (window.end - window.start);
    let marketplace = match scope {
        ReportScope::Global => None,
        ReportScope::Marketplace(id) => Some(id.as_str()),
    };
    sqlx::query_as!(
        ReportSkillFigure,
        r#"SELECT s.plugin_id AS "plugin_id!: PluginId", s.skill AS "skill!",
                  COUNT(DISTINCT s.session_id) FILTER (WHERE s.first_invoked_at >= $2)::bigint AS "sessions!",
                  COUNT(DISTINCT s.session_id) FILTER (WHERE s.first_invoked_at >= $2 AND c.scored_criteria > 0)::bigint AS "scored_sessions!",
                  COALESCE(SUM(s.invocations) FILTER (WHERE s.first_invoked_at >= $2), 0)::bigint AS "invocations!",
                  AVG(c.intent) FILTER (WHERE s.first_invoked_at >= $2)::float8 AS intent,
                  AVG(c.quality) FILTER (WHERE s.first_invoked_at >= $2)::float8 AS quality,
                  AVG(c.speed) FILTER (WHERE s.first_invoked_at >= $2)::float8 AS speed,
                  AVG(c.cost) FILTER (WHERE s.first_invoked_at >= $2)::float8 AS cost,
                  AVG(c.quality) FILTER (WHERE s.first_invoked_at < $2)::float8 AS quality_prior
             FROM evaluation_session_skills s
             LEFT JOIN evaluation_session_scorecard c ON c.session_id = s.session_id
            WHERE s.first_invoked_at >= $4 AND s.first_invoked_at < $3
              AND ($1::text IS NULL OR s.marketplace_id = $1)
            GROUP BY s.plugin_id, s.skill
           HAVING COUNT(DISTINCT s.session_id) FILTER (WHERE s.first_invoked_at >= $2) > 0
            ORDER BY AVG(c.quality) FILTER (WHERE s.first_invoked_at >= $2) ASC NULLS LAST,
                     "sessions!" DESC
            LIMIT $5"#,
        marketplace,
        window.start,
        window.end,
        prior_start,
        limit,
    )
    .fetch_all(pool)
    .await
}

/// One conversation category with its means and its token and cost load.
#[derive(Debug, Clone, Serialize)]
pub struct ReportCategoryFigure {
    pub category: String,
    pub conversations: i64,
    pub achieved: i64,
    pub scored: i64,
    pub intent: Option<f64>,
    pub quality: Option<f64>,
    pub speed: Option<f64>,
    pub cost: Option<f64>,
    pub tokens_avg: Option<f64>,
    pub cost_avg_microdollars: Option<f64>,
}

pub async fn list_report_category_figures(
    pool: &PgPool,
    scope: &ReportScope,
    window: ReportWindow,
) -> Result<Vec<ReportCategoryFigure>, sqlx::Error> {
    let marketplace = match scope {
        ReportScope::Global => None,
        ReportScope::Marketplace(id) => Some(id.as_str()),
    };
    sqlx::query_file_as!(
        ReportCategoryFigure,
        "src/repositories/evaluation/report_categories.sql",
        marketplace,
        window.start,
        window.end,
    )
    .fetch_all(pool)
    .await
}

/// One of the worst or best judged sessions, with the judge's words.
#[derive(Debug, Clone, Serialize)]
pub struct ReportOutlierSession {
    pub session_id: SessionId,
    pub is_worst: bool,
    pub intent: Option<i16>,
    pub quality: Option<i16>,
    pub speed: Option<i16>,
    pub cost: Option<i16>,
    pub context_id: Option<ContextId>,
    pub title: Option<String>,
    pub category: Option<String>,
    pub skills: Vec<String>,
    pub quality_rationale: Option<String>,
    pub evidence_quote: Option<String>,
    pub intent_rationale: Option<String>,
}

pub async fn list_report_outlier_sessions(
    pool: &PgPool,
    scope: &ReportScope,
    window: ReportWindow,
    per_side: i64,
) -> Result<Vec<ReportOutlierSession>, sqlx::Error> {
    let marketplace = match scope {
        ReportScope::Global => None,
        ReportScope::Marketplace(id) => Some(id.as_str()),
    };
    sqlx::query_file_as!(
        ReportOutlierSession,
        "src/repositories/evaluation/report_outliers.sql",
        marketplace,
        window.start,
        window.end,
        per_side,
    )
    .fetch_all(pool)
    .await
}
