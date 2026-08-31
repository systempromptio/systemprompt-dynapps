//! Minting per-user Atlassian bearers from a banked OAuth 2.0 3LO grant.
//!
//! Atlassian rotates refresh tokens: every refresh returns a new one and
//! invalidates the old. That makes the stored credential the user's whole
//! Atlassian session, and a lost write signs them out. Two defences, both
//! required:
//!
//! 1. The access token is cached in the row until it nears expiry, so the
//!    ordinary tool call is a decrypt rather than a token-endpoint round trip.
//!    A user refreshes about once an hour instead of once per call.
//! 2. The refresh itself runs under `SELECT ... FOR UPDATE` on the credential
//!    row. A single agent turn touching both the Jira and Confluence servers
//!    issues two accessor calls at once — that is the ordinary case, not an
//!    edge case — and without the lock both would spend the same refresh token
//!    and strand one of the results.

use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, Transaction};
use systemprompt::identifiers::UserId;

use crate::handlers::atlassian_auth::{
    ACCESSIBLE_RESOURCES_URL, API_AUDIENCE, AtlassianConfig, AtlassianError, TOKEN_URL,
    client_secret,
};
use crate::repositories::secrets::secret_crypto::{self, SecretCryptoError};
use crate::repositories::secrets::secret_keys::get_or_create_user_dek;
use crate::repositories::users::atlassian_credentials::{self, SealedTokens};

// Why: the lock is held across the network call to Atlassian, so an unbounded
// request would pin a Postgres connection and block every later tool call for
// that user behind it. Ten seconds is well past Atlassian's normal latency and
// well short of anything a user would wait through.
const TOKEN_HTTP_TIMEOUT: Duration = Duration::from_secs(10);

// Why: refresh a little early rather than hand out a token that expires
// mid-flight. The MCP call downstream of us has its own latency budget.
pub const EXPIRY_SKEW_SECS: i64 = 120;

// Why: Atlassian's token endpoint takes JSON, unlike Salesforce's
// form-encoded one — which suits a workspace reqwest built without the `form`
// feature but with `json`.
#[derive(Debug, Serialize)]
struct CodeExchangeRequest<'a> {
    grant_type: &'a str,
    client_id: &'a str,
    client_secret: &'a str,
    code: &'a str,
    redirect_uri: &'a str,
    code_verifier: &'a str,
}

#[derive(Debug, Serialize)]
struct RefreshRequest<'a> {
    grant_type: &'a str,
    client_id: &'a str,
    client_secret: &'a str,
    refresh_token: &'a str,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AtlassianTokenResponse {
    pub access_token: String,
    // Why: only present when `offline_access` was granted. Absent means the app
    // registration or the consent screen dropped the scope, and the grant is
    // useless to us beyond the hour — the callback rejects it rather than
    // banking a credential that will die silently.
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    #[serde(default)]
    pub scope: String,
}

// Why: one accessible Atlassian site, from the accessible-resources endpoint.
#[derive(Debug, Deserialize)]
pub(crate) struct AccessibleResource {
    pub id: String,
    pub url: String,
}

// Why: what the accessor hands back to core.
#[derive(Debug, Clone)]
pub(crate) struct MintedToken {
    pub access_token: String,
    pub cloud_id: Option<String>,
}

fn http_client() -> Result<reqwest::Client, AtlassianError> {
    Ok(reqwest::Client::builder()
        .timeout(TOKEN_HTTP_TIMEOUT)
        .build()?)
}

// Why: unlike Salesforce's form-encoded endpoint, Atlassian's takes JSON —
// which is just as well, since workspace `reqwest` is built without the
// `form` feature but with `json`.
async fn post_token<B: Serialize + Sync>(
    body: &B,
) -> Result<AtlassianTokenResponse, AtlassianError> {
    let resp = http_client()?.post(TOKEN_URL).json(body).send().await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        // Why: `invalid_grant` is the one failure the user can fix, and it is
        // reported as a plain 400 alongside unrelated request errors — the body
        // is the only thing that distinguishes them.
        if status == reqwest::StatusCode::BAD_REQUEST && body.contains("invalid_grant") {
            return Err(AtlassianError::RefreshRejected);
        }
        return Err(AtlassianError::TokenEndpoint { status, body });
    }
    Ok(resp.json().await?)
}

pub(crate) async fn exchange_code(
    cfg: &AtlassianConfig,
    code: &str,
    code_verifier: &str,
) -> Result<AtlassianTokenResponse, AtlassianError> {
    let secret = client_secret().ok_or(AtlassianError::NotConfigured)?;
    post_token(&CodeExchangeRequest {
        grant_type: "authorization_code",
        client_id: &cfg.consumer_key,
        client_secret: &secret,
        code,
        redirect_uri: &cfg.redirect_uri,
        code_verifier,
    })
    .await
}

// Why: called once at link time. The hosted MCP server resolves the site from
// the token's own consent, so this is not on the hot path — but capturing the
// cloud id now is the difference between building a direct REST path in an
// afternoon and re-consenting every user to get it.
pub(crate) async fn accessible_resources(
    access_token: &str,
) -> Result<Vec<AccessibleResource>, AtlassianError> {
    let resp = http_client()?
        .get(ACCESSIBLE_RESOURCES_URL)
        .bearer_auth(access_token)
        .send()
        .await?;
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AtlassianError::TokenEndpoint { status, body });
    }
    Ok(resp.json().await?)
}

// Why: seal both tokens under `dek` and compute the expiry to record with them.
pub(crate) fn seal_tokens(
    dek: &[u8; 32],
    refresh_token: &str,
    access_token: &str,
    expires_in: i64,
) -> Result<SealedTokens, SecretCryptoError> {
    let (refresh_cipher, refresh_nonce) = seal(dek, refresh_token)?;
    let (access_cipher, access_nonce) = seal(dek, access_token)?;
    Ok(SealedTokens {
        refresh_cipher,
        refresh_nonce: refresh_nonce.to_vec(),
        access_cipher,
        access_nonce: access_nonce.to_vec(),
        access_expires_at: Utc::now() + chrono::Duration::seconds(expires_in),
    })
}

// Why: a fresh nonce on every seal. ChaCha20-Poly1305 with a repeated
// (key, nonce) pair is catastrophic, and the rotation path re-seals the same
// logical field over and over — exactly the shape that invites reuse.
fn seal(dek: &[u8; 32], plaintext: &str) -> Result<(Vec<u8>, [u8; 12]), SecretCryptoError> {
    let nonce = secret_crypto::generate_nonce();
    let cipher = secret_crypto::encrypt(dek, &nonce, plaintext.as_bytes())?;
    Ok((cipher, nonce))
}

fn open(dek: &[u8; 32], nonce: &[u8], ciphertext: &[u8]) -> Result<String, SecretCryptoError> {
    let nonce: [u8; 12] = nonce
        .try_into()
        .map_err(|_bad_len| SecretCryptoError::InvalidKeyMaterial)?;
    let plaintext = secret_crypto::decrypt(dek, &nonce, ciphertext)?;
    String::from_utf8(plaintext).map_err(|_not_utf8| SecretCryptoError::InvalidKeyMaterial)
}

pub const fn is_fresh(expires_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> bool {
    match expires_at {
        Some(exp) => exp.timestamp() - now.timestamp() > EXPIRY_SKEW_SECS,
        None => false,
    }
}

// Why: the access token for `user_id`, refreshing it if the cached one is
// stale.
pub(crate) async fn get_access_token(
    pool: &PgPool,
    user_id: &UserId,
    cfg: &AtlassianConfig,
) -> Result<MintedToken, AtlassianError> {
    let master = secret_crypto::load_master_key()?;
    let dek = get_or_create_user_dek(pool, user_id, &master).await?;

    let mut tx = pool.begin().await?;
    let row = atlassian_credentials::find_for_update(&mut tx, user_id)
        .await?
        .ok_or(AtlassianError::NotLinked)?;

    // Why: the common path. A peer that refreshed while we waited for the lock
    // has already written a fresh token, so the waiter does no HTTP at all.
    if is_fresh(row.access_token_expires_at, Utc::now())
        && let (Some(cipher), Some(nonce)) = (
            row.encrypted_access_token.as_deref(),
            row.access_token_nonce.as_deref(),
        )
    {
        let access_token = open(&dek, nonce, cipher)?;
        tx.commit().await?;
        return Ok(MintedToken {
            access_token,
            cloud_id: row.cloud_id,
        });
    }

    let refresh_token = open(&dek, &row.refresh_token_nonce, &row.encrypted_refresh_token)?;
    let fresh = refresh(cfg, &refresh_token).await?;

    // Why: the rotated refresh token must be persisted by the same transaction
    // that spent the old one. A panic between mint and commit rolls back to the
    // old token, which Atlassian still honours inside its reuse window.
    persist(&mut tx, user_id, &dek, &fresh).await?;
    tx.commit().await?;

    Ok(MintedToken {
        access_token: fresh.access_token,
        cloud_id: row.cloud_id,
    })
}

async fn refresh(
    cfg: &AtlassianConfig,
    refresh_token: &str,
) -> Result<AtlassianTokenResponse, AtlassianError> {
    let secret = client_secret().ok_or(AtlassianError::NotConfigured)?;
    post_token(&RefreshRequest {
        grant_type: "refresh_token",
        client_id: &cfg.consumer_key,
        client_secret: &secret,
        refresh_token,
    })
    .await
}

async fn persist(
    tx: &mut Transaction<'_, Postgres>,
    user_id: &UserId,
    dek: &[u8; 32],
    fresh: &AtlassianTokenResponse,
) -> Result<(), AtlassianError> {
    // Why: Atlassian returns a new refresh token on every refresh, but the field
    // is optional in the response shape. If it is somehow absent, keeping the
    // old one is the only safe move — it is still valid inside the reuse window,
    // where writing an empty string would end the session outright.
    let Some(new_refresh) = fresh.refresh_token.as_deref() else {
        tracing::warn!(
            user_id = %user_id,
            "Atlassian refresh returned no new refresh token; keeping the existing one"
        );
        return Ok(());
    };
    let sealed = seal_tokens(dek, new_refresh, &fresh.access_token, fresh.expires_in)?;
    atlassian_credentials::store_refreshed(tx, user_id, &sealed).await?;
    Ok(())
}

// Why: the authorize URL needs the API audience and `prompt=consent`; Atlassian
// omits the refresh token on a silent re-authorization otherwise.
#[must_use]
pub(crate) const fn audience_param() -> &'static str {
    API_AUDIENCE
}
