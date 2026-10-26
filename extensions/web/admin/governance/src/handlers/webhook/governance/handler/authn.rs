//! Bearer-token authentication for the governance webhook.
//!
//! Validates the inbound JWT against the hook/plugin/api audiences and maps
//! failures to a denied [`Decision`] plus a recorded auth-denial audit row.

use axum::http::HeaderMap;
use systemprompt::governance::authz::{Decision, DenyReason};
use systemprompt::identifiers::{ClientId, SessionId, UserId};
use systemprompt::models::auth::JwtAudience;
use systemprompt::oauth::OauthError;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

use crate::handlers::webhook::helpers::extract_bearer_token;

use super::super::types::AuthDenialParams;
use super::response::HookVerdict;
use super::spawn_auth_denial;

pub(super) struct Principal {
    pub user_id: UserId,
    pub client_id: Option<ClientId>,
    pub gateway_session: Option<SessionId>,
}

pub(super) fn deny_for_auth_failure(reason: &str) -> Decision {
    Decision::Deny {
        reason: DenyReason::HookUnavailable {
            policy: "auth_failure".to_owned(),
            detail: reason.to_owned(),
        },
    }
}

// Why: a Claude Code `PreToolUse` hook treats a 401 as "hook unavailable" and
// allows the call, so this `Err` is a 200 deny verdict, never an `AdminError`.
pub(super) fn authenticate_request(
    boot: &ExtensionBoot,
    headers: &HeaderMap,
    denial_params: &AuthDenialParams<'_>,
) -> Result<Principal, Box<HookVerdict>> {
    let Some(token) = extract_bearer_token(headers) else {
        let reason = "Missing Authorization header — tool call blocked";
        spawn_auth_denial(denial_params, reason);
        return Err(Box::new(HookVerdict::new(
            deny_for_auth_failure(reason),
            denial_params.hook_event_name,
        )));
    };

    let jwt_issuer = &boot.config().jwt_issuer;

    let expected_aud = "hook|plugin|api";
    let claims = systemprompt::oauth::validate_jwt_token(
        boot.token_authority().as_ref(),
        token,
        &[
            JwtAudience::Resource("hook".to_owned()),
            JwtAudience::Resource("plugin".to_owned()),
            JwtAudience::Api,
        ],
    )
    .map_err(|e| {
        log_jwt_failure(&e, expected_aud, jwt_issuer);
        let reason = "Invalid or expired token — tool call blocked";
        spawn_auth_denial(denial_params, reason);
        Box::new(HookVerdict::new(
            deny_for_auth_failure(reason),
            denial_params.hook_event_name,
        ))
    })?;

    Ok(Principal {
        user_id: UserId::new(&claims.registered.sub),
        client_id: claims.custom.client_id,
        gateway_session: claims.custom.session_id,
    })
}

fn log_jwt_failure(err: &OauthError, expected_aud: &str, issuer: &str) {
    let (detail, message) = jwt_failure_detail(err);
    tracing::warn!(detail = %detail, expected_aud, issuer, reason = message, "JWT rejected");
}

fn jwt_failure_detail(err: &OauthError) -> (String, &'static str) {
    match err {
        OauthError::TokenAlgMismatch { got, expected } => (
            format!("alg got={got} expected={expected}"),
            "Governance webhook JWT rejected: signing algorithm mismatch",
        ),
        OauthError::TokenMissingKid => (
            "missing kid header".to_owned(),
            "Governance webhook JWT rejected: missing `kid` header",
        ),
        OauthError::TokenUnknownKid { kid } => (
            format!("unknown kid={kid}"),
            "Governance webhook JWT rejected: unknown signing key — token was minted under a \
             different RSA authority",
        ),
        OauthError::Expired(reason) => (
            format!("expired: {reason}"),
            "Governance webhook JWT rejected: token expired",
        ),
        other => (
            format!("{other}"),
            "Governance webhook JWT validation failed",
        ),
    }
}
