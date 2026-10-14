//! Who is calling, and which runs they may see.
//!
//! A [`Viewer`] is the caller's identity envelope reduced to what the tools
//! decide on: user id, roles, project memberships, and whether the roles
//! grant administration. [`authorize_run`] is the one rule every run tool
//! applies: a person reaches a run they launched or that ran as them, a run
//! of a project they belong to, and nothing else; an administrator reaches
//! every run.

use rmcp::ErrorData;
use sqlx::PgPool;
use systemprompt::agent_runs::AgentRunRow;
use systemprompt::identifiers::{Email, UserId};
use systemprompt_mcp_shared::reply::forbidden;
use systemprompt_web_shared::ProjectId;

use crate::error::{ToolError, ToolResult};
use systemprompt_web_admin_core::repositories::principal::identity::find_identity_envelope;
use systemprompt_web_admin_core::repositories::scope::Participation;
use systemprompt_web_admin_core::types::{
    Role, UserContext, roles_grant_console, roles_grant_developer, roles_grant_manage,
    roles_grant_platform,
};

#[derive(Debug, Clone)]
pub struct Viewer {
    pub user_id: UserId,
    pub is_admin: bool,
    pub roles: Vec<String>,
    pub project_ids: Vec<String>,
    context: Option<UserContext>,
}

/// Who a run belongs to beyond core's own row: the person who launched it
/// when the row ran as a service account, and the project its origin sits in.
#[derive(Debug, Clone, Default)]
pub struct RunOrigin {
    pub launched_by: Option<UserId>,
    pub project_id: Option<ProjectId>,
}

impl Viewer {
    #[must_use]
    pub fn from_parts(
        user_id: UserId,
        email: Email,
        roles: Vec<String>,
        group_ids: Vec<String>,
        project_ids: Vec<String>,
    ) -> Self {
        let is_admin = roles_grant_manage(&roles)
            || roles.iter().any(|role| role == Role::SuperAdmin.as_str());
        let context = UserContext {
            user_id: user_id.clone(),
            username: user_id.as_str().to_owned(),
            email,
            is_admin,
            is_console: roles_grant_console(&roles),
            is_platform_admin: roles_grant_platform(&roles),
            is_developer: roles_grant_developer(&roles),
            roles: roles.clone(),
            group_ids,
            project_ids: project_ids.clone(),
            participation: Participation::default(),
            email_verified: false,
            session_id: None,
        };
        Self {
            user_id,
            is_admin,
            roles,
            project_ids,
            context: Some(context),
        }
    }

    pub fn context(&self) -> ToolResult<&UserContext> {
        self.context
            .as_ref()
            .ok_or_else(|| ToolError::Failure("the caller's identity was not resolved".to_owned()))
    }

    // Why: the tool list is rendered from handler instances before any
    // caller exists; this viewer holds nothing and authorises nothing.
    #[must_use]
    pub fn for_listing() -> Self {
        Self {
            user_id: UserId::new(String::new()),
            is_admin: false,
            roles: Vec::new(),
            project_ids: Vec::new(),
            context: None,
        }
    }

    #[must_use]
    pub fn is_member_of(&self, project: &str) -> bool {
        self.project_ids.iter().any(|id| id == project)
    }
}

pub async fn viewer_for(pool: &PgPool, user_id: &UserId) -> ToolResult<Viewer> {
    let envelope = find_identity_envelope(pool, user_id)
        .await?
        .ok_or_else(|| ToolError::Forbidden("the signed-in user no longer exists".to_owned()))?;
    let email = Email::try_new(envelope.email)?;
    Ok(Viewer::from_parts(
        envelope.user_id,
        email,
        envelope.roles,
        envelope.group_ids,
        envelope.project_ids,
    ))
}

pub fn authorize_run(
    viewer: &Viewer,
    run: &AgentRunRow,
    origin: &RunOrigin,
) -> Result<(), ErrorData> {
    if viewer.is_admin || run.user_id == viewer.user_id {
        return Ok(());
    }
    if origin
        .launched_by
        .as_ref()
        .is_some_and(|launcher| *launcher == viewer.user_id)
    {
        return Ok(());
    }
    if origin
        .project_id
        .as_ref()
        .is_some_and(|project| viewer.is_member_of(project.as_str()))
    {
        return Ok(());
    }
    Err(forbidden(format!(
        "run {} is not yours and is not in a project you belong to",
        run.id
    )))
}
