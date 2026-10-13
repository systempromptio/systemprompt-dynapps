//! Turning a held machine credential into a provider `Authorization` value.
//!
//! The broker returns the complete header value and the MCP server declares an
//! empty `scheme`, so Basic (Atlassian) and Bearer tokens share one shape.

use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
// JSON: a credential's `config` is per-kind JSONB; each field is read by name
// and refused when absent.
use serde_json::Value;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

use super::MachineKind;

const GITHUB_API: &str = "https://api.github.com";
// Why: GitHub refuses an App JWT whose `exp` is more than 10 minutes past
// `iat`; `iat` is back-dated a minute against clock drift.
const APP_JWT_SECS: i64 = 540;
// Why: a Salesforce client-credentials response carries no lifetime; the
// session lasts at least the org's 15-minute minimum timeout.
const SALESFORCE_TOKEN_SECS: i64 = 600;
// Why: an Atlassian API token does not expire on a schedule we can read, so
// the cache only saves the decrypt and re-reads after this long.
const STATIC_TOKEN_SECS: i64 = 3600;

pub(super) struct Minted {
    pub header_value: String,
    pub expires_at: DateTime<Utc>,
}

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

fn field<'a>(config: &'a Value, name: &str) -> AdminResult<&'a str> {
    config
        .get(name)
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AdminError::Unavailable(format!("Machine credential lacks {name}")))
}

fn client() -> AdminResult<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(AdminError::internal)
}

pub(super) async fn mint(kind: MachineKind, config: &Value, secret: &str) -> AdminResult<Minted> {
    match kind {
        MachineKind::GithubApp => github_installation(config, secret).await,
        MachineKind::SalesforceClientCredentials => salesforce(config, secret).await,
        MachineKind::AtlassianApiToken => Ok(Minted {
            header_value: format!(
                "Basic {}",
                STANDARD.encode(format!("{}:{secret}", field(config, "email")?))
            ),
            expires_at: Utc::now() + chrono::Duration::seconds(STATIC_TOKEN_SECS),
        }),
    }
}

#[derive(Serialize)]
struct AppClaims<'a> {
    iss: &'a str,
    iat: i64,
    exp: i64,
}

#[derive(Deserialize)]
struct InstallationToken {
    token: String,
    expires_at: DateTime<Utc>,
}

async fn github_installation(config: &Value, private_key: &str) -> AdminResult<Minted> {
    let app_id = field(config, "app_id")?;
    let installation = field(config, "installation_id")?;
    let api = config
        .get("api_url")
        .and_then(Value::as_str)
        .unwrap_or(GITHUB_API);
    if !endpoint_allowed(api) {
        return Err(AdminError::Unavailable(
            "GitHub API URL is not allowed".into(),
        ));
    }
    let now = Utc::now().timestamp();
    let key = EncodingKey::from_rsa_pem(private_key.as_bytes()).map_err(|_redacted| {
        AdminError::Unavailable("GitHub App private key is unreadable".into())
    })?;
    let jwt = encode(
        &Header::new(Algorithm::RS256),
        &AppClaims {
            iss: app_id,
            iat: now - 60,
            exp: now + APP_JWT_SECS,
        },
        &key,
    )
    .map_err(AdminError::internal)?;
    let url = format!(
        "{}/app/installations/{installation}/access_tokens",
        api.trim_end_matches('/')
    );
    let response = client()?
        .post(url)
        .bearer_auth(jwt)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "Dynapps-systemprompt")
        .send()
        .await
        .map_err(|_redacted| AdminError::Upstream("GitHub token exchange unreachable".into()))?;
    if !response.status().is_success() {
        return Err(AdminError::Upstream(format!(
            "GitHub token exchange returned {}",
            response.status().as_u16()
        )));
    }
    let body: InstallationToken = response
        .json()
        .await
        .map_err(|_redacted| AdminError::Upstream("GitHub token response unreadable".into()))?;
    Ok(Minted {
        header_value: format!("Bearer {}", body.token),
        expires_at: body.expires_at,
    })
}

#[derive(Deserialize)]
struct SalesforceToken {
    access_token: String,
    #[serde(default)]
    expires_in: Option<i64>,
}

async fn salesforce(config: &Value, client_secret: &str) -> AdminResult<Minted> {
    let instance = field(config, "instance_url")?;
    if !endpoint_allowed(instance) {
        return Err(AdminError::Unavailable(
            "Salesforce instance URL is not allowed".into(),
        ));
    }
    let form = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("grant_type", "client_credentials")
        .append_pair("client_id", field(config, "consumer_key")?)
        .append_pair("client_secret", client_secret)
        .finish();
    let response = client()?
        .post(format!(
            "{}/services/oauth2/token",
            instance.trim_end_matches('/')
        ))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(form)
        .send()
        .await
        .map_err(|_redacted| {
            AdminError::Upstream("Salesforce token exchange unreachable".into())
        })?;
    if !response.status().is_success() {
        return Err(AdminError::Upstream(format!(
            "Salesforce token exchange returned {}",
            response.status().as_u16()
        )));
    }
    let body: SalesforceToken = response
        .json()
        .await
        .map_err(|_redacted| AdminError::Upstream("Salesforce token response unreadable".into()))?;
    let lifetime = body
        .expires_in
        .unwrap_or(SALESFORCE_TOKEN_SECS)
        .clamp(60, 86_400);
    Ok(Minted {
        header_value: format!("Bearer {}", body.access_token),
        expires_at: Utc::now() + chrono::Duration::seconds(lifetime),
    })
}
