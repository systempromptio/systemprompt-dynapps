//! What this installation records around core's SSO driver.
//!
//! That is the `login` activity, the directory group and project memberships
//! an assertion carried, and revocation when the directory refuses someone.
//! Roles are not
//! written here: [`super::roles`] settles them before the session is minted.
//!
//! A refused sign-in (`forbidden`, `no_group`, `group_overage`) revokes the
//! cookie sessions and bridge links the directory once vouched for; every
//! failure here is logged and never changes the sign-in's outcome.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::sso::{AuthEventSink, SsoHookContext, SsoLoginRefused, SsoLoginSuccess};
use systemprompt_web_admin_core::activity::NewActivity;
use systemprompt_web_admin_core::repositories::user_activity::record;

use crate::repositories::users::revocation;
use crate::repositories::{groups, projects};

const REVOKING_REASONS: [&str; 3] = ["forbidden", "no_group", "group_overage"];

#[derive(Debug)]
pub struct DynappsSsoEventSink {
    pool: Arc<PgPool>,
}

pub fn factory(ctx: &SsoHookContext) -> Arc<dyn AuthEventSink> {
    Arc::new(DynappsSsoEventSink {
        pool: ctx.db.write_pool(),
    })
}

#[async_trait]
impl AuthEventSink for DynappsSsoEventSink {
    async fn on_login_success(&self, event: &SsoLoginSuccess<'_>) {
        let pool = &*self.pool;
        let user_id = event.user_id;
        let identity = event.identity;
        record(pool, NewActivity::login(user_id, &identity.display_name)).await;
        if let Err(e) =
            groups::members::replace_directory_group_memberships(pool, user_id, &identity.groups)
                .await
        {
            tracing::warn!(error = %e, %user_id, "Failed to persist SSO group memberships");
        }
        if let Err(e) = projects::members::replace_directory_project_memberships(
            pool,
            user_id,
            &identity.groups,
        )
        .await
        {
            tracing::warn!(error = %e, %user_id, "Failed to persist SSO project memberships");
        }
    }

    async fn on_login_refused(&self, event: &SsoLoginRefused<'_>) {
        let reason = event.reason;
        if !REVOKING_REASONS.contains(&reason) {
            return;
        }
        let Some(email) = event.email else {
            return;
        };
        match revocation::revoke_access_by_email(&self.pool, email).await {
            Ok(Some((user_id, counts))) if !counts.is_empty() => tracing::warn!(
                %user_id,
                reason,
                sessions = counts.sessions,
                api_keys = counts.api_keys,
                device_certs = counts.device_certs,
                exchange_codes = counts.exchange_codes,
                "Revoked credentials after SSO refused the sign-in"
            ),
            Ok(_) => {},
            Err(e) => {
                tracing::error!(error = %e, reason, "Failed to revoke credentials after a refused SSO sign-in");
            },
        }
    }
}
