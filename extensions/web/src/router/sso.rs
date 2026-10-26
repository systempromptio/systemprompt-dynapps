//! Core's SSO driver for this installation. Discovery calls each registered
//! provider's factory once; the Entra ID factory loads its own
//! `services/web/config/entra-oidc.yaml` from the active services root.

use std::sync::Arc;

use systemprompt::oauth::SessionCreationService;
use systemprompt::sso::{SsoDriver, SsoHookContext};
use systemprompt_web_admin_identity::SsoDeps;

use super::pools::DbHandles;

pub(super) fn build(db: &DbHandles, session_service: &Arc<SessionCreationService>) -> SsoDeps {
    let ctx = SsoHookContext {
        db: Arc::clone(&db.db),
        config: Arc::clone(db.boot.config_arc()),
        services: db.services.view().clone(),
        services_root: db.boot.active_services_root().clone(),
        secrets: Arc::clone(db.boot.secrets()),
    };
    let driver = SsoDriver::discover(&ctx, db.oauth.oauth.clone(), Arc::clone(session_service));
    SsoDeps {
        driver: Some(driver),
        write_pool: Arc::clone(&db.write),
        session_service: Arc::clone(session_service),
    }
}
