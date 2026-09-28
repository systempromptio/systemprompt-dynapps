//! The write half of federated resolution: just-in-time provisioning and the
//! per-login role projection. Split from [`super::federated`] so the read
//! ladder there stays readable; nothing here is called from outside it.
//!
//! A returning login rewrites only the directory half of the role set; manual
//! dashboard grants survive it. A claim that maps no roles projects nothing, so
//! an operator-created account keeps its CLI-granted roles. Losing any role
//! that mints `Permission::Admin` is reported as `lost_admin`, because tokens
//! issued before this login still carry it.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::constants::ROLE_USER;

use super::federated::{FederatedClaims, ResolvedFederatedUser};

// Why: Postgres `users.name` is UNIQUE, so `name` is set to the email;
// `display_name` carries the human-friendly form.
pub(super) async fn create_federated(
    pool: &PgPool,
    claims: &FederatedClaims<'_>,
) -> Result<ResolvedFederatedUser, MarketplaceError> {
    let FederatedClaims {
        issuer,
        external_sub,
        email,
        display_name,
        roles,
    } = *claims;

    let user_id = uuid::Uuid::new_v4().to_string();
    let roles = roles_or_plain_user(roles);
    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        INSERT INTO users (id, name, email, display_name, status, email_verified, roles)
        VALUES ($1, $2, $3, $4, 'active', true, $5)
        "#,
        &user_id,
        email,
        email,
        display_name,
        &roles,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO federated_identities (issuer, external_sub, user_id) VALUES ($1, $2, $3)",
        issuer,
        external_sub,
        &user_id,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO user_profile_ext (user_id) VALUES ($1) ON CONFLICT DO NOTHING",
        &user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(ResolvedFederatedUser {
        user_id: UserId::new(user_id),
        email: email.to_owned(),
        display_name: display_name.to_owned(),
        roles,
        lost_admin: false,
    })
}

fn roles_or_plain_user(mapped: &[String]) -> Vec<String> {
    if mapped.is_empty() {
        vec![ROLE_USER.to_owned()]
    } else {
        mapped.to_vec()
    }
}

pub(super) struct ProjectedRoles {
    pub roles: Vec<String>,
    pub lost_admin: bool,
}

pub(super) async fn project_roles(
    pool: &PgPool,
    user_id: &UserId,
    current: Vec<String>,
    mapped: &[String],
) -> Result<ProjectedRoles, sqlx::Error> {
    if mapped.is_empty() {
        return Ok(ProjectedRoles {
            roles: current,
            lost_admin: false,
        });
    }
    let roles = super::roles::recompute_roles(pool, user_id, Some(mapped)).await?;
    Ok(ProjectedRoles {
        lost_admin: mints_admin_permission(&current) && !mints_admin_permission(&roles),
        roles,
    })
}

fn mints_admin_permission(roles: &[String]) -> bool {
    crate::types::role::has_any(roles, crate::types::role::ROLES_MANAGE)
}
