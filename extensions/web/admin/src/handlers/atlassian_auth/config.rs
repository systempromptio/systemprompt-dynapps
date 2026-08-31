//! Atlassian OAuth 2.0 (3LO) connection config, loaded from
//! `services/web/config/atlassian.yaml`.

use serde::Deserialize;

// Why: env var first, then the encrypted secrets store — the same priority
// chain as `secret_crypto::load_master_key`. The secret is never written into
// `atlassian.yaml`, which is a tracked file.
pub(crate) fn client_secret() -> Option<String> {
    std::env::var("ATLASSIAN_CLIENT_SECRET")
        .ok()
        .or_else(|| {
            systemprompt::config::SecretsBootstrap::get()
                .ok()
                .and_then(|s| s.get("atlassian_client_secret").cloned())
        })
        // Why: the shipped placeholder is a non-empty string, so an unconfigured
        // install would otherwise look configured and fail at the token endpoint
        // instead of reporting itself unavailable.
        .filter(|t| !t.is_empty() && !t.starts_with("REPLACE_WITH"))
}

// Why: `offline_access` is what makes the whole design possible — without it
// Atlassian returns no refresh token and every link decays into a one-hour
// session. The rest is the minimum for a BA agent that files epics and issues
// and a developer agent that reads Confluence.
//
// These are CLASSIC scopes. An app registered with granular scopes rejects
// them outright, so the app registration and this list must agree; see
// services/web/config/atlassian.yaml.
pub(super) fn default_scopes() -> String {
    [
        "offline_access",
        "read:me",
        "read:jira-work",
        "write:jira-work",
        "read:jira-user",
        "manage:jira-project",
        "read:confluence-space.summary",
        "read:confluence-content.all",
        "write:confluence-content",
        "search:confluence",
    ]
    .join(" ")
}

/// Atlassian OAuth connection config.
///
/// The client *secret* is never stored here: it is read at token-exchange time
/// from `ATLASSIAN_CLIENT_SECRET` or the encrypted secret store.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AtlassianConfig {
    #[serde(default)]
    pub enabled: bool,
    // Why: named `consumer_key`, not `client_id`, because a field suffixed
    // `_id` reads as a typed identifier in this codebase and this is an opaque
    // app credential. The YAML key stays `client_id`, which is what Atlassian
    // calls it.
    #[serde(rename = "client_id")]
    pub consumer_key: String,
    pub redirect_uri: String,
    #[serde(default = "default_scopes")]
    pub scopes: String,
}

impl AtlassianConfig {
    // Why: the placeholder used when no `atlassian.yaml` is present, so the
    // routes still register and report "unavailable" cleanly rather than 404ing
    // and looking like a deployment bug.
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            consumer_key: String::new(),
            redirect_uri: String::new(),
            scopes: default_scopes(),
        }
    }

    // Why: `enabled` alone is not enough — a half-filled YAML would sail past it
    // and fail at Atlassian with an opaque error.
    #[must_use]
    pub const fn is_usable(&self) -> bool {
        self.enabled && !self.consumer_key.is_empty() && !self.redirect_uri.is_empty()
    }
}

// Why: Atlassian's authorization server is a fixed hosted endpoint, not a
// per-tenant domain like Salesforce's My Domain — there is nothing to configure
// and nothing a deployment can get wrong, so these are constants rather than
// config fields.
pub(crate) const AUTHORIZE_URL: &str = "https://auth.atlassian.com/authorize";
pub(crate) const TOKEN_URL: &str = "https://auth.atlassian.com/oauth/token";
pub(crate) const ACCESSIBLE_RESOURCES_URL: &str =
    "https://api.atlassian.com/oauth/token/accessible-resources";
pub(crate) const API_AUDIENCE: &str = "api.atlassian.com";
