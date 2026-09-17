//! Judge scores keyed on marketplace version (`evaluation_version_scores`,
//! schema 43): what each version of a marketplace scored over the sessions
//! that invoked its skills, the current version against every earlier one,
//! and the day-by-day means the Evaluation page draws.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::MarketplaceId;

/// One version of one marketplace with its mean scores.
#[derive(Debug, Clone, Serialize)]
pub struct VersionScoreRow {
    pub marketplace_id: MarketplaceId,
    pub content_hash: String,
    pub is_current: bool,
    pub first_seen_at: DateTime<Utc>,
    pub effective_until: Option<DateTime<Utc>>,
    pub sessions: i64,
    pub scored_sessions: i64,
    pub intent: Option<f64>,
    pub quality: Option<f64>,
    pub speed: Option<f64>,
    pub cost: Option<f64>,
}

pub async fn list_version_scores(
    pool: &PgPool,
    marketplace_id: &MarketplaceId,
) -> Result<Vec<VersionScoreRow>, sqlx::Error> {
    sqlx::query_as!(
        VersionScoreRow,
        r#"SELECT v.marketplace_id AS "marketplace_id: MarketplaceId", v.content_hash,
                  (v.effective_until IS NULL) AS "is_current!",
                  v.first_seen_at, v.effective_until,
                  COALESCE(e.sessions, 0)::bigint AS "sessions!",
                  COALESCE(e.scored_sessions, 0)::bigint AS "scored_sessions!",
                  e.intent, e.quality, e.speed, e.cost
             FROM marketplace_versions v
             LEFT JOIN evaluation_version_scores e
               ON e.marketplace_id = v.marketplace_id AND e.marketplace_hash = v.content_hash
            WHERE v.marketplace_id = $1
            ORDER BY (v.effective_until IS NULL) DESC, v.first_seen_at DESC"#,
        marketplace_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

/// Mean scores over a set of sessions.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct CriterionMeans {
    pub sessions: i64,
    pub scored_sessions: i64,
    pub intent: Option<f64>,
    pub quality: Option<f64>,
    pub speed: Option<f64>,
    pub cost: Option<f64>,
}

/// One marketplace: its current version's scores beside every earlier
/// version's, so the Evaluation page can say whether the latest is better.
#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceScoreRollup {
    pub marketplace_id: MarketplaceId,
    pub current_hash: Option<String>,
    pub versions: i64,
    pub current: CriterionMeans,
    pub history: CriterionMeans,
}

pub async fn list_marketplace_score_rollups(
    pool: &PgPool,
) -> Result<Vec<MarketplaceScoreRollup>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT v.marketplace_id, v.content_hash, (v.effective_until IS NULL) AS "is_current!",
                  COALESCE(e.sessions, 0)::bigint AS "sessions!",
                  COALESCE(e.scored_sessions, 0)::bigint AS "scored_sessions!",
                  e.intent, e.quality, e.speed, e.cost
             FROM marketplace_versions v
             LEFT JOIN evaluation_version_scores e
               ON e.marketplace_id = v.marketplace_id AND e.marketplace_hash = v.content_hash
            ORDER BY v.marketplace_id, (v.effective_until IS NULL) DESC, v.first_seen_at DESC"#
    )
    .fetch_all(pool)
    .await?;
    let mut out: Vec<MarketplaceScoreRollup> = Vec::new();
    for row in rows {
        let means = CriterionMeans {
            sessions: row.sessions,
            scored_sessions: row.scored_sessions,
            intent: row.intent,
            quality: row.quality,
            speed: row.speed,
            cost: row.cost,
        };
        let rollup = match out.last_mut() {
            Some(r) if r.marketplace_id.as_str() == row.marketplace_id => r,
            _ => {
                out.push(MarketplaceScoreRollup {
                    marketplace_id: MarketplaceId::new(&row.marketplace_id),
                    current_hash: None,
                    versions: 0,
                    current: CriterionMeans::default(),
                    history: CriterionMeans::default(),
                });
                out.last_mut().expect("just pushed")
            },
        };
        rollup.versions += 1;
        if row.is_current {
            rollup.current_hash = Some(row.content_hash);
            rollup.current = means;
        } else {
            rollup.history = merge_means(rollup.history, means);
        }
    }
    Ok(out)
}

// Why: history spans several versions, so its mean is weighted by the scored
// sessions each version contributed rather than averaged over versions.
fn merge_means(acc: CriterionMeans, next: CriterionMeans) -> CriterionMeans {
    let weighted = |a: Option<f64>, b: Option<f64>| match (a, b) {
        (Some(a), Some(b)) if acc.scored_sessions + next.scored_sessions > 0 => {
            let total = (acc.scored_sessions + next.scored_sessions) as f64;
            Some((a * acc.scored_sessions as f64 + b * next.scored_sessions as f64) / total)
        },
        (Some(a), None) => Some(a),
        (None, b) => b,
        (a, _) => a,
    };
    CriterionMeans {
        sessions: acc.sessions + next.sessions,
        scored_sessions: acc.scored_sessions + next.scored_sessions,
        intent: weighted(acc.intent, next.intent),
        quality: weighted(acc.quality, next.quality),
        speed: weighted(acc.speed, next.speed),
        cost: weighted(acc.cost, next.cost),
    }
}

/// Mean scores for one day, for the trend chart.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct ScoreTrendPoint {
    pub day: DateTime<Utc>,
    pub scored_sessions: i64,
    pub intent: Option<f64>,
    pub quality: Option<f64>,
    pub speed: Option<f64>,
    pub cost: Option<f64>,
}

pub async fn list_score_trend(
    pool: &PgPool,
    marketplace_id: Option<&MarketplaceId>,
    days: i32,
) -> Result<Vec<ScoreTrendPoint>, sqlx::Error> {
    sqlx::query_as!(
        ScoreTrendPoint,
        r#"SELECT date_trunc('day', c.last_scored_at) AS "day!",
                  COUNT(*)::bigint AS "scored_sessions!",
                  AVG(c.intent)::float8 AS intent, AVG(c.quality)::float8 AS quality,
                  AVG(c.speed)::float8 AS speed, AVG(c.cost)::float8 AS cost
             FROM evaluation_session_scorecard c
            WHERE c.scored_criteria > 0
              AND c.last_scored_at > clock_timestamp() - make_interval(days => $2)
              AND ($1::text IS NULL OR EXISTS (
                    SELECT 1 FROM evaluation_session_skills s
                     WHERE s.session_id = c.session_id AND s.marketplace_id = $1))
            GROUP BY 1 ORDER BY 1"#,
        marketplace_id.map(MarketplaceId::as_str),
        days,
    )
    .fetch_all(pool)
    .await
}
