//! The resolved caller identity carried through admin request handling.

use serde::Serialize;
use systemprompt::identifiers::{Email, SessionId, UserId};

use super::Project;
use super::constants::{ROLE_ADMIN, ROLE_PROJECT_MANAGER};

#[derive(Debug, Clone, Serialize)]
pub struct UserContext {
    pub user_id: UserId,
    pub username: String,
    pub email: Email,
    // Why: The project AD FS placed the caller in. `None` only for operator
    // accounts that never signed in through the directory.
    pub project: Option<Project>,
    pub roles: Vec<String>,
    pub is_admin: bool,
    // Why: `is_admin` means the `admin` role and nothing else, because it
    // still guards every privileged mutation. `is_console` is the wider
    // "may see the admin dashboard" test — `admin` or `project_manager` —
    // and is what read-only pages and listings gate on.
    pub is_console: bool,
    pub email_verified: bool,
    pub session_id: Option<SessionId>,
}

// Why: whether a role set reaches the admin dashboard — `admin` and
// `project_manager` both do. Kept beside `UserContext` so the middleware and
// every test fixture derive the flag from one rule instead of each restating
// it.
#[must_use]
pub fn roles_grant_console(roles: &[String]) -> bool {
    roles
        .iter()
        .any(|r| r == ROLE_ADMIN || r == ROLE_PROJECT_MANAGER)
}
