//! The attach form: one kind per provider, non-secret identifiers kept in
//! `config`, the secret sealed and never echoed back.

use serde::Deserialize;
use serde_json::{Value, json};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

use super::grant::endpoint_allowed;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MachineKind {
    GithubApp,
    OdooApiKey,
}

impl MachineKind {
    pub const ALL: [Self; 2] = [Self::GithubApp, Self::OdooApiKey];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GithubApp => "github_app",
            Self::OdooApiKey => "odoo_api_key",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::GithubApp => "GitHub App installation",
            Self::OdooApiKey => "Odoo API key",
        }
    }

    pub fn parse(raw: &str) -> AdminResult<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.as_str() == raw)
            .ok_or_else(|| AdminError::BadRequest("Unknown machine credential kind".into()))
    }
}

/// Every field a kind might need; `validated` keeps only that kind's.
#[derive(Deserialize)]
pub struct AttachCredential {
    pub kind: MachineKind,
    #[serde(default)]
    pub app_id: String,
    #[serde(default)]
    pub installation_id: String,
    #[serde(default)]
    pub api_url: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub db: String,
    #[serde(default)]
    pub login: String,
    #[serde(default)]
    pub secret: String,
}

impl std::fmt::Debug for AttachCredential {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AttachCredential")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

fn plain(value: &str, what: &str) -> AdminResult<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > 512 || value.chars().any(char::is_control) {
        return Err(AdminError::BadRequest(format!("{what} is required")));
    }
    Ok(value.to_owned())
}

fn digits(value: &str, what: &str) -> AdminResult<String> {
    let value = plain(value, what)?;
    if !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err(AdminError::BadRequest(format!("{what} must be numeric")));
    }
    Ok(value)
}

fn endpoint(value: &str, what: &str) -> AdminResult<String> {
    let value = plain(value, what)?.trim_end_matches('/').to_owned();
    if !endpoint_allowed(&value) {
        return Err(AdminError::BadRequest(format!(
            "{what} must be an HTTPS origin"
        )));
    }
    Ok(value)
}

impl AttachCredential {
    pub fn validated(self) -> AdminResult<(MachineKind, Value, String)> {
        if self.secret.trim().is_empty() || self.secret.len() > 16384 || self.secret.contains('\0')
        {
            return Err(AdminError::BadRequest("The secret is required".into()));
        }
        let config = match self.kind {
            MachineKind::GithubApp => {
                let mut config = json!({
                    "app_id": digits(&self.app_id, "App ID")?,
                    "installation_id": digits(&self.installation_id, "Installation ID")?,
                });
                if !self.api_url.trim().is_empty() {
                    config["api_url"] = Value::String(endpoint(&self.api_url, "API URL")?);
                }
                config
            },
            MachineKind::OdooApiKey => json!({
                "url": endpoint(&self.url, "Odoo URL")?,
                "db": plain(&self.db, "Database")?,
                "login": plain(&self.login, "Login")?,
            }),
        };
        Ok((self.kind, config, self.secret))
    }
}
