//! OAuth token exchange and refresh, using only bound destinations.
use super::failure::{FailureKind, FlowFailure, FlowResult};
use super::token_response::{Tokens, accepted_tokens};
use super::transport::client;
use super::{Grant, Provider};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use serde::Deserialize;
use systemprompt_web_admin_core::error::AdminResult;

const SALESFORCE_OPAQUE_TOKEN_SECONDS: i64 = 900;

#[derive(Deserialize)]
struct JwtExpiry {
    exp: i64,
}

fn unverified_jwt_exp(token: &str) -> Option<i64> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    serde_json::from_slice::<JwtExpiry>(&bytes)
        .ok()
        .map(|claims| claims.exp)
}

async fn request_with_client(
    grant: &mut Grant,
    fields: &[(&str, &str)],
    http: &reqwest::Client,
) -> FlowResult<()> {
    super::generic::validate_grant(grant).map_err(|error| FlowFailure {
        kind: FailureKind::Configuration,
        error,
    })?;
    let body = {
        let mut form = url::form_urlencoded::Serializer::new(String::new());
        form.extend_pairs(fields)
            .append_pair("client_id", &grant.client);
        if !grant.client_secret.is_empty()
            && matches!(grant.token_auth_method.as_str(), "" | "client_secret_post")
        {
            form.append_pair("client_secret", &grant.client_secret);
        }
        if grant.provider == Provider::Atlassian || matches!(grant.provider, Provider::Generic(_)) {
            form.append_pair("resource", &grant.provider.endpoint());
        }
        form.finish()
    };
    let mut request = http.post(&grant.token_endpoint);
    if grant.token_auth_method == "client_secret_basic" {
        let encode = |value: &str| {
            url::form_urlencoded::byte_serialize(value.as_bytes()).collect::<String>()
        };
        request = request.basic_auth(encode(&grant.client), Some(encode(&grant.client_secret)));
    }
    let provider = grant.provider.slug();
    let refreshing = fields
        .first()
        .is_some_and(|(_, value)| *value == "refresh_token");
    let response = request
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|error| {
            tracing::warn!(
                target: "connector_oauth",
                provider,
                user_id = %grant.user,
                stage = "token",
                grant_type = fields.first().map_or("", |(_, v)| v),
                timeout = error.is_timeout(),
                connect = error.is_connect(),
                "token request failed before a response"
            );
            FlowFailure::new(if refreshing && !error.is_connect() {
                FailureKind::RefreshUncertain
            } else {
                FailureKind::Outage
            })
        })?;
    let tokens = accepted_tokens(grant, response, refreshing).await?;
    apply_tokens(grant, tokens, refreshing)
}


fn apply_tokens(grant: &mut Grant, tokens: Tokens, refreshing: bool) -> FlowResult<()> {
    // Why: Salesforce rotation has already occurred before response validation.
    // Save the replacement even when the access token or its expiry is invalid.
    let replacement_received = tokens.refresh_token.is_some();
    let invalid = if refreshing && !replacement_received {
        FailureKind::RefreshUncertain
    } else {
        FailureKind::InvalidResponse
    };
    if let Some(refresh) = tokens.refresh_token {
        if refresh.trim().is_empty() {
            return Err(token_validation_failure(
                grant,
                if refreshing {
                    FailureKind::RefreshUncertain
                } else {
                    FailureKind::InvalidResponse
                },
                "empty_refresh_token",
            ));
        }
        grant.refresh_token = Some(refresh);
    }
    grant.expires_at = 0;
    if tokens
        .token_type
        .as_deref()
        .is_some_and(|kind| !kind.eq_ignore_ascii_case("bearer"))
        || (matches!(grant.provider, Provider::Generic(_)) && tokens.token_type.is_none())
    {
        return Err(token_validation_failure(
            grant,
            invalid,
            "unexpected_token_type",
        ));
    }
    grant.access_token = tokens
        .access_token
        .filter(|token| !token.trim().is_empty())
        .ok_or_else(|| token_validation_failure(grant, invalid, "missing_access_token"))?;
    let seconds = match tokens.expires_in {
        Some(seconds) if seconds > 0 && seconds <= 31_536_000 => Some(seconds),
        Some(_) => None,
        None => undeclared_expiry_seconds(&grant.provider, &grant.access_token),
    }
    .ok_or_else(|| token_validation_failure(grant, invalid, "invalid_expiry"))?;
    grant.expires_at = if seconds == 0 && matches!(grant.provider, Provider::Generic(_)) {
        i64::MAX
    } else {
        Utc::now().timestamp().saturating_add(seconds)
    };
    if grant.provider.is_salesforce()
        && let Some(expiry) = unverified_jwt_exp(&grant.access_token)
    {
        grant.expires_at = grant.expires_at.min(expiry);
    }
    if grant.expires_at <= Utc::now().timestamp() {
        return Err(token_validation_failure(
            grant,
            invalid,
            "expired_access_token",
        ));
    }
    if grant.refresh_token.is_none() && !matches!(grant.provider, Provider::Generic(_)) {
        return Err(token_validation_failure(
            grant,
            FailureKind::Permission,
            "missing_refresh_authorization",
        ));
    }
    Ok(())
}

fn token_validation_failure(grant: &Grant, kind: FailureKind, reason: &'static str) -> FlowFailure {
    tracing::warn!(target: "connector_oauth", provider = grant.provider.slug(), user_id = %grant.user,
        stage = "token", reason, classification = kind.code(), "token response validation failed");
    FlowFailure::new(kind)
}

// Why: Salesforce omits `expires_in`; treating its token as expired would
// rotate the refresh token on every broker call and break the MCP proxy's
// session binding after `initialize`, so read the JWT `exp` or assume a short
// life.
fn undeclared_expiry_seconds(provider: &Provider, access_token: &str) -> Option<i64> {
    match provider {
        Provider::Salesforce(_) => Some(
            unverified_jwt_exp(access_token)
                .map_or(SALESFORCE_OPAQUE_TOKEN_SECONDS, |exp| {
                    exp.saturating_sub(Utc::now().timestamp())
                })
                .min(31_536_000),
        ),
        Provider::Generic(_) => Some(0),
        Provider::Atlassian => None,
    }
}

pub async fn exchange(grant: &mut Grant, code: &str) -> AdminResult<()> {
    exchange_classified(grant, code)
        .await
        .map_err(|failure| failure.error)
}

pub(crate) async fn exchange_classified(grant: &mut Grant, code: &str) -> FlowResult<()> {
    let callback = grant
        .provider
        .callback()
        .map_err(FlowFailure::verification)?;
    exchange_request(
        grant,
        code,
        &callback,
        &client().map_err(FlowFailure::verification)?,
    )
    .await
}

pub async fn exchange_with_client(
    grant: &mut Grant,
    code: &str,
    callback: &str,
    http: &reqwest::Client,
) -> AdminResult<()> {
    exchange_request(grant, code, callback, http)
        .await
        .map_err(|failure| failure.error)
}

async fn exchange_request(
    grant: &mut Grant,
    code: &str,
    callback: &str,
    http: &reqwest::Client,
) -> FlowResult<()> {
    let verifier = grant.verifier.clone();
    request_with_client(
        grant,
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", callback),
            ("code_verifier", &verifier),
        ],
        http,
    )
    .await?;
    grant.verifier.clear();
    Ok(())
}

pub(super) async fn refresh(grant: &mut Grant) -> FlowResult<()> {
    refresh_classified(grant, &client().map_err(FlowFailure::verification)?).await
}

pub async fn refresh_with_client(grant: &mut Grant, http: &reqwest::Client) -> AdminResult<()> {
    refresh_classified(grant, http)
        .await
        .map_err(|failure| failure.error)
}

pub(super) async fn refresh_classified(
    grant: &mut Grant,
    http: &reqwest::Client,
) -> FlowResult<()> {
    if matches!(grant.provider, Provider::Generic(_)) {
        super::generic::validate_refresh(grant, http)
            .await
            .map_err(|error| FlowFailure {
                kind: FailureKind::Configuration,
                error,
            })?;
    }
    let token = grant
        .refresh_token
        .clone()
        .filter(|token| !token.trim().is_empty())
        .ok_or_else(|| FlowFailure::new(FailureKind::Permission))?;
    request_with_client(
        grant,
        &[("grant_type", "refresh_token"), ("refresh_token", &token)],
        http,
    )
    .await
}
