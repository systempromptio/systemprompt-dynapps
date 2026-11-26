//! What each viewer decided about a live page, per scope (`mcp:<server>`):
//! granted or denied, and when. No row is "not asked yet".

use sqlx::PgPool;
use systemprompt::identifiers::{DocumentId, UserId};

#[derive(Debug)]
pub struct ArtifactConsent {
    pub scope: String,
    pub granted: bool,
}

pub async fn set_artifact_consent(
    pool: &PgPool,
    user: &UserId,
    document: &DocumentId,
    consent: &ArtifactConsent,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO artifact_consents (user_id, document_id, scope, granted)
           VALUES ($1, $2::text::uuid, $3, $4)
           ON CONFLICT (user_id, document_id, scope) DO UPDATE
              SET granted = EXCLUDED.granted,
                  decided_at = CURRENT_TIMESTAMP"#,
        user.as_str(),
        document.as_str(),
        consent.scope,
        consent.granted,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_artifact_consent(
    pool: &PgPool,
    user: &UserId,
    document: &DocumentId,
    scope: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"DELETE FROM artifact_consents
            WHERE user_id = $1 AND document_id = $2::text::uuid AND scope = $3"#,
        user.as_str(),
        document.as_str(),
        scope,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_artifact_consents(
    pool: &PgPool,
    user: &UserId,
    document: &DocumentId,
) -> Result<Vec<ArtifactConsent>, sqlx::Error> {
    sqlx::query_as!(
        ArtifactConsent,
        r#"SELECT scope, granted
             FROM artifact_consents
            WHERE user_id = $1 AND document_id = $2::text::uuid
            ORDER BY scope"#,
        user.as_str(),
        document.as_str(),
    )
    .fetch_all(pool)
    .await
}
