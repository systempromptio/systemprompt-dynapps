//! Eval history as every surface reports it.
//!
//! Per agent per day: the number of eval runs, the case pass rate, the mean
//! judge score and the spend, over the farms the caller may use. The public
//! API, the MCP results tool and the dashboard read the same points.

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt_web_admin_core::types::UserContext;

use crate::error::FarmResult;
use crate::repositories::rows::EvalDayRow;
use crate::repositories::{evals, farms};
use crate::services::entitlement::may_use_farm;
use crate::types::{FarmAgentId, FarmId};

pub const DEFAULT_HISTORY_DAYS: i64 = 30;
const MAX_HISTORY_DAYS: i64 = 366;
const MICRODOLLARS_PER_DOLLAR: f64 = 1_000_000.0;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvalHistoryPoint {
    pub farm: FarmId,
    pub agent: FarmAgentId,
    pub day: NaiveDate,
    pub runs: i64,
    pub passed: i64,
    pub decided: i64,
    pub pass_rate: Option<f64>,
    pub mean_score: Option<f64>,
    pub cost_usd: f64,
}

impl From<EvalDayRow> for EvalHistoryPoint {
    fn from(row: EvalDayRow) -> Self {
        let pass_rate = (row.decided > 0).then(|| row.passed as f64 * 100.0 / row.decided as f64);
        Self {
            farm: row.farm_id,
            agent: row.agent_id,
            day: row.day,
            runs: row.runs,
            passed: row.passed,
            decided: row.decided,
            pass_rate,
            mean_score: row.mean_score,
            cost_usd: row.cost_microdollars as f64 / MICRODOLLARS_PER_DOLLAR,
        }
    }
}

#[must_use]
pub fn parse_since(raw: &str, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let raw = raw.trim();
    let floor = now - Duration::days(MAX_HISTORY_DAYS);
    if raw.is_empty() {
        return Some(now - Duration::days(DEFAULT_HISTORY_DAYS));
    }
    let relative = |suffix: char, unit: fn(i64) -> Duration| {
        raw.strip_suffix(suffix)
            .and_then(|n| n.parse::<i64>().ok())
            .filter(|n| *n > 0)
            .map(|n| now - unit(n))
    };
    let at = relative('d', Duration::days)
        .or_else(|| relative('h', Duration::hours))
        .or_else(|| {
            DateTime::parse_from_rfc3339(raw)
                .ok()
                .map(|t| t.with_timezone(&Utc))
        })?;
    Some(at.max(floor))
}

pub async fn list_visible_history(
    pool: &PgPool,
    ctx: &UserContext,
    farm: Option<&FarmId>,
    agent: Option<&FarmAgentId>,
    since: DateTime<Utc>,
) -> FarmResult<Vec<EvalHistoryPoint>> {
    let mut usable: Vec<FarmId> = Vec::new();
    for row in farms::list_farms(pool).await? {
        if farm.is_none_or(|f| *f == row.id) && may_use_farm(pool, ctx, &row).await? {
            usable.push(row.id);
        }
    }
    if usable.is_empty() {
        return Ok(Vec::new());
    }
    let one = (usable.len() == 1).then(|| &usable[0]);
    let rows = evals::list_eval_history(pool, one, agent, since).await?;
    Ok(rows
        .into_iter()
        .filter(|r| usable.contains(&r.farm_id))
        .map(EvalHistoryPoint::from)
        .collect())
}
