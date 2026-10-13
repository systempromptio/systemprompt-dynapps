//! Core's SSO driver for this installation: install the Entra ID SAML and
//! OIDC configs into the identity crate's provider slots, then discover.
//!
//! Order matters — discovery calls each provider's factory once, so a config
//! installed afterwards is never seen.

use std::sync::Arc;

use systemprompt::manifest::Config;
use systemprompt::oauth::SessionCreationService;
use systemprompt::sso::SsoDriver;
use systemprompt_web_admin_identity::SsoDeps;
use systemprompt_web_admin_identity::sso::{entra_oidc, entra_saml};

use super::pools::DbHandles;
use crate::extension::WebExtension;

pub(super) fn build(
    db: &DbHandles,
    session_service: &Arc<SessionCreationService>,
) -> Option<SsoDeps> {
    if let Some(cfg) = WebExtension::entra_saml_config() {
        entra_saml::install_config(cfg);
    }
    if let Some(cfg) = WebExtension::entra_oidc_config() {
        entra_oidc::install_config(cfg);
    }
    let config = Config::get()
        .map_err(|e| tracing::error!(error = %e, "Profile config unavailable; no SSO driver"))
        .ok()?;
    let driver = SsoDriver::discover(
        &db.db,
        Arc::clone(session_service),
        Arc::new(config.clone()),
    );
    Some(SsoDeps {
        driver,
        write_pool: Arc::clone(&db.write),
        session_service: Arc::clone(session_service),
    })
}
