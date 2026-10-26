//! The installation's in-process authz hook (`governance.authz.hook.mode:
//! extension`).
//!
//! Core composes its `RuleBasedHook` ahead of this one and calls it on every
//! gateway request and MCP enforcement point. Registered through
//! `inventory`, so every binary that links this crate gets it whatever its
//! server role: the gateway no longer posts each request to
//! `/api/public/govern/authz`, which only the admin role serves. The pool is
//! the one core hands every hook factory, beside the booted services view. The
//! decision is [`crate::authz_eval::evaluate`], the same one the webhook
//! answers.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::governance::authz::{
    AuthzDecision, AuthzDecisionHook, AuthzRequest, SharedAuthzHook,
};
use systemprompt_web_admin_core::extension_services::ExtensionServices;

#[derive(Debug, Clone)]
pub struct InstallationAuthzHook {
    pool: Arc<PgPool>,
    services: ExtensionServices,
}

impl InstallationAuthzHook {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>, services: ExtensionServices) -> Self {
        Self { pool, services }
    }
}

#[async_trait::async_trait]
impl AuthzDecisionHook for InstallationAuthzHook {
    async fn evaluate(&self, req: AuthzRequest) -> AuthzDecision {
        crate::authz_eval::evaluate(&self.pool, &self.services, req).await
    }

    async fn probe(&self, req: AuthzRequest) -> AuthzDecision {
        crate::authz_eval::probe(&self.pool, &self.services, req).await
    }
}

systemprompt::governance::register_authz_hook!(|ctx| {
    let hook: SharedAuthzHook = Arc::new(InstallationAuthzHook::new(
        Arc::clone(&ctx.pool),
        ExtensionServices::from_hook(ctx),
    ));
    hook
});
