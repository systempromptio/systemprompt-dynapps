//! "Sign in with Dynapps SSO (Microsoft Entra ID)" — SAML 2.0 Web Browser
//! SSO against an Entra ID enterprise application, served by core's SSO
//! driver at this installation's historical paths.
//!
//! Module layout: [`config`] (the loaded YAML and the slot the aggregator
//! installs it into), [`metadata`] (the federation metadata in force),
//! [`provider`] (the [`SsoProvider`] and `start`), [`finish`] (assertion
//! verification), [`claims`] (the pure assertion → identity read).

mod claims;
mod config;
mod finish;
mod metadata;
mod provider;

use std::sync::Arc;

use systemprompt::sso::{SsoHookContext, SsoProvider};

pub use claims::AssertionClaims;
pub use config::{
    ACS_PATH, EntraSamlConfig, GROUPS_OVERAGE_ATTRIBUTE, METADATA_PATH, START_PATH, install_config,
};
pub use metadata::REFRESH_INTERVAL as IDP_METADATA_REFRESH_INTERVAL;
pub use provider::EntraSamlProvider;

pub const ID: &str = "entra-saml";

/// `None` — SSO off — when no config was installed or it is not usable.
pub fn factory(_ctx: &SsoHookContext) -> Option<Arc<dyn SsoProvider>> {
    let cfg = config::installed_config().filter(|cfg| cfg.is_usable())?;
    let issuer = cfg
        .idp_entity_id
        .clone()
        .filter(|id| !id.is_empty())
        .or_else(|| metadata::entity_id_of(&cfg.idp_metadata_xml))
        .or_else(|| cfg.idp_metadata_url.clone())?;
    Some(Arc::new(EntraSamlProvider::new(cfg, issuer)))
}
