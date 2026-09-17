//! The resolved caller identity carried through admin request handling.

use serde::Serialize;
use systemprompt::identifiers::{Email, SessionId, UserId};

use super::role::{ROLES_CONSOLE, ROLES_PLATFORM, Role, has_any};
use crate::repositories::scope::Participation;

#[derive(Debug, Clone, Serialize)]
pub struct UserContext {
    pub user_id: UserId,
    pub username: String,
    pub email: Email,
    pub roles: Vec<String>,
    // Why: the groups and projects the caller belongs to, resolved once per
    // request. Listings narrow to these for a caller who may not see the
    // whole estate, so they are part of identity rather than something each
    // handler re-reads.
    pub group_ids: Vec<String>,
    pub project_ids: Vec<String>,
    // Why: the marketplaces a non-console caller reaches through those
    // memberships, and the containers those marketplaces name. It is what the
    // scoped Analysis and Code sync pages run on, so it is resolved once with
    // the identity. Console callers carry an empty value: they see everything.
    pub participation: Participation,
    // Why: `is_admin` is the write tier — `admin` or `platform_admin` — and
    // guards every privileged mutation. `is_console` is "sees the whole
    // instance"; today both are the same two roles, and they stay separate
    // fields because the read and write questions are still asked apart.
    pub is_admin: bool,
    pub is_console: bool,
    pub is_platform_admin: bool,
    pub is_developer: bool,
    pub email_verified: bool,
    pub session_id: Option<SessionId>,
}

impl UserContext {
    // Why: whether the caller may open the scoped console at all — an admin,
    // or a person who participates in at least one marketplace. A signed-in
    // account with neither has only the Account section.
    #[must_use]
    pub fn has_scoped_console(&self) -> bool {
        self.is_console || !self.participation.is_empty()
    }
}

// Why: whether a role set reaches the admin dashboard. Kept beside
// `UserContext` so the middleware and every test fixture derive the flag from
// one rule instead of each restating it.
#[must_use]
pub fn roles_grant_console(roles: &[String]) -> bool {
    has_any(roles, ROLES_CONSOLE)
}

#[must_use]
pub fn roles_grant_manage(roles: &[String]) -> bool {
    systemprompt_mcp_shared::access_policy::roles_grant_manage(roles)
}

#[must_use]
pub fn roles_grant_platform(roles: &[String]) -> bool {
    has_any(roles, ROLES_PLATFORM)
}

#[must_use]
pub fn roles_grant_developer(roles: &[String]) -> bool {
    has_any(roles, &[Role::Developer])
}
