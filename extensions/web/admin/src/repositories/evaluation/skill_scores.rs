//! Judge scores attributed to skills.
//!
//! A skill inherits the latest scores of every session that invoked it
//! (`evaluation_session_skills` × `evaluation_session_scorecard`, schema 43),
//! averaged over the window and over the equal window before it so a row can
//! show which way it is moving.

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{PluginId, SessionId};

use crate::repositories::analysis::retained::RetainedWindow;

/// Mean criterion scores for one skill over the sessions that invoked it.
#[derive(Debug, Clone, Serialize)]
pub struct SkillScoreFigures {
    pub plugin_id: PluginId,
    pub skill: String,
    pub sessions: i64,
    pub scored_sessions: i64,
    pub intent: Option<f64>,
    pub quality: Option<f64>,
    pub speed: Option<f64>,
    pub cost: Option<f64>,
    // Why: the same mean over the window immediately before this one, so the
    // page can say "up 6" instead of asking the reader to remember.
    pub quality_prior: Option<f64>,
}

pub async fn list_skill_score_figures(
    pool: &PgPool,
    window: RetainedWindow,
) -> Result<Vec<SkillScoreFigures>, sqlx::Error> {
    let prior_start = window.start - (window.end - window.start);
    sqlx::query_as!(
        SkillScoreFigures,
        r#"SELECT s.plugin_id AS "plugin_id!: PluginId", s.skill AS "skill!",
                  COUNT(DISTINCT s.session_id) FILTER (WHERE s.first_invoked_at >= $1)::bigint AS "sessions!",
                  COUNT(DISTINCT s.session_id) FILTER (WHERE s.first_invoked_at >= $1 AND c.scored_criteria > 0)::bigint AS "scored_sessions!",
                  AVG(c.intent) FILTER (WHERE s.first_invoked_at >= $1)::float8 AS intent,
                  AVG(c.quality) FILTER (WHERE s.first_invoked_at >= $1)::float8 AS quality,
                  AVG(c.speed) FILTER (WHERE s.first_invoked_at >= $1)::float8 AS speed,
                  AVG(c.cost) FILTER (WHERE s.first_invoked_at >= $1)::float8 AS cost,
                  AVG(c.quality) FILTER (WHERE s.first_invoked_at < $1)::float8 AS quality_prior
             FROM evaluation_session_skills s
             LEFT JOIN evaluation_session_scorecard c ON c.session_id = s.session_id
            WHERE s.first_invoked_at >= $3 AND s.first_invoked_at < $2
            GROUP BY s.plugin_id, s.skill"#,
        window.start,
        window.end,
        prior_start,
    )
    .fetch_all(pool)
    .await
}

/// Scores per day for one skill, for the sparkline on its page.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SkillScoreDay {
    pub day: chrono::DateTime<chrono::Utc>,
    pub scored_sessions: i64,
    pub quality: Option<f64>,
    pub intent: Option<f64>,
}

pub async fn list_skill_score_days(
    pool: &PgPool,
    plugin_id: &PluginId,
    skill: &str,
    window: RetainedWindow,
) -> Result<Vec<SkillScoreDay>, sqlx::Error> {
    sqlx::query_as!(
        SkillScoreDay,
        r#"SELECT date_trunc('day', s.first_invoked_at) AS "day!",
                  COUNT(DISTINCT s.session_id) FILTER (WHERE c.scored_criteria > 0)::bigint AS "scored_sessions!",
                  AVG(c.quality)::float8 AS quality, AVG(c.intent)::float8 AS intent
             FROM evaluation_session_skills s
             LEFT JOIN evaluation_session_scorecard c ON c.session_id = s.session_id
            WHERE s.plugin_id = $1 AND s.skill = $2
              AND s.first_invoked_at >= $3 AND s.first_invoked_at < $4
            GROUP BY 1 ORDER BY 1"#,
        plugin_id.as_str(),
        skill,
        window.start,
        window.end,
    )
    .fetch_all(pool)
    .await
}

/// The latest scores of one session, for the rows of a skill's page.
#[derive(Debug, Clone, Serialize)]
pub struct SessionScorecardRow {
    pub session_id: SessionId,
    pub intent: Option<i16>,
    pub quality: Option<i16>,
    pub speed: Option<i16>,
    pub cost: Option<i16>,
    pub scored_criteria: i32,
}

pub async fn list_session_scorecards(
    pool: &PgPool,
    session_ids: &[String],
) -> Result<Vec<SessionScorecardRow>, sqlx::Error> {
    sqlx::query_as!(
        SessionScorecardRow,
        r#"SELECT c.session_id AS "session_id!: SessionId", c.intent, c.quality, c.speed, c.cost,
                  c.scored_criteria AS "scored_criteria!"
             FROM evaluation_session_scorecard c
            WHERE c.session_id = ANY($1)"#,
        session_ids,
    )
    .fetch_all(pool)
    .await
}

/// Mean scores over every judged session that invoked any skill in the
/// window — counted once per session, however many skills it used.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct WindowScoreMeans {
    pub sessions: i64,
    pub scored_sessions: i64,
    pub intent: Option<f64>,
    pub quality: Option<f64>,
    pub speed: Option<f64>,
    pub cost: Option<f64>,
    pub quality_prior: Option<f64>,
}

pub async fn get_window_score_means(
    pool: &PgPool,
    window: RetainedWindow,
) -> Result<WindowScoreMeans, sqlx::Error> {
    let prior_start = window.start - (window.end - window.start);
    sqlx::query_as!(
        WindowScoreMeans,
        r#"WITH s AS (
               SELECT session_id, MIN(first_invoked_at) AS first_invoked_at
                 FROM evaluation_session_skills
                WHERE first_invoked_at >= $3 AND first_invoked_at < $2
                GROUP BY session_id)
           SELECT COUNT(*) FILTER (WHERE s.first_invoked_at >= $1)::bigint AS "sessions!",
                  COUNT(*) FILTER (WHERE s.first_invoked_at >= $1 AND c.scored_criteria > 0)::bigint AS "scored_sessions!",
                  AVG(c.intent) FILTER (WHERE s.first_invoked_at >= $1)::float8 AS intent,
                  AVG(c.quality) FILTER (WHERE s.first_invoked_at >= $1)::float8 AS quality,
                  AVG(c.speed) FILTER (WHERE s.first_invoked_at >= $1)::float8 AS speed,
                  AVG(c.cost) FILTER (WHERE s.first_invoked_at >= $1)::float8 AS cost,
                  AVG(c.quality) FILTER (WHERE s.first_invoked_at < $1)::float8 AS quality_prior
             FROM s
             LEFT JOIN evaluation_session_scorecard c ON c.session_id = s.session_id"#,
        window.start,
        window.end,
        prior_start,
    )
    .fetch_one(pool)
    .await
}
