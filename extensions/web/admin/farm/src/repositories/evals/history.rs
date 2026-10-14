//! Eval history: finished eval runs folded per agent per day, the series the
//! history API, the MCP results tool and the dashboard trend table read.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::repositories::rows::EvalDayRow;
use crate::types::{FarmAgentId, FarmId};

pub async fn list_eval_history(
    pool: &PgPool,
    farm_id: Option<&FarmId>,
    agent_id: Option<&FarmAgentId>,
    since: DateTime<Utc>,
) -> Result<Vec<EvalDayRow>, sqlx::Error> {
    sqlx::query_file_as!(
        EvalDayRow,
        "src/repositories/evals/list_eval_history.sql",
        farm_id.map(FarmId::as_str),
        agent_id.map(FarmAgentId::as_str),
        since,
    )
    .fetch_all(pool)
    .await
}
