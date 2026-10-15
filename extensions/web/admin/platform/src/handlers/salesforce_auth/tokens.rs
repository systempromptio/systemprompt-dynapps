//! The Salesforce OAuth token endpoint call the RFC 7523 JWT-bearer grant
//! posts to.

use serde::Deserialize;

use super::SalesforceError;

#[derive(Debug, Deserialize)]
pub(crate) struct SalesforceTokenResponse {
    pub access_token: String,
}

pub(crate) async fn post_token_request(
    token_url: &str,
    body: String,
) -> Result<SalesforceTokenResponse, SalesforceError> {
    let resp = reqwest::Client::new()
        .post(token_url)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp
            .text()
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "salesforce token: error body unreadable"))
            .unwrap_or_default();
        return Err(SalesforceError::TokenEndpoint { status, body });
    }
    Ok(resp.json().await?)
}
