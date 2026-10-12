//! The hook token the governance plugin presents, minted the way core's
//! bridge does (`bin/bridge/src/gateway/auth.rs`): PAT → bridge JWT
//! (`/v1/auth/bridge/pat`), JWT → OAuth client
//! (`/v1/auth/bridge/oauth-client`), then a `client_credentials` grant scoped
//! `hook:govern hook:track` for one plugin.

use serde::Deserialize;
use systemprompt::identifiers::{ClientId, PluginId};

use crate::error::{FarmError, FarmResult};

#[derive(Debug, Deserialize)]
struct BridgeAuth {
    token: String,
}

#[derive(Debug, Deserialize)]
struct OAuthClient {
    client_id: ClientId,
    client_secret: String,
    token_endpoint: String,
}

#[derive(Debug, Deserialize)]
struct HookToken {
    access_token: String,
}

pub async fn mint_hook_token(
    http: &reqwest::Client,
    internal_base: &str,
    pat: &str,
    gateway_session_id: &str,
    plugin_id: &PluginId,
) -> FarmResult<String> {
    let base = internal_base.trim_end_matches('/');
    let bridge: BridgeAuth = decode(
        http.post(format!("{base}/v1/auth/bridge/pat"))
            .bearer_auth(pat)
            .header("content-type", "application/json")
            .header("x-session-id", gateway_session_id)
            .body("{}"),
        "bridge pat",
    )
    .await?;
    let client: OAuthClient = decode(
        http.post(format!("{base}/v1/auth/bridge/oauth-client"))
            .bearer_auth(&bridge.token)
            .header("content-type", "application/json")
            .body("{}"),
        "oauth client",
    )
    .await?;
    let form = [
        ("grant_type", "client_credentials"),
        ("client_id", client.client_id.as_str()),
        ("client_secret", client.client_secret.as_str()),
        ("scope", "hook:govern hook:track"),
        ("audience", "hook"),
        ("plugin_id", plugin_id.as_str()),
    ];
    let token: HookToken = decode(
        http.post(resolve(base, &client.token_endpoint))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(encode_form(&form)),
        "hook token",
    )
    .await?;
    Ok(token.access_token)
}

fn encode_form(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(key, value)| format!("{}={}", percent_encode(key), percent_encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(byte));
        } else if byte == b' ' {
            out.push('+');
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn resolve(base: &str, endpoint: &str) -> String {
    if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        endpoint.to_owned()
    } else {
        format!("{base}/{}", endpoint.trim_start_matches('/'))
    }
}

async fn decode<T: serde::de::DeserializeOwned>(
    request: reqwest::RequestBuilder,
    step: &str,
) -> FarmResult<T> {
    let resp = request
        .send()
        .await
        .map_err(|source| FarmError::GatewayHttp {
            step: step.to_owned(),
            source,
        })?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp
            .text()
            .await
            .unwrap_or_else(|err| format!("<unreadable body: {err}>"));
        return Err(FarmError::Gateway(format!(
            "{step}: HTTP {status}: {}",
            body.chars().take(300).collect::<String>()
        )));
    }
    resp.json::<T>()
        .await
        .map_err(|source| FarmError::GatewayHttp {
            step: format!("{step} decode"),
            source,
        })
}
