//! The four principals every route is driven under, and the credentials that
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
    // A valid session for a user holding the semi-admin `project_manager`
    // role: the read-only admin dashboard, no write route, no admin MCP.
    ProjectManager,
}

impl Principal {
    pub const ALL: [Self; 4] = [
        Self::Anonymous,
        Self::NonAdmin,
        Self::Admin,
        Self::ProjectManager,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Anonymous => "anonymous",
            Self::NonAdmin => "non-admin",
            Self::Admin => "admin",
            Self::ProjectManager => "project-manager",
        }
    }
}

// Bearer tokens for the two authenticated principals, minted once and reused
// across the whole table.
pub struct Credentials {
    pub non_admin: String,
    pub admin: String,
    pub project_manager: String,
    // The non-admin's own user id, so a case can seed a row that principal
    // genuinely owns and drive an owner-facing route as its owner.
    pub non_admin_user_id: UserId,
}

impl Credentials {
    pub fn token_for(&self, principal: Principal) -> Option<&str> {
        match principal {
            Principal::Anonymous => None,
            Principal::NonAdmin => Some(&self.non_admin),
            Principal::Admin => Some(&self.admin),
            Principal::ProjectManager => Some(&self.project_manager),
        }
    }
}

// Seed one admin and one plain user, and mint a token for each.
//
// The plain user carries a project, as every SSO-provisioned user does; the
// admin carries none, as an operator account that never came through AD FS
// would. `admin` alone is what makes a system-wide admin here.
pub async fn provision(pool: &PgPool) -> Credentials {
    let (non_admin, non_admin_user_id) =
        provision_one(pool, "contract-user", &["user"], Some("core")).await;
    let (admin, _) = provision_one(pool, "contract-admin", &["admin", "user"], None).await;
    // The project manager carries a project as an SSO-provisioned account
    // does, even though `project_manager` is exempt from the project gate.
    let (project_manager, _) = provision_one(
        pool,
        "contract-pm",
        &["project_manager", "user"],
        Some("core"),
    )
    .await;
    Credentials {
        non_admin,
        admin,
        project_manager,
        non_admin_user_id,
    }
}

async fn provision_one(
    pool: &PgPool,
    name: &str,
    roles: &[&str],
    project: Option<&str>,
) -> (String, UserId) {
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
    if let Some(project) = project {
        sqlx::query("INSERT INTO user_profile_ext (user_id, project) VALUES ($1, $2)")
            .bind(user_id.as_str())
            .bind(project)
            .execute(pool)
            .await
            .expect("seed the principal's project");
    }

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

    (token.as_str().to_owned(), user_id)
}
