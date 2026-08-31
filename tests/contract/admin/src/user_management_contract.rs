//! HTTP-level authorization contract for user provisioning: operator-created
//! accounts, the org-membership endpoints, and the narrow grant that lets an
//! organization's own admin manage their members' roles.
//!
//! These are the rules that decide whether one customer's administrator can
//! reach another customer's people, so they are asserted over the real router
//! with real tokens rather than at the repository layer, where the middleware
//! and handler guards would not run at all.

use axum::http::StatusCode;
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt_security::{AdminTokenParams, JwtService};

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

// An organization admin: the `admin` role, owner of their own org, and
// deliberately NOT a member of the platform tenant.
struct OrgAdmin {
    token: String,
    org_slug: String,
}

async fn seed_org(pool: &PgPool, slug: &str) -> String {
    let id = format!("{slug}-{}", uuid::Uuid::new_v4().simple());
    sqlx::query(
        "INSERT INTO organizations (id, slug, name, status)
         VALUES ($1, $2, $2, 'active')",
    )
    .bind(&id)
    .bind(slug)
    .execute(pool)
    .await
    .expect("seed organization");
    id
}

async fn seed_member(pool: &PgPool, org_id: &str, roles: &[&str], org_role: &str) -> UserId {
    let user_id = UserId::new(format!("member-{}", uuid::Uuid::new_v4().simple()));
    let email = format!("{}@contract.test", user_id.as_str());
    sqlx::query(
        "INSERT INTO users (id, name, email, roles, email_verified)
         VALUES ($1, $1, $2, $3, true)",
    )
    .bind(user_id.as_str())
    .bind(&email)
    .bind(roles.iter().map(|r| (*r).to_owned()).collect::<Vec<_>>())
    .execute(pool)
    .await
    .expect("seed member");
    sqlx::query(
        "INSERT INTO organization_members (user_id, org_id, org_role) VALUES ($1, $2, $3)
         ON CONFLICT (user_id) DO UPDATE SET org_id = EXCLUDED.org_id, org_role = EXCLUDED.org_role",
    )
    .bind(user_id.as_str())
    .bind(org_id)
    .bind(org_role)
    .execute(pool)
    .await
    .expect("seed membership");
    user_id
}

fn mint(user_id: &UserId) -> String {
    let session_id = SessionId::new(uuid::Uuid::new_v4().to_string());
    JwtService::generate_admin_token(&AdminTokenParams {
        user_id,
        session_id: &session_id,
        email: &format!("{}@contract.test", user_id.as_str()),
        issuer: &globals::jwt_issuer(),
        duration: chrono::Duration::hours(1),
        client_id: None,
    })
    .expect("mint a session token")
    .as_str()
    .to_owned()
}

async fn seed_org_admin(pool: &PgPool, slug: &str) -> (OrgAdmin, String) {
    let org_id = seed_org(pool, slug).await;
    let admin = seed_member(pool, &org_id, &["admin", "user"], "owner").await;
    (
        OrgAdmin {
            token: mint(&admin),
            org_slug: slug.to_owned(),
        },
        org_id,
    )
}

// Why: this rule used to be asserted against `POST /invites`. Invites are gone
// — SSO is the only door and an AD group is the invitation — but the escalation
// rule they carried still governs the surviving operator-created door, so it is
// re-asserted there rather than deleted with them.
#[tokio::test]
async fn an_org_admin_may_not_create_a_user_with_elevated_roles() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let (org_admin, _) = seed_org_admin(&db.pool, "acme").await;

    let (status, body) = app
        .call_with_bearer(
            Call::json(
                "post",
                "/api/public/admin/users",
                Principal::Anonymous,
                r#"{"user_id":"acme-newcomer","display_name":"Newcomer","email":"newcomer@acme.test","roles":["admin"]}"#,
            ),
            &org_admin.token,
        )
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "body: {body}");
    assert!(
        body.to_lowercase().contains("platform"),
        "the refusal must name the reason: {body}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn listing_organizations_is_platform_admin_only() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let (org_admin, _) = seed_org_admin(&db.pool, "acme").await;

    let (status, _) = app
        .call_with_bearer(
            Call::get(
                "/api/public/admin/management/organizations",
                Principal::Anonymous,
            ),
            &org_admin.token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = app
        .call(Call::get(
            "/api/public/admin/management/organizations",
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK);
    db.cleanup().await;
}

#[tokio::test]
async fn an_org_admin_may_promote_their_own_member_but_not_beyond() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let (org_admin, org_id) = seed_org_admin(&db.pool, "acme").await;
    let member = seed_member(&db.pool, &org_id, &["user"], "member").await;
    let owner = seed_member(&db.pool, &org_id, &["user"], "owner").await;

    let path = format!(
        "/api/public/admin/management/users/{}/organization",
        member.as_str()
    );
    let body = format!(r#"{{"org":"{}","org_role":"admin"}}"#, org_admin.org_slug);
    let (status, response) = app
        .call_with_bearer(
            Call::json("put", &path, Principal::Anonymous, &body),
            &org_admin.token,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "member -> admin is allowed: {response}"
    );

    let body = format!(r#"{{"org":"{}","org_role":"owner"}}"#, org_admin.org_slug);
    let (status, _) = app
        .call_with_bearer(
            Call::json("put", &path, Principal::Anonymous, &body),
            &org_admin.token,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "assigning owner stays a platform act"
    );

    let owner_path = format!(
        "/api/public/admin/management/users/{}/organization",
        owner.as_str()
    );
    let body = format!(r#"{{"org":"{}","org_role":"member"}}"#, org_admin.org_slug);
    let (status, _) = app
        .call_with_bearer(
            Call::json("put", &owner_path, Principal::Anonymous, &body),
            &org_admin.token,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "an owner's role may not be changed by an org admin"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn an_org_admin_may_not_touch_another_organizations_member() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let (acme_admin, _) = seed_org_admin(&db.pool, "acme").await;
    let rival_id = seed_org(&db.pool, "rival").await;
    let outsider = seed_member(&db.pool, &rival_id, &["user"], "member").await;

    let path = format!(
        "/api/public/admin/management/users/{}/organization",
        outsider.as_str()
    );

    // Naming their own org: the target is not a member of it, so this would be
    // an addition, which org admins may not perform.
    let body = format!(r#"{{"org":"{}","org_role":"member"}}"#, acme_admin.org_slug);
    let (status, _) = app
        .call_with_bearer(
            Call::json("put", &path, Principal::Anonymous, &body),
            &acme_admin.token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "no adding outsiders");

    // Naming the other org: a cross-tenant move.
    let (status, _) = app
        .call_with_bearer(
            Call::json(
                "put",
                &path,
                Principal::Anonymous,
                r#"{"org":"rival","org_role":"admin"}"#,
            ),
            &acme_admin.token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "no cross-tenant moves");
    db.cleanup().await;
}

#[tokio::test]
async fn creating_a_user_hands_back_no_link_and_says_so() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // An org that claims the email domain, so the created user has somewhere
    // to belong when SSO adopts the row.
    let org_id = seed_org(&db.pool, "claimed").await;
    sqlx::query("UPDATE organizations SET email_domains = ARRAY['claimed.test'] WHERE id = $1")
        .bind(&org_id)
        .execute(&*db.pool)
        .await
        .expect("claim the domain");

    let (status, body) = app
        .call(Call::json(
            "post",
            "/api/public/admin/users",
            Principal::Admin,
            r#"{"user_id":"claimed-newcomer","display_name":"Newcomer","email":"newcomer@claimed.test","roles":["user"],"department":"Engineering"}"#,
        ))
        .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let created: serde_json::Value = serde_json::from_str(&body).expect("created json");
    assert!(
        created.get("invite_path").is_none(),
        "there is no link to hand out — SSO is the only door: {body}"
    );
    assert!(
        created["invite_note"]
            .as_str()
            .is_some_and(|n| n.contains("SSO")),
        "the operator is told how the person signs in instead: {body}"
    );

    let department: String = sqlx::query_scalar(
        "SELECT department FROM user_profile_ext WHERE user_id = 'claimed-newcomer'",
    )
    .fetch_one(&*db.pool)
    .await
    .expect("the department was persisted");
    assert_eq!(department, "Engineering");
    db.cleanup().await;
}

#[tokio::test]
async fn creating_a_user_on_an_unclaimed_domain_says_so() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app
        .call(Call::json(
            "post",
            "/api/public/admin/users",
            Principal::Admin,
            r#"{"user_id":"orphan-newcomer","display_name":"Orphan","email":"orphan@nobody-claims-this.test","roles":["user"]}"#,
        ))
        .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let created: serde_json::Value = serde_json::from_str(&body).expect("created json");
    assert!(
        created["invite_note"]
            .as_str()
            .is_some_and(|n| n.contains("organization")),
        "the operator must be told the domain claims no organization: {body}"
    );
    db.cleanup().await;
}
