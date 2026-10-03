//! Loader for the ADFS SSO config (`services/web/config/adfs*.yaml`).
//!
//! Split from the parent module to keep it inside the file-size gate. The two
//! SAML URLs are derived here from the profile rather than authored in YAML,
//! so one committed config is correct on every deployment host.
//! `adfs.<profile>.yaml` overrides the committed `adfs.yaml` and names its own
//! `idp_metadata_path`, so two farms' signing certificates sit side by side. A
//! missing metadata file is a config error, never a silent "SSO unavailable".

use std::sync::Arc;

use systemprompt::config::ProfileBootstrap;

use super::{ConfigError, load_app_paths, load_config_section};

fn adfs_config_filename() -> String {
    ProfileBootstrap::get().map_or_else(
        |_| "adfs.yaml".to_owned(),
        |profile| format!("adfs.{}.yaml", profile.name),
    )
}

// Why: AD FS answers `MSIS7007` when the relying-party id or ACS is not this
// host's own URL, so both derive from `api_external_url`; an explicit value
// still wins for a trust registered under another URL.
fn derive_saml_urls(config: &mut systemprompt_web_admin_identity::AdfsConfig) {
    if !config.entity_id.is_empty() && !config.acs_url.is_empty() {
        return;
    }
    let Ok(global) = systemprompt::manifest::Config::get().inspect_err(|e| {
        tracing::warn!(error = %e, "No profile config; ADFS SAML URLs stay unset");
    }) else {
        return;
    };
    let base = global.api_external_url.trim_end_matches('/');
    if base.is_empty() {
        return;
    }
    if config.entity_id.is_empty() {
        config.entity_id = format!(
            "{base}{}",
            systemprompt_web_admin_identity::ADFS_METADATA_PATH
        );
    }
    if config.acs_url.is_empty() {
        config.acs_url = format!("{base}{}", systemprompt_web_admin_identity::ADFS_ACS_PATH);
    }
    tracing::info!(
        entity_id = %config.entity_id,
        acs_url = %config.acs_url,
        "Derived ADFS SAML URLs from the profile"
    );
}

pub(super) fn load_adfs_config()
-> Result<Option<Arc<systemprompt_web_admin_identity::AdfsConfig>>, ConfigError> {
    let profile_filename = adfs_config_filename();
    let (filename, value) = match load_config_section(&profile_filename)? {
        Some(value) => (profile_filename, value),
        None => match load_config_section("adfs.yaml")? {
            Some(value) => ("adfs.yaml".to_owned(), value),
            None => return Ok(None),
        },
    };

    let mut config: systemprompt_web_admin_identity::AdfsConfig = serde_yaml::from_value(value)
        .map_err(|e| ConfigError::Parse {
            config_name: filename.clone(),
            message: e.to_string(),
        })?;

    derive_saml_urls(&mut config);

    if config.enabled {
        let metadata_path = load_app_paths()?
            .system()
            .services()
            .join("web/config")
            .join(&config.idp_metadata_path);
        config.idp_metadata_xml =
            std::fs::read_to_string(&metadata_path).map_err(|e| ConfigError::Parse {
                config_name: filename.clone(),
                message: format!(
                    "idp_metadata_path {} unreadable: {e}",
                    metadata_path.display()
                ),
            })?;
    }

    tracing::info!(
        enabled = config.enabled,
        file = %filename,
        "Loaded ADFS SSO config"
    );

    Ok(Some(Arc::new(config)))
}
