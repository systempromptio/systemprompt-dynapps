//! Session ownership at ingestion: a session id is bound to the first
//! principal that reports it.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

// Why: binds the session to the authenticated principal before anything is
// recorded against it, so a hook carrying someone else's session id is
// refused by the database rather than trusted by the handler.
pub(crate) async fn assert_ingestion_owner(
    pool: &PgPool,
    session_id: &SessionId,
    user_id: &UserId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "SELECT assert_ingestion_owner($1, $2)",
        session_id.as_str(),
        user_id.as_str()
    )
    .fetch_one(pool)
    .await?;
    Ok(())
}
