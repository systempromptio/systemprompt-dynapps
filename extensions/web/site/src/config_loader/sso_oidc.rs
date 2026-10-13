//! Loader for the Microsoft Entra ID OpenID Connect config
//! (`services/web/config/entra-oidc.yaml`).
//!
//! The absolute `redirect_uri` is derived from the profile's
//! `api_external_url` plus the file's `redirect_path`, as the SAML URLs are,
//! so one committed config is correct on every deployment host. A missing
//! file is a disabled provider, never an error.

use std::sync::Arc;

use systemprompt_web_admin_identity::sso::entra_oidc::EntraOidcConfig;

use super::{ConfigError, load_config_section};

const FILENAME: &str = "entra-oidc.yaml";

// Why: Microsoft Entra ID answers `AADSTS50011` when the redirect URI is not
// one registered on the App Registration, so it derives from this host's own
// `api_external_url`.
fn derive_redirect_uri(config: &mut EntraOidcConfig) {
    if config.redirect_uri.is_some() {
        return;
    }
    let Ok(global) = systemprompt::manifest::Config::get().inspect_err(|e| {
        tracing::warn!(error = %e, "No profile config; Entra OIDC redirect_uri stays unset");
    }) else {
        return;
    };
    let base = global.api_external_url.trim_end_matches('/');
    if base.is_empty() {
        return;
    }
    let uri = format!("{base}{}", config.redirect_path);
    tracing::info!(redirect_uri = %uri, "Derived Entra OIDC redirect_uri from the profile");
    config.redirect_uri = Some(uri);
}

pub(super) fn load_entra_oidc_config() -> Result<Arc<EntraOidcConfig>, ConfigError> {
    let Some(value) = load_config_section(FILENAME)? else {
        return Ok(Arc::new(EntraOidcConfig::default()));
    };
    let mut config: EntraOidcConfig =
        serde_yaml::from_value(value).map_err(|e| ConfigError::Parse {
            config_name: FILENAME.to_owned(),
            message: e.to_string(),
        })?;
    derive_redirect_uri(&mut config);
    tracing::info!(
        enabled = config.enabled,
        usable = config.is_usable(),
        "Loaded Entra OIDC config"
    );
    Ok(Arc::new(config))
}
