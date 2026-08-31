//! Organization and membership rows.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_shared::error::MarketplaceError;


#[derive(Debug, Clone)]
pub struct OrganizationSummary {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub plan_id: Option<String>,
    pub plan_name: Option<String>,
    pub status: String,
    pub is_platform: bool,
    pub seat_limit: Option<i32>,
    pub seats_used: i64,
}

#[derive(Debug, Clone)]
pub struct OrganizationMember {
    pub user_id: UserId,
    pub email: String,
    pub display_name: Option<String>,
    pub org_role: String,
    pub department: Option<String>,
    pub is_active: bool,
}

pub async fn list_organizations(
    pool: &PgPool,
) -> Result<Vec<OrganizationSummary>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            o.id AS "id!",
            o.slug AS "slug!",
            o.name AS "name!",
            o.plan_id,
            p.name AS "plan_name?",
            o.status AS "status!",
            o.is_platform AS "is_platform!",
            COALESCE(o.seat_limit_override, p.seat_limit) AS "seat_limit?",
            (SELECT COUNT(*) FROM organization_members m
              JOIN users u ON u.id = m.user_id
             WHERE m.org_id = o.id AND u.status = 'active') AS "seats_used!"
        FROM organizations o
        LEFT JOIN plans p ON p.id = o.plan_id
        ORDER BY o.name
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| OrganizationSummary {
            id: r.id,
            slug: r.slug,
            name: r.name,
            plan_id: r.plan_id,
            plan_name: r.plan_name,
            status: r.status,
            is_platform: r.is_platform,
            seat_limit: r.seat_limit,
            seats_used: r.seats_used,
        })
        .collect())
}

pub async fn find_organization_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<OrganizationSummary>, MarketplaceError> {
    Ok(list_organizations(pool)
        .await?
        .into_iter()
        .find(|o| o.slug == slug))
}

// Why: Suspended organizations are returned: this answers "who owns this user",
// which stays true while a customer is suspended, unlike the authz provider,
// which deliberately reports nothing so grants stop resolving.
pub async fn find_organization_for_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<String>, MarketplaceError> {
    let slug = sqlx::query_scalar!(
        "SELECT o.slug FROM organization_members m
         JOIN organizations o ON o.id = m.org_id
         WHERE m.user_id = $1",
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?;
    Ok(slug)
}

pub async fn find_platform_organization(
    pool: &PgPool,
) -> Result<Option<(String, String)>, MarketplaceError> {
    let row = sqlx::query!("SELECT id, name FROM organizations WHERE is_platform")
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| (r.id, r.name)))
}

// Why: An email domain is how a federated arrival is matched to the customer
// whose seat they consume. A domain no organization claims is not an error —
// that arrival is simply not on a named contract — but it must not leave them
// unattached either: every analytics, reporting, and entitlement query scopes
// through `organization_members`, so a user with no row is invisible to the
// dashboards meant to account for them and entitled to nothing. The default
// organization is that fallback, which is why it lives inside this function
// rather than at its call sites: this is the only thing both provisioning
// doors — operator-created users and SSO just-in-time arrivals — consult, and
// a fallback each door had to remember is a fallback a third door will forget.
//
// `None` therefore means something narrower than it used to: no domain matched
// AND no default organization is configured.
//
// The platform tenant is excluded from both halves. Membership of it is what
// the enterprise console authorises against, so a claimed domain resolving to
// it would mint super-admins out of a customer's SSO; the loader separately
// refuses to let the platform tenant be the default.
pub async fn find_organization_for_new_user(
    pool: &PgPool,
    email: &str,
) -> Result<Option<String>, MarketplaceError> {
    if let Some(id) = find_organization_for_email(pool, email).await? {
        return Ok(Some(id));
    }
    find_default_organization(pool).await
}

// Why: The organization an otherwise-unclaimed user joins. A partial unique
// index guarantees at most one, so this is `fetch_optional` over a set the
// database keeps singular rather than a `LIMIT 1` over an ambiguous one.
pub async fn find_default_organization(pool: &PgPool) -> Result<Option<String>, MarketplaceError> {
    let id = sqlx::query_scalar!(
        "SELECT id FROM organizations
         WHERE status = 'active' AND NOT is_platform AND is_default"
    )
    .fetch_optional(pool)
    .await?;
    Ok(id)
}

// Why: The report and dashboard surfaces key on slug, not id, so the default
// organization is offered in both shapes rather than making every caller
// re-look-up what it just resolved.
pub async fn find_default_organization_slug(
    pool: &PgPool,
) -> Result<Option<String>, MarketplaceError> {
    let slug = sqlx::query_scalar!(
        "SELECT slug FROM organizations
         WHERE status = 'active' AND NOT is_platform AND is_default"
    )
    .fetch_optional(pool)
    .await?;
    Ok(slug)
}

// Why: The domain half of the match, split out so the fallback above reads as
// the policy it is. Suspended organizations are skipped: a lapsed contract must
// not keep absorbing new seats.
pub async fn find_organization_for_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<String>, MarketplaceError> {
    let Some((_, domain)) = email.rsplit_once('@').filter(|(_, d)| !d.is_empty()) else {
        return Ok(None);
    };
    let domain = domain.to_lowercase();

    let id = sqlx::query_scalar!(
        "SELECT id FROM organizations
         WHERE status = 'active' AND NOT is_platform AND $1 = ANY(email_domains)
         LIMIT 1",
        domain
    )
    .fetch_optional(pool)
    .await?;
    Ok(id)
}
