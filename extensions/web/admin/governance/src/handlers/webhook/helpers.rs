//! Helpers shared by the webhook handlers.

use axum::http::HeaderMap;

pub(super) fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|v| {
            v.to_str()
                .inspect_err(|e| tracing::warn!(error = %e, "Non-ASCII authorization header"))
                .ok()
        })
        .and_then(|v| v.strip_prefix("Bearer "))
}
