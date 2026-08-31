//! Organization membership rows: who belongs to an organization, in what
//! role, and the platform organization's own roster.
//!
//! Split from `crud.rs`, which keeps the organization lookups themselves. The
//! seam is the table each query is anchored on — `organization_members` here,
//! `organizations` there.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_shared::error::MarketplaceError;

use super::crud::OrganizationMember;
use crate::authz::organization;

pub async fn find_member_role(
    pool: &PgPool,
    user_id: &UserId,
    org_id: &str,
) -> Result<Option<String>, MarketplaceError> {
    let role = sqlx::query_scalar!(
        "SELECT org_role FROM organization_members WHERE user_id = $1 AND org_id = $2",
        user_id.as_str(),
        org_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(role)
}

pub async fn find_membership_org(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<String>, MarketplaceError> {
    let org_id = sqlx::query_scalar!(
        "SELECT org_id FROM organization_members WHERE user_id = $1",
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?;
    Ok(org_id)
}

// Why: `ON CONFLICT DO NOTHING` on the user_id PK keeps this idempotent and
// non-destructive — a user already in an organization (this one or another)
// is left exactly where they are, and the return value says whether a row
// landed so callers can distinguish "granted" from "already settled".
pub async fn insert_membership_if_absent(
    pool: &PgPool,
    user_id: &UserId,
    org_id: &str,
    org_role: &str,
) -> Result<bool, MarketplaceError> {
    let result = sqlx::query!(
        "INSERT INTO organization_members (user_id, org_id, org_role)
         VALUES ($1, $2, $3)
         ON CONFLICT (user_id) DO NOTHING",
        user_id.as_str(),
        org_id,
        org_role,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_platform_membership(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<bool, MarketplaceError> {
    let result = sqlx::query!(
        "DELETE FROM organization_members m
         USING organizations o
         WHERE o.id = m.org_id AND o.is_platform AND m.user_id = $1",
        user_id.as_str()
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn count_platform_members(pool: &PgPool) -> Result<i64, MarketplaceError> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM organization_members m
           JOIN organizations o ON o.id = m.org_id
           WHERE o.is_platform"#
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

// Why: The second half of the super-admin test — the first is the `admin` role.
// A customer's own administrator holds that role too, so the role alone says
// "may administer an organization", and this says "may administer *every*
// organization".
//
// lint-ok: unused-pub — no production caller while this instance is
// single-tenant: `is_platform_admin` is derived from the `admin` role alone
// (see `middleware/mod.rs` and `handlers/bridge_whoami.rs`). Kept because it is
// the seam that restores pooled multi-tenancy — re-tightening those two
// derivations to `is_admin && get_platform_membership(..)` is the whole change.
pub async fn get_platform_membership(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<bool, MarketplaceError> {
    let found = sqlx::query_scalar!(
        "SELECT 1 AS present FROM organization_members m
         JOIN organizations o ON o.id = m.org_id
         WHERE m.user_id = $1 AND o.is_platform",
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?;
    Ok(found.is_some())
}

pub async fn list_members(
    pool: &PgPool,
    org_id: &str,
) -> Result<Vec<OrganizationMember>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            u.id AS "user_id!: UserId",
            u.email AS "email!",
            u.display_name,
            m.org_role AS "org_role!",
            NULLIF(e.department, '') AS "department?",
            (u.status = 'active') AS "is_active!"
        FROM organization_members m
        JOIN users u ON u.id = m.user_id
        LEFT JOIN user_profile_ext e ON e.user_id = u.id
        WHERE m.org_id = $1
        ORDER BY u.email
        "#,
        org_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| OrganizationMember {
            user_id: r.user_id,
            email: r.email,
            display_name: r.display_name,
            org_role: r.org_role,
            department: r.department,
            is_active: r.is_active,
        })
        .collect())
}

// Why: Callers that are creating a *new* seat must call
// [`super::seats::assert_seat_available`] first — this function does not
// check the limit, because moving an existing member between departments of
// the same org, or repairing a membership row, must not be blocked by a full
// plan.
pub async fn set_membership(
    pool: &PgPool,
    user_id: &UserId,
    org_id: &str,
    org_role: &str,
) -> Result<(), MarketplaceError> {
    sqlx::query!(
        "INSERT INTO organization_members (user_id, org_id, org_role)
         VALUES ($1, $2, $3)
         ON CONFLICT (user_id) DO UPDATE
            SET org_id = EXCLUDED.org_id,
                org_role = EXCLUDED.org_role",
        user_id.as_str(),
        org_id,
        org_role,
    )
    .execute(pool)
    .await?;

    organization::invalidate(user_id).await;
    Ok(())
}
