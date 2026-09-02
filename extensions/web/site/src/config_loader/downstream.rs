//! Loaders for the downstream-credential configs (Atlassian, Salesforce).
//!
//! Split from the parent module to keep it inside the file-size gate. Both are
//! optional: absent means "not configured", the accessors report themselves
//! unavailable, and the MCP servers that depend on them ship disabled.

use std::sync::Arc;

use crate::config_loader::ConfigError;

use super::load_config_section;

// Why: both downstream-credential configs load the same way — a flat section
// deserialized into the admin crate's type, absent meaning "not configured"
// rather than an error. Neither is required for the server to run: the
// accessors report themselves unavailable and the MCP servers that use them
// ship disabled.
fn load_downstream_config<T: serde::de::DeserializeOwned>(
    filename: &str,
    label: &str,
) -> Result<Option<Arc<T>>, ConfigError> {
    let Some(value) = load_config_section(filename)? else {
        return Ok(None);
    };
    let config: T = serde_yaml::from_value(value).map_err(|e| ConfigError::Parse {
        config_name: filename.to_owned(),
        message: e.to_string(),
    })?;
    tracing::info!(file = %filename, "Loaded {label} config");
    Ok(Some(Arc::new(config)))
}

// Why: the callback URL has to name the host the browser came back to, and
// that host is a per-deployment fact the profile already carries. Deriving it
// from `api_external_url` keeps one committed `atlassian.yaml` correct on every
// instance; a literal in the tracked file was only ever right on one of them.
// An explicit `redirect_uri` still wins, for an app registered against a URL
// this instance does not serve itself under.
pub(super) fn load_atlassian_config()
-> Result<Option<Arc<systemprompt_web_admin::AtlassianConfig>>, ConfigError> {
    let Some(config) = load_downstream_config::<systemprompt_web_admin::AtlassianConfig>(
        "atlassian.yaml",
        "Atlassian",
    )?
    else {
        return Ok(None);
    };
    if !config.redirect_uri.is_empty() {
        return Ok(Some(config));
    }
    let Some(external) = external_url() else {
        return Ok(Some(config));
    };
    let mut derived = (*config).clone();
    derived.redirect_uri = format!(
        "{external}{}",
        systemprompt_web_admin::ATLASSIAN_CALLBACK_PATH
    );
    tracing::info!(redirect_uri = %derived.redirect_uri, "Derived Atlassian redirect_uri from the profile");
    Ok(Some(Arc::new(derived)))
}

// Why: absent config is not an error here — the accessor reports itself
// unavailable, which is the same outcome as an unconfigured install.
fn external_url() -> Option<String> {
    systemprompt::models::Config::get()
        .inspect_err(
            |e| tracing::warn!(error = %e, "No profile config; Atlassian redirect_uri stays unset"),
        )
        .ok()
        .map(|c| c.api_external_url.trim_end_matches('/').to_owned())
        .filter(|url| !url.is_empty())
}

pub(super) fn load_salesforce_config()
-> Result<Option<Arc<systemprompt_web_admin::SalesforceConfig>>, ConfigError> {
    load_downstream_config("salesforce.yaml", "Salesforce")
}
