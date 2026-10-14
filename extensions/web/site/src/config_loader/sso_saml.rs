//! Loader for the SSO config (`services/web/config/adfs*.yaml`), a SAML
//! relying party of Microsoft Entra ID.
//!
//! Split from the parent module to keep it inside the file-size gate. The two
//! SAML URLs are derived here from the profile rather than authored in YAML,
//! so one committed config is correct on every deployment host.
//! `adfs.<profile>.yaml` overrides the committed `adfs.yaml` and names its own
//! metadata, so two tenants' signing certificates sit side by side. With
//! `idp_metadata_url` set the metadata is fetched at run time and the file is
//! an optional fallback; without it a missing metadata file is a config
//! error, never a silent "SSO unavailable". A disabled config reads no file.
//! The metadata refresh itself belongs to the provider built from this
//! config.

use std::sync::Arc;

use systemprompt::config::ProfileBootstrap;

use super::{ConfigError, load_app_paths, load_config_section};

fn saml_config_filename() -> String {
    ProfileBootstrap::get().map_or_else(
        |_| "adfs.yaml".to_owned(),
        |profile| format!("adfs.{}.yaml", profile.name),
    )
}

// Why: Microsoft Entra ID answers `AADSTS700016`/`AADSTS50011` when the
// Identifier or Reply URL is not this host's own URL, so both derive from
// `api_external_url`; an explicit value still wins for a trust registered under
// another URL.
fn derive_saml_urls(config: &mut systemprompt_web_admin_identity::EntraSamlConfig) {
    if !config.entity_id.is_empty() && !config.acs_url.is_empty() {
        return;
    }
    let Ok(global) = systemprompt::manifest::Config::get().inspect_err(|e| {
        tracing::warn!(error = %e, "No profile config; SSO SAML URLs stay unset");
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
        "Derived SSO SAML URLs from the profile"
    );
}

pub(super) fn load_entra_saml_config()
-> Result<Option<Arc<systemprompt_web_admin_identity::EntraSamlConfig>>, ConfigError> {
    let profile_filename = saml_config_filename();
    let (filename, value) = match load_config_section(&profile_filename)? {
        Some(value) => (profile_filename, value),
        None => match load_config_section("adfs.yaml")? {
            Some(value) => ("adfs.yaml".to_owned(), value),
            None => return Ok(None),
        },
    };

    let mut config: systemprompt_web_admin_identity::EntraSamlConfig =
        serde_yaml::from_value(value).map_err(|e| ConfigError::Parse {
            config_name: filename.clone(),
            message: e.to_string(),
        })?;

    derive_saml_urls(&mut config);

    if config.enabled {
        read_pinned_metadata(&mut config, &filename)?;
    }

    tracing::info!(
        enabled = config.enabled,
        file = %filename,
        "Loaded SSO config"
    );

    Ok(Some(Arc::new(config)))
}

fn read_pinned_metadata(
    config: &mut systemprompt_web_admin_identity::EntraSamlConfig,
    filename: &str,
) -> Result<(), ConfigError> {
    let fetched = config.idp_metadata_url.is_some();
    if config.idp_metadata_path.is_empty() {
        return if fetched {
            Ok(())
        } else {
            Err(ConfigError::Parse {
                config_name: filename.to_owned(),
                message: "enabled SSO needs idp_metadata_url or idp_metadata_path".to_owned(),
            })
        };
    }
    let metadata_path = load_app_paths()?
        .system()
        .services()
        .join("web/config")
        .join(&config.idp_metadata_path);
    match std::fs::read_to_string(&metadata_path) {
        Ok(xml) => config.idp_metadata_xml = xml,
        Err(e) if fetched => tracing::warn!(
            error = %e,
            path = %metadata_path.display(),
            "Pinned IdP metadata unreadable; relying on idp_metadata_url alone"
        ),
        Err(e) => {
            return Err(ConfigError::Parse {
                config_name: filename.to_owned(),
                message: format!(
                    "idp_metadata_path {} unreadable: {e}",
                    metadata_path.display()
                ),
            });
        },
    }
    Ok(())
}
