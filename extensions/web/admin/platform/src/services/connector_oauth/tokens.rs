//! OAuth token exchange and refresh, using only bound destinations.
use super::failure::{FailureKind, FlowFailure, FlowResult};
use super::token_response::{Tokens, accepted_tokens};
use super::transport::client;
use super::{AuthorizationCode, Grant};
use chrono::Utc;
use systemprompt::models::net::{LoopbackPolicy, validate_outbound_url};
use systemprompt::traits::SecretStore;
use systemprompt_web_admin_core::error::AdminResult;

async fn request_with_client(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    grant: &mut Grant,
    fields: &[(&str, &str)],
    http: &reqwest::Client,
) -> FlowResult<()> {
    super::generic::validate_grant(services, grant).map_err(|error| FlowFailure {
        kind: FailureKind::Configuration,
        error: Box::new(error),
    })?;
    let entra = super::entra::settings(&services.config(), &grant.provider);
    let body = entra.as_ref().map_or_else(
        || generic_form(services, grant, fields),
        |settings| super::entra::token_form(settings, grant, fields),
    );
    let token_endpoint = validate_outbound_url(&grant.token_endpoint, LoopbackPolicy::Allow)
        .map_err(|error| {
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
    let mut tokens = accepted_tokens(grant, response, refreshing).await?;
    let id_token = tokens.id_token.take();
    apply_tokens(grant, tokens, refreshing)?;
    if entra.is_some() {
        super::entra_claims::admit(grant, id_token.as_deref(), refreshing)?;
    }
    Ok(())
}

fn generic_form(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    grant: &Grant,
    fields: &[(&str, &str)],
) -> String {
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
    config: &systemprompt::manifest::Config,
    grant: &mut Grant,
    code: &str,
) -> AdminResult<()> {
    let callback = grant.provider.callback(config);
    let code = AuthorizationCode {
        code,
        callback: &callback,
    };
    exchange_request(services, None, grant, code, &client()?)
        .await
        .map_err(|failure| *failure.error)
}

pub(crate) async fn exchange_classified(
    access: super::ConnectorAccess<'_>,
    config: &systemprompt::manifest::Config,
    grant: &mut Grant,
    code: &str,
) -> FlowResult<()> {
    let callback = grant.provider.callback(config);
    exchange_request(
        access.services,
        Some(access.secrets),
        grant,
        AuthorizationCode {
            code,
            callback: &callback,
        },
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
    let code = AuthorizationCode { code, callback };
    exchange_request(services, None, grant, code, http)
        .await
        .map_err(|failure| *failure.error)
}

pub(super) async fn exchange_request(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    secrets: Option<&dyn SecretStore>,
    grant: &mut Grant,
    code: AuthorizationCode<'_>,
    http: &reqwest::Client,
) -> FlowResult<()> {
    let admission = if super::entra::is_entra(&services.config(), &grant.provider) {
        Some(secrets.ok_or_else(super::entra::allow_list_unavailable)?)
    } else {
        None
    };
    let verifier = grant.verifier.clone();
    let fields = [
        ("grant_type", "authorization_code"),
        ("code", code.code),
        ("redirect_uri", code.callback),
        ("code_verifier", verifier.as_str()),
    ];
    let outcome = request_with_client(services, grant, &fields, http).await;
    if outcome.is_err() && admission.is_some() {
        super::entra_claims::wipe(grant);
    }
    outcome?;
    grant.verifier.clear();
    admission.map_or(Ok(()), |secrets| super::entra::admit_tenant(secrets, grant))
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
        .map_err(|failure| *failure.error)
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
            error: Box::new(error),
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
