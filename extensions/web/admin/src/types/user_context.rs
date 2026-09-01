//! The resolved caller identity carried through admin request handling.

use serde::Serialize;
use systemprompt::identifiers::{Email, SessionId, UserId};

use super::Project;

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
    pub email_verified: bool,
    pub session_id: Option<SessionId>,
}
