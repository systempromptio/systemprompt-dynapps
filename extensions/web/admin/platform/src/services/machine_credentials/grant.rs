//! A held machine credential as core's [`MachineGrant`].
//!
//! Core owns every minting step: the GitHub App JWT and installation-token
//! exchange, the Salesforce client-credentials exchange, the Basic pair for an
//! Atlassian API token, and the expiry cache. This module only maps a stored
//! row's non-secret `config` and its opened secret onto the grant.

// JSON: a credential's `config` is per-kind JSONB; each field is read by name
// and refused when absent.
use serde_json::Value;
use systemprompt_security::connector_grant::MachineGrant;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

use super::MachineKind;

const GITHUB_API: &str = "https://api.github.com";

// Why: plain HTTP is allowed only to a loopback host, which is how the test
// workspace stands in for GitHub and Salesforce.
#[must_use]
pub fn endpoint_allowed(value: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(value) else {
        return false;
    };
    let loopback = matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
    let scheme_ok = url.scheme() == "https" || (url.scheme() == "http" && loopback);
    scheme_ok
        && url.username().is_empty()
        && url.password().is_none()
        && url.query().is_none()
        && url.fragment().is_none()
}

fn field(config: &Value, name: &str) -> AdminResult<String> {
    config
        .get(name)
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| AdminError::Unavailable(format!("Machine credential lacks {name}")))
}

fn origin(config: &Value, name: &str, default: Option<&str>) -> AdminResult<String> {
    let value = match (config.get(name).and_then(Value::as_str), default) {
        (Some(value), _) => value.to_owned(),
        (None, Some(default)) => default.to_owned(),
        (None, None) => return field(config, name),
    };
    if !endpoint_allowed(&value) {
        return Err(AdminError::Unavailable(format!("{name} is not allowed")));
    }
    Ok(value.trim_end_matches('/').to_owned())
}

pub(super) fn grant(
    kind: MachineKind,
    config: &Value,
    secret: String,
) -> AdminResult<MachineGrant> {
    Ok(match kind {
        MachineKind::GithubApp => MachineGrant::AppInstallation {
            token_url: format!(
                "{}/app/installations/{}/access_tokens",
                origin(config, "api_url", Some(GITHUB_API))?,
                field(config, "installation_id")?
            ),
            app_id: field(config, "app_id")?,
            private_key_pem: secret,
        },
        MachineKind::SalesforceClientCredentials => MachineGrant::ClientCredentials {
            token_url: format!(
                "{}/services/oauth2/token",
                origin(config, "instance_url", None)?
            ),
            client_id: field(config, "consumer_key")?,
            client_secret: secret,
            scopes: Vec::new(),
        },
        MachineKind::AtlassianApiToken => MachineGrant::ApiToken {
            email: field(config, "email")?,
            api_token: secret,
        },
    })
}
