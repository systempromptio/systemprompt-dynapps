//! The judge's name for a session, written back onto its summary row.

use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

pub async fn update_session_title(
    pool: &PgPool,
    session_id: &SessionId,
    title: &str,
    summary: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"UPDATE plugin_session_summaries
          SET ai_title = $2, ai_summary = $3, updated_at = NOW()
          WHERE session_id = $1",
        session_id.as_str(),
        title,
        summary,
    )
    .execute(pool)
    .await?;
    Ok(())
}
