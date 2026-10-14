//! Who is calling, what they may do here, and how Odoo is told.
//!
//! Core's RBAC proves which platform user is calling; this module reads that
//! user's live roles from the database on every call (a revoked
//! `odoo_destructive` grant takes effect on the next call, not the next
//! login) and decides the one authority this server owns: the guarded tools
//! need an administrator or the `odoo_destructive` role, for people and
//! service accounts alike. The same envelope supplies the attribution footer
//! posted to Odoo chatter, so a write made through the farm's bot login still
//! names the person or run behind it.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::repositories::principal::identity::find_identity_envelope;
use systemprompt_web_admin_core::types::{Role, roles_grant_manage};

use crate::error::OdooError;
use crate::tools::{GUARDED_TOOLS, TOOL_GDPR_ERASE_INVENTORY};

pub const DESTRUCTIVE_ROLE: &str = "odoo_destructive";
pub const DESTRUCTIVE_POLICY: &str = "odoo_destructive_role";
pub const FARM_PURPOSE_PREFIX: &str = "farm:";

#[derive(Debug, Clone)]
pub struct Caller {
    pub user_id: UserId,
    pub label: String,
    pub roles: Vec<String>,
    pub run_id: Option<String>,
}

#[must_use]
pub fn needs_destructive_authority(tool: &str) -> bool {
    GUARDED_TOOLS.contains(&tool) || tool == TOOL_GDPR_ERASE_INVENTORY
}

#[must_use]
pub fn roles_grant_destructive(roles: &[String]) -> bool {
    roles_grant_manage(roles)
        || roles
            .iter()
            .any(|role| role == DESTRUCTIVE_ROLE || role == Role::SuperAdmin.as_str())
}

#[must_use]
pub fn farm_run_id(caller_purpose: Option<&str>) -> Option<String> {
    caller_purpose
        .and_then(|purpose| purpose.strip_prefix(FARM_PURPOSE_PREFIX))
        .map(str::trim)
        .filter(|run| !run.is_empty())
        .map(ToOwned::to_owned)
}

impl Caller {
    #[must_use]
    pub fn may_act_destructively(&self) -> bool {
        roles_grant_destructive(&self.roles)
    }

    #[must_use]
    pub fn footer(&self) -> String {
        self.run_id.as_ref().map_or_else(
            || format!("via Dynapps · {}", self.label),
            |run| format!("via Dynapps · {} · farm run {run}", self.label),
        )
    }
}

pub async fn load_caller(
    pool: &PgPool,
    user_id: &UserId,
    caller_purpose: Option<&str>,
) -> Result<Caller, OdooError> {
    let envelope = find_identity_envelope(pool, user_id)
        .await
        .map_err(|e| OdooError::Internal(e.to_string()))?
        .ok_or_else(|| OdooError::AccessDenied("the calling user no longer exists".to_owned()))?;
    let label = envelope
        .display_name
        .filter(|name| !name.trim().is_empty())
        .map_or_else(
            || envelope.email.clone(),
            |name| format!("{name} <{}>", envelope.email),
        );
    Ok(Caller {
        user_id: envelope.user_id,
        label,
        roles: envelope.roles,
        run_id: farm_run_id(caller_purpose),
    })
}
