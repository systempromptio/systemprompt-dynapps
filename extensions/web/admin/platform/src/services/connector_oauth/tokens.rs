//! OAuth token exchange and refresh, using only bound destinations.
use super::Grant;
use super::failure::{FailureKind, FlowFailure, FlowResult};
use super::token_response::{Tokens, accepted_tokens};
use super::transport::client;
use chrono::Utc;
use systemprompt::models::net::validate_outbound_url;
use systemprompt_web_admin_core::error::AdminResult;

async fn request_with_client(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    grant: &mut Grant,
    fields: &[(&str, &str)],
    http: &reqwest::Client,
) -> FlowResult<()> {
    super::generic::validate_grant(services, grant).map_err(|error| FlowFailure {
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
        form.append_pair("resource", &grant.provider.endpoint(&services.config()));
        form.finish()
    };
    let token_endpoint = validate_outbound_url(&grant.token_endpoint).map_err(|error| {
        FlowFailure::verification(systemprompt_web_admin_core::error::AdminError::from(error))
    })?;
    let mut request = http.post(token_endpoint);
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
    // Why: a provider that rotates refresh tokens has already rotated before
    // response validation, so save the replacement even when the access token
    // or its expiry is invalid.
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
        || tokens.token_type.is_none()
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
        None => Some(0),
    }
    .ok_or_else(|| token_validation_failure(grant, invalid, "invalid_expiry"))?;
    grant.expires_at = if seconds == 0 {
        i64::MAX
    } else {
        Utc::now().timestamp().saturating_add(seconds)
    };
    if grant.expires_at <= Utc::now().timestamp() {
        return Err(token_validation_failure(
            grant,
            invalid,
            "expired_access_token",
        ));
    }
    Ok(())
}

fn token_validation_failure(grant: &Grant, kind: FailureKind, reason: &'static str) -> FlowFailure {
    tracing::warn!(target: "connector_oauth", provider = grant.provider.slug(), user_id = %grant.user,
        stage = "token", reason, classification = kind.code(), "token response validation failed");
    FlowFailure::new(kind)
}

pub async fn exchange(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    grant: &mut Grant,
    code: &str,
) -> AdminResult<()> {
    exchange_classified(services, grant, code)
        .await
        .map_err(|failure| failure.error)
}

pub(crate) async fn exchange_classified(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    grant: &mut Grant,
    code: &str,
) -> FlowResult<()> {
    let callback = grant
        .provider
        .callback()
        .map_err(FlowFailure::verification)?;
    exchange_request(
        services,
        grant,
        code,
        &callback,
        &client().map_err(FlowFailure::verification)?,
    )
    .await
}

pub async fn exchange_with_client(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    grant: &mut Grant,
    code: &str,
    callback: &str,
    http: &reqwest::Client,
) -> AdminResult<()> {
    exchange_request(services, grant, code, callback, http)
        .await
        .map_err(|failure| failure.error)
}

async fn exchange_request(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    grant: &mut Grant,
    code: &str,
    callback: &str,
    http: &reqwest::Client,
) -> FlowResult<()> {
    let verifier = grant.verifier.clone();
    request_with_client(
        services,
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

pub(super) async fn refresh(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    grant: &mut Grant,
) -> FlowResult<()> {
    refresh_classified(
        services,
        grant,
        &client().map_err(FlowFailure::verification)?,
    )
    .await
}

pub async fn refresh_with_client(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    grant: &mut Grant,
    http: &reqwest::Client,
) -> AdminResult<()> {
    refresh_classified(services, grant, http)
        .await
        .map_err(|failure| failure.error)
}

pub(super) async fn refresh_classified(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    grant: &mut Grant,
    http: &reqwest::Client,
) -> FlowResult<()> {
    super::generic::validate_refresh(services, grant, http)
        .await
        .map_err(|error| FlowFailure {
            kind: FailureKind::Configuration,
            error,
        })?;
    let token = grant
        .refresh_token
        .clone()
        .filter(|token| !token.trim().is_empty())
        .ok_or_else(|| FlowFailure::new(FailureKind::Permission))?;
    request_with_client(
        services,
        grant,
        &[("grant_type", "refresh_token"), ("refresh_token", &token)],
        http,
    )
    .await
}
