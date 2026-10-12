//! `POST /api/public/gateway/sessions` with the run PAT: core mints a
//! `user_sessions` row for an API-key caller and returns its id, which every
//! `/v1/messages` call then carries as `x-session-id`.

use serde::Deserialize;
use systemprompt::identifiers::SessionId;

use crate::error::{FarmError, FarmResult};

#[derive(Debug, Deserialize)]
struct MintedSession {
    session_id: SessionId,
}

pub async fn mint_gateway_session(
    http: &reqwest::Client,
    internal_base: &str,
    pat: &str,
) -> FarmResult<String> {
    let url = format!(
        "{}/api/public/gateway/sessions",
        internal_base.trim_end_matches('/')
    );
    let resp = http
        .post(&url)
        .bearer_auth(pat)
        .send()
        .await
        .map_err(|source| FarmError::GatewayHttp {
            step: "gateway session".to_owned(),
            source,
        })?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp
            .text()
            .await
            .unwrap_or_else(|err| format!("<unreadable body: {err}>"));
        return Err(FarmError::Gateway(format!(
            "gateway session: HTTP {status}: {}",
            body.chars().take(300).collect::<String>()
        )));
    }
    resp.json::<MintedSession>()
        .await
        .map(|minted| minted.session_id.as_str().to_owned())
        .map_err(|source| FarmError::GatewayHttp {
            step: "gateway session decode".to_owned(),
            source,
        })
}
