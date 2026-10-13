//! The roles an SSO sign-in leaves a user with, decided before core's driver
//! writes them and mints the session.
//!
//! `users.roles` has two writers: the directory, through the identity policy's
//! group mapping, and an admin granting roles by hand (`user_manual_roles`).
//! The resolver returns their union, so a sign-in never erases a manual grant.
//! A claim that maps no roles is not a demotion: the account keeps what it
//! holds, which is how an operator created with `admin users create` keeps its
//! CLI-granted roles.
//!
//! Losing every role that mints admin revokes what the account already holds.
//! That runs here rather than in the event sink because only here are the
//! previous roles still on the row, and because no session for this sign-in
//! exists yet, so the revocation cannot reach the one about to be minted.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt::sso::{RoleResolver, SsoError, SsoHookContext};
use systemprompt_web_admin_core::types::role::{ROLES_MANAGE, has_any};

use crate::repositories::users::{revocation, roles};

#[derive(Debug)]
pub struct DynappsRoleResolver {
    pool: Arc<PgPool>,
}

impl DynappsRoleResolver {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

pub fn factory(ctx: &SsoHookContext) -> Arc<dyn RoleResolver> {
    Arc::new(DynappsRoleResolver::new(ctx.db.write_pool()))
}

#[async_trait]
impl RoleResolver for DynappsRoleResolver {
    async fn resolve(
        &self,
        user_id: &UserId,
        directory_roles: &[String],
    ) -> Result<Vec<String>, SsoError> {
        let pool = &*self.pool;
        let manual = roles::list_manual_roles(pool, user_id)
            .await
            .map_err(SsoError::internal)?;
        let held = roles::list_directory_roles(pool, user_id)
            .await
            .map_err(SsoError::internal)?;
        let previous = union(&held, &manual);
        if directory_roles.is_empty() {
            return Ok(previous);
        }
        let resolved = union(directory_roles, &manual);
        if has_any(&previous, ROLES_MANAGE) && !has_any(&resolved, ROLES_MANAGE) {
            revoke_after_demotion(pool, user_id).await;
        }
        Ok(resolved)
    }
}

/// `first` then whatever of `second` it lacks, each role once.
#[must_use]
pub fn union(first: &[String], second: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(first.len() + second.len());
    for role in first.iter().chain(second) {
        if !out.contains(role) {
            out.push(role.clone());
        }
    }
    out
}

async fn revoke_after_demotion(pool: &PgPool, user_id: &UserId) {
    match revocation::revoke_user_access(pool, user_id).await {
        Ok(counts) => tracing::warn!(
            %user_id,
            sessions = counts.sessions,
            api_keys = counts.api_keys,
            "Revoked credentials after SSO removed the admin role"
        ),
        Err(e) => tracing::error!(
            error = %e,
            %user_id,
            "Failed to revoke credentials after an SSO admin demotion"
        ),
    }
}
