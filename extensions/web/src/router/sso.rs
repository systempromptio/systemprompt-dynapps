//! Core's SSO driver for this installation. Discovery calls each registered
//! provider's factory once; the Entra ID factory loads its own
//! `services/web/config/entra-oidc.yaml` from the active services root.

use std::sync::Arc;

use systemprompt::manifest::Config;
use systemprompt::oauth::SessionCreationService;
use systemprompt::sso::SsoDriver;
use systemprompt_web_admin_identity::SsoDeps;

use super::pools::DbHandles;

pub(super) fn build(db: &DbHandles, session_service: &Arc<SessionCreationService>) -> SsoDeps {
    let driver = match Config::get() {
        Ok(config) => Some(SsoDriver::discover(
            &db.db,
            Arc::clone(session_service),
            Arc::new(config.clone()),
            db.services.view().clone(),
        )),
        Err(e) => {
            tracing::warn!(error = %e, "Profile config unavailable; SSO routes answer unavailable");
            None
        },
    };
    SsoDeps {
        driver,
        write_pool: Arc::clone(&db.write),
        session_service: Arc::clone(session_service),
    }
}
