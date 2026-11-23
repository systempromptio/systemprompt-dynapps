//! One-shot exchange codes for the bridge device-link flow.
//!
//! Only the hash is stored, and codes are short-lived: the code travels through
//! a browser redirect, so a leaked log line must not stay usable. The window is
//! ten minutes rather than the two a machine round-trip needs, because the
//! browserless path asks a person to read the code off a screen and type it
//! into a terminal.
//!
//! A device-link code may also carry the person's UI session, written into
//! the same `ui_*` columns core's `issue_device_link_code` writes, so core's
//! `POST /v1/auth/bridge/session-pat` answers it with a `ui_code`. The caller
//! checks the request first; this only stores what was approved.

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::identifiers::{ClientId, UserId};

use super::error::Result;

const EXCHANGE_CODE_BYTES: usize = 32;
pub const EXCHANGE_CODE_TTL_SECONDS: i64 = 600;

#[derive(Debug)]
pub struct IssuedExchangeCode {
    pub code: String,
    pub expires_at: DateTime<Utc>,
}

pub async fn issue_exchange_code(pool: &PgPool, user_id: &UserId) -> Result<IssuedExchangeCode> {
    let code = generate_code();
    let code_hash = hash_code(&code);
    let expires_at = Utc::now() + ChronoDuration::seconds(EXCHANGE_CODE_TTL_SECONDS);

    sqlx::query!(
        "INSERT INTO bridge_exchange_codes (code_hash, user_id, expires_at) VALUES ($1, $2, $3)",
        code_hash,
        user_id.as_str(),
        expires_at,
    )
    .execute(pool)
    .await?;

    Ok(IssuedExchangeCode { code, expires_at })
}

/// The UI session a device link was approved for: a declared `person_ui`
/// client, its registered redirect, the scope, the S256 challenge whose
/// verifier stays with the client, and when the person signed in to the
/// approving browser.
#[derive(Debug, Clone, Copy)]
pub struct UiSessionGrant<'a> {
    pub client_id: &'a ClientId,
    pub redirect_uri: &'a str,
    pub scope: &'a str,
    pub code_challenge: &'a str,
    pub authenticated_at: DateTime<Utc>,
}

pub async fn issue_device_link_code(
    pool: &PgPool,
    user_id: &UserId,
    grant: UiSessionGrant<'_>,
) -> Result<IssuedExchangeCode> {
    let code = generate_code();
    let code_hash = hash_code(&code);
    let expires_at = Utc::now() + ChronoDuration::seconds(EXCHANGE_CODE_TTL_SECONDS);

    sqlx::query!(
        r#"
        INSERT INTO bridge_exchange_codes
            (code_hash, user_id, expires_at, ui_client_id, ui_redirect_uri, ui_scope,
             ui_code_challenge, ui_authenticated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
        code_hash,
        user_id.as_str(),
        expires_at,
        grant.client_id.as_str(),
        grant.redirect_uri,
        grant.scope,
        grant.code_challenge,
        grant.authenticated_at,
    )
    .execute(pool)
    .await?;

    Ok(IssuedExchangeCode { code, expires_at })
}

fn generate_code() -> String {
    let mut raw = [0u8; EXCHANGE_CODE_BYTES];
    rand::rng().fill_bytes(&mut raw);
    hex::encode(raw)
}

fn hash_code(code: &str) -> String {
    hex::encode(Sha256::digest(code.as_bytes()))
}
