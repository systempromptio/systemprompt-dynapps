//! The write half of federated resolution: just-in-time provisioning and the
//! per-login role projection. Split from [`super::federated`] so the read
//! ladder there stays readable; nothing here is called from outside it.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_shared::error::MarketplaceError;

use super::federated::{FederatedClaims, ResolvedFederatedUser};
use crate::repositories::organizations;

// Why: `name` is set to the email to sidestep the `users.name` uniqueness
// constraint; `display_name` carries the human-friendly form.
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
    // Why: the seat check runs before the user exists, so a full plan rejects
    // the login rather than creating an orphaned account that cannot reach
    // anything. An unclaimed email domain is not an error — that arrival joins
    // the default organization, and consumes a seat there like any other.
    let org_id = organizations::crud::find_organization_for_new_user(pool, email).await?;
    if let Some(org_id) = org_id.as_deref() {
        organizations::seats::assert_seat_available(pool, org_id).await?;
    }

    let user_id = uuid::Uuid::new_v4().to_string();
    // Why: an IdP that maps no roles still provisions a plain user; an empty
    // role list would be an account that can reach nothing.
    let roles = if roles.is_empty() {
        vec!["user".to_owned()]
    } else {
        roles.to_vec()
    };
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

    if let Some(org_id) = org_id.as_deref() {
        sqlx::query!(
            "INSERT INTO organization_members (user_id, org_id, org_role)
             VALUES ($1, $2, 'member')",
            &user_id,
            org_id,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(ResolvedFederatedUser {
        user_id: UserId::new(user_id),
        email: email.to_owned(),
        display_name: display_name.to_owned(),
        roles,
        lost_admin: false,
    })
}

pub(super) struct ProjectedRoles {
    pub roles: Vec<String>,
    // Why: the JWT bakes permissions in at issue time, so a role the directory
    // has taken away still rides in every token minted before this login. The
    // caller uses this to tear those down.
    pub lost_admin: bool,
}

// Why: The IdP is the source of truth for roles, so a returning login
// overwrites whatever the row held. Skipped when the claim carries nothing —
// an operator-created account signing in through an IdP that maps no roles
// (evaluation instances) keeps its CLI-granted roles rather than losing them.
pub(super) async fn project_roles(
    pool: &PgPool,
    user_id: &UserId,
    current: Vec<String>,
    mapped: &[String],
) -> Result<ProjectedRoles, sqlx::Error> {
    if mapped.is_empty() || current == mapped {
        return Ok(ProjectedRoles {
            roles: current,
            lost_admin: false,
        });
    }
    let lost_admin = has_admin(&current) && !has_admin(mapped);
    sqlx::query!(
        "UPDATE users SET roles = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $1",
        user_id.as_str(),
        mapped,
    )
    .execute(pool)
    .await?;
    Ok(ProjectedRoles {
        roles: mapped.to_vec(),
        lost_admin,
    })
}

fn has_admin(roles: &[String]) -> bool {
    roles.iter().any(|r| r == "admin")
}
