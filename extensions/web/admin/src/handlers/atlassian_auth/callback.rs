//! `GET /admin/auth/atlassian/callback` — exchange the consent code, capture
//! the accessible site, and bank the credential for the signed-in user.

use axum::Extension;
use axum::extract::Query;
use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use systemprompt::identifiers::UserId;

use super::{AtlassianDeps, STATE_COOKIE, read_state_cookie, secure_flag};
use crate::authz;
use crate::handlers::users::extract_user_from_cookie;
use crate::repositories::secrets::secret_crypto;
use crate::repositories::secrets::secret_keys::get_or_create_user_dek;
use crate::repositories::users::atlassian_credentials::{self, NewCredential, SealedTokens};
use crate::services::atlassian_oauth::{self, AtlassianTokenResponse};

#[derive(Deserialize)]
pub(crate) struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

// Why: every exit from the callback funnels through here — the state cookie is
// cleared on success and failure alike, so a stale one cannot outlive the flow.
fn done(redirect_to: &str, outcome: &str) -> Response {
    // Why: lint-ok: http-error — the consent flow's only exit: a redirect back
    // to the page the user came from, carrying the outcome, not an HTTP error.
    let sep = if redirect_to.contains('?') { '&' } else { '?' };
    let clear = format!(
        "{STATE_COOKIE}=; Path=/admin/auth/atlassian; HttpOnly; SameSite=Lax; Max-Age=0{}",
        secure_flag()
    );
    let mut headers = HeaderMap::new();
    if let Ok(val) = clear.parse() {
        headers.insert(SET_COOKIE, val);
    }
    (
        headers,
        Redirect::to(&format!("{redirect_to}{sep}atlassian={outcome}")),
    )
        .into_response()
}

pub(crate) async fn atlassian_callback(
    Extension(deps): Extension<AtlassianDeps>,
    headers: HeaderMap,
    Query(params): Query<CallbackParams>,
) -> Response {
    // Why: lint-ok: http-error — same redirect-based failure channel as
    // `atlassian_start`; every exit from here returns the browser to the page
    // it came from carrying an outcome.
    let Some((expected_state, verifier, redirect_to)) = read_state_cookie(&headers) else {
        return done("/admin/profile", "expired");
    };

    if params.error.is_some() {
        return done(&redirect_to, "denied");
    }

    // Why: this is a link, not a login. The credential attaches to whoever is
    // already signed in via ADFS; without a session there is no user to attach
    // it to, and creating one here would make Atlassian a second front door.
    let Ok(session) = extract_user_from_cookie(&headers) else {
        return done(&redirect_to, "signin_required");
    };

    let (Some(code), Some(state)) = (params.code, params.state) else {
        return done(&redirect_to, "invalid");
    };
    // Why: constant-time-ness is not the concern here (the nonce is single-use
    // and short-lived); a plain mismatch means CSRF or a stale tab.
    if state != expected_state {
        return done(&redirect_to, "invalid");
    }

    let tokens = match atlassian_oauth::exchange_code(&deps.config, &code, &verifier).await {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(error = %e, user_id = %session.user_id, "Atlassian code exchange failed");
            return done(&redirect_to, "exchange_failed");
        },
    };

    // Why: no refresh token means `offline_access` was not granted, and the
    // credential would expire within the hour with no way to renew it. Refuse
    // the link rather than bank something that breaks silently tomorrow.
    let Some(refresh_token) = tokens.refresh_token.as_deref() else {
        tracing::warn!(
            user_id = %session.user_id,
            "Atlassian returned no refresh token; offline_access was not granted"
        );
        return done(&redirect_to, "no_offline_access");
    };

    // Why: one call at link time to learn which site the user consented to. A
    // failure here is not fatal — the hosted MCP server resolves the site from
    // the token itself, so bank the credential and leave cloud_id null.
    let resource = match atlassian_oauth::accessible_resources(&tokens.access_token).await {
        Ok(list) => list.into_iter().next(),
        Err(e) => {
            tracing::warn!(error = %e, "Atlassian accessible-resources lookup failed");
            None
        },
    };

    let sealed = match seal_for_user(&deps, &session.user_id, refresh_token, &tokens).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = %e, user_id = %session.user_id, "Atlassian credential seal failed");
            return done(&redirect_to, "storage_failed");
        },
    };

    // Why: `read:me` is requested so the account id is available for support
    // ("which Atlassian account is this?"), but a missing one must not fail the
    // link — the cloud id already identifies the site.
    let account_id = resource.as_ref().map_or("", |r| r.id.as_str());

    if let Err(e) = atlassian_credentials::upsert_credential(
        &deps.write_pool,
        &NewCredential {
            user_id: &session.user_id,
            account_id,
            tokens: &sealed,
            cloud_id: resource.as_ref().map(|r| r.id.as_str()),
            cloud_url: resource.as_ref().map(|r| r.url.as_str()),
            granted_scopes: &tokens.scope,
        },
    )
    .await
    {
        tracing::error!(error = %e, user_id = %session.user_id, "Atlassian credential store failed");
        return done(&redirect_to, "storage_failed");
    }

    authz::atlassian::invalidate(&session.user_id).await;
    tracing::info!(user_id = %session.user_id, "Atlassian account linked");
    done(&redirect_to, "linked")
}

// Why: split out so the callback reads as a sequence of decisions rather than a
// nest of matches. Both failures here are internal (no master key, no DEK, a
// seal that will not encrypt) and collapse to one outcome for the user.
async fn seal_for_user(
    deps: &AtlassianDeps,
    user_id: &UserId,
    refresh_token: &str,
    tokens: &AtlassianTokenResponse,
) -> Result<SealedTokens, super::AtlassianError> {
    let master = secret_crypto::load_master_key()?;
    let dek = get_or_create_user_dek(&deps.write_pool, user_id, &master).await?;
    Ok(atlassian_oauth::seal_tokens(
        &dek,
        refresh_token,
        &tokens.access_token,
        tokens.expires_in,
    )?)
}
