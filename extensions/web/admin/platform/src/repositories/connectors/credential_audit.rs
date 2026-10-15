//! The credential audit trail: who attached, rotated or removed a service
//! account's machine credential. Append-only.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, Copy)]
pub struct CredentialAudit<'a> {
    pub subject: &'a UserId,
    pub provider: &'a str,
    pub action: &'a str,
    pub actor: &'a UserId,
}

pub async fn insert_credential_audit(
    pool: &PgPool,
    audit: CredentialAudit<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO connector_credential_audit (subject_user_id, provider, action, \
         actor_user_id) VALUES ($1, $2, $3, $4)",
        audit.subject.as_str(),
        audit.provider,
        audit.action,
        audit.actor.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(())
}
