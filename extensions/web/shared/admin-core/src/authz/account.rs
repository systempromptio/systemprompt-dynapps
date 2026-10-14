//! Resolves access scope from the account's current status and stored roles.
//!
//! `account_scope_and_roles` also returns the roles, for governance policies
//! that gate by a role; an inactive account holds none.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::policy::types::AccessScope;

pub async fn account_scope(pool: &PgPool, user_id: &UserId) -> Result<AccessScope, sqlx::Error> {
    Ok(account_scope_and_roles(pool, user_id).await?.0)
}

pub async fn account_scope_and_roles(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<(AccessScope, Vec<String>), sqlx::Error> {
    let identity = crate::repositories::principal::identity::find_identity_envelope(pool, user_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    if identity.status != "active" {
        return Ok((AccessScope::Unknown, Vec::new()));
    }
    let scope = if crate::types::roles_grant_manage(&identity.roles) {
        AccessScope::Admin
    } else {
        AccessScope::User
    };
    Ok((scope, identity.roles))
}
