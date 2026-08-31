//! The three principals every route is driven under, and the credentials that
//! distinguish them.
//!
//! Role membership is deliberately *not* carried in the JWT:
//! `user_context_middleware` reads `users.roles` from the database, so the
//! admin / non-admin split is seeded as table rows and the middleware resolves
//! it the same way it does in production. The token only has to validate and
//! carry a subject.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt_security::{AdminTokenParams, JwtService};

use crate::globals;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Principal {
    // No cookie and no `Authorization` header.
    Anonymous,
    // A valid session for a user holding only the `user` role.
    NonAdmin,
    // A valid session for a user holding `admin`.
    Admin,
}

impl Principal {
    pub const ALL: [Self; 3] = [Self::Anonymous, Self::NonAdmin, Self::Admin];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Anonymous => "anonymous",
            Self::NonAdmin => "non-admin",
            Self::Admin => "admin",
        }
    }
}

// Bearer tokens for the two authenticated principals, minted once and reused
// across the whole table.
pub struct Credentials {
    pub non_admin: String,
    pub admin: String,
}

impl Credentials {
    pub fn token_for(&self, principal: Principal) -> Option<&str> {
        match principal {
            Principal::Anonymous => None,
            Principal::NonAdmin => Some(&self.non_admin),
            Principal::Admin => Some(&self.admin),
        }
    }
}

// Seed one admin and one plain user, and mint a token for each.
//
// Neither is joined to an organization, and that is the point. This fixture
// used to insert an `organization_members` row for the admin so it could reach
// the platform console — manufacturing a state production never created, since
// nothing outside a boot job for the single configured system admin ever wrote
// that row. The suite therefore asserted 200 on `/admin/reports/customer` while
// every real admin got a 404 from it, and asserted a populated analytics
// dashboard while every real admin saw zeros.
//
// What the fixture seeds instead is what a deployed instance actually has: a
// default organization named in `plans.yaml`, and admins who are members of
// nothing. `admin` alone is what makes a system-wide admin here, and the
// default organization is what gives a report something to report on.
pub async fn provision(pool: &PgPool) -> Credentials {
    seed_default_organization(pool).await;
    let non_admin = provision_one(pool, "contract-user", &["user"]).await;
    let admin = provision_one(pool, "contract-admin", &["admin", "user"]).await;
    Credentials { non_admin, admin }
}

async fn provision_one(pool: &PgPool, name: &str, roles: &[&str]) -> String {
    let user_id = UserId::new(format!("{name}-{}", uuid::Uuid::new_v4().simple()));
    let email = format!("{name}@contract.test");

    sqlx::query(
        "INSERT INTO users (id, name, email, roles, email_verified)
         VALUES ($1, $2, $3, $4, true)",
    )
    .bind(user_id.as_str())
    .bind(user_id.as_str().to_owned())
    .bind(&email)
    .bind(roles.iter().map(|r| (*r).to_owned()).collect::<Vec<_>>())
    .execute(pool)
    .await
    .expect("seed contract principal");

    let session_id = SessionId::new(uuid::Uuid::new_v4().to_string());
    let token = JwtService::generate_admin_token(&AdminTokenParams {
        user_id: &user_id,
        session_id: &session_id,
        email: &email,
        issuer: &globals::jwt_issuer(),
        duration: chrono::Duration::hours(1),
        client_id: None,
    })
    .expect("mint a session token");

    token.as_str().to_owned()
}

// Seed the default organization: the one an otherwise-unclaimed user joins,
// and the one every org-scoped report falls back to.
//
// Deliberately seeds NO membership. A contract suite whose admin belongs to an
// organization cannot see the bug that an admin who belongs to none is shown a
// sidebar full of links that 404.
//
// `is_default` carries a partial unique index, so this adopts whichever default
// is already present rather than insisting on its own.
async fn seed_default_organization(pool: &PgPool) {
    sqlx::query(
        "INSERT INTO organizations (id, slug, name, is_default)
         VALUES ('contract-default', 'contract-default', 'Contract Default', TRUE)
         ON CONFLICT DO NOTHING",
    )
    .execute(pool)
    .await
    .expect("seed the default organization");
}
