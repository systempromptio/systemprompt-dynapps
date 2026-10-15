//! Bounded provider token response decoding and safe diagnostics.
use super::Grant;
use super::failure::{FailureKind, FlowFailure, FlowResult, oauth_failure, safe_oauth_code};
use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct Tokens {
    pub(super) access_token: Option<String>,
    pub(super) refresh_token: Option<String>,
    pub(super) expires_in: Option<i64>,
    pub(super) error: Option<String>,
    pub(super) token_type: Option<String>,
}

// Why: OAuth response diagnostics can contain credentials; return only the
// safe failure classification across this boundary.
pub(super) async fn accepted_tokens(
    grant: &Grant,
    response: reqwest::Response,
    refreshing: bool,
) -> FlowResult<Tokens> {
    let status = response.status().as_u16();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("missing")
        .to_owned();
    tracing::info!(target: "connector_oauth", provider = grant.provider.slug(), user_id = %grant.user,
        stage = "token", status, refreshing, content_type, "token endpoint responded");
    if status >= 500 || status == 429 {
        return Err(FlowFailure::new(FailureKind::Outage));
    }
    // JSON: inspect provider field types without exposing tokens.
    let value: serde_json::Value = super::generic_discovery::bounded_json(response)
        .await
        .map_err(|_untrusted_response_error| {
            FlowFailure::new(if refreshing && (200..300).contains(&status) {
                FailureKind::RefreshUncertain
            } else {
                FailureKind::InvalidResponse
            })
        })?;
    log_shape(grant, &value);
    let tokens: Tokens = serde_json::from_value(value).map_err(|error| {
        tracing::warn!(target: "connector_oauth", provider = grant.provider.slug(),
            stage = "token", category = ?error.classify(), "token response schema mismatch (values omitted)");
        FlowFailure::new(if refreshing && (200..300).contains(&status) {
            FailureKind::RefreshUncertain
        } else { FailureKind::InvalidResponse })
    })?;
    if !(200..300).contains(&status) || tokens.error.is_some() {
        let kind = oauth_failure(status, tokens.error.as_deref());
        let code = safe_oauth_code(tokens.error.as_deref());
        tracing::warn!(target: "connector_oauth", provider = grant.provider.slug(), user_id = %grant.user,
            stage = "token", status, oauth_error = code, classification = kind.code(), "token request rejected");
        return Err(FlowFailure::new(kind));
    }
    Ok(tokens)
}


// JSON: only types of allowlisted token fields are logged at this protocol
// boundary.
fn log_shape(grant: &Grant, value: &serde_json::Value) {
    let shape: Vec<_> = [
        "access_token",
        "refresh_token",
        "token_type",
        "expires_in",
        "error",
        "error_description",
    ]
    .into_iter()
    .map(|field| {
        let kind = match value.get(field) {
            None => "missing",
            Some(serde_json::Value::Null) => "null",
            Some(serde_json::Value::String(_)) => "string",
            Some(serde_json::Value::Number(_)) => "number",
            Some(serde_json::Value::Bool(_)) => "boolean",
            Some(serde_json::Value::Array(_)) => "array",
            Some(serde_json::Value::Object(_)) => "object",
        };
        format!("{field}:{kind}")
    })
    .collect();
    tracing::info!(target: "connector_oauth", provider = grant.provider.slug(), user_id = %grant.user,
        stage = "token", fields = ?shape, "token response field types (values omitted)");
}
