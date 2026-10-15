//! Whether a `farm:<run id>` purpose names a run its bearer is running now.
//!
//! Read from core's `agent_runs` (origin `agent_farm_run`, origin id = the
//! farm run id, user = the account the run's credentials were issued to).

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub async fn get_farm_run_active(
    pool: &PgPool,
    farm_run_id: &str,
    user: &UserId,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT EXISTS (SELECT 1 FROM agent_runs WHERE origin_kind = 'agent_farm_run' \
         AND origin_id = $1 AND user_id = $2 \
         AND status IN ('queued', 'preparing', 'running')) AS \"active!\"",
        farm_run_id,
        user.as_str(),
    )
    .fetch_one(pool)
    .await
}
