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
    pub group_ids: Vec<String>,
    pub project_ids: Vec<String>,
    pub participation: Participation,
    pub is_admin: bool,
    pub is_console: bool,
    pub is_platform_admin: bool,
    pub is_developer: bool,
    pub email_verified: bool,
    pub session_id: Option<SessionId>,
}

impl UserContext {
    #[must_use]
    pub const fn has_scoped_console(&self) -> bool {
        self.is_console || !self.participation.is_empty()
    }
}

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
