//! A person's linked Odoo account, one row per user, and the credential audit
//! trail shared with service-account machine credentials.
//!
//! Only `find_odoo_user_link` returns the sealed key; the console reads
//! `find_odoo_link_summary`, which never selects it.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug)]
pub struct NewOdooUserLink<'a> {
    pub user: &'a UserId,
    pub odoo_url: &'a str,
    pub odoo_db: &'a str,
    pub odoo_login: &'a str,
    pub odoo_uid: i64,
    pub api_key_ciphertext: Vec<u8>,
    pub api_key_nonce: Vec<u8>,
}

#[derive(Debug)]
pub struct OdooUserLinkRow {
    pub odoo_url: String,
    pub odoo_db: String,
    pub odoo_login: String,
    pub odoo_uid: i64,
    pub api_key_ciphertext: Vec<u8>,
    pub api_key_nonce: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct OdooLinkSummary {
    pub odoo_url: String,
    pub odoo_login: String,
    pub odoo_uid: i64,
    pub linked_at: DateTime<Utc>,
    pub verified_at: DateTime<Utc>,
}

// Why: Postgres `ON CONFLICT` makes a re-link a rotation in one statement; it
// keeps `linked_at` and restamps `verified_at`. The return says which it was.
pub async fn set_odoo_user_link(
    pool: &PgPool,
    link: &NewOdooUserLink<'_>,
) -> Result<bool, sqlx::Error> {
    let inserted = sqlx::query_scalar!(
        "INSERT INTO odoo_user_links (user_id, odoo_url, odoo_db, odoo_login, odoo_uid, \
         api_key_ciphertext, api_key_nonce) VALUES ($1, $2, $3, $4, $5, $6, $7) \
         ON CONFLICT (user_id) DO UPDATE SET odoo_url = EXCLUDED.odoo_url, \
         odoo_db = EXCLUDED.odoo_db, odoo_login = EXCLUDED.odoo_login, \
         odoo_uid = EXCLUDED.odoo_uid, api_key_ciphertext = EXCLUDED.api_key_ciphertext, \
         api_key_nonce = EXCLUDED.api_key_nonce, verified_at = NOW() \
         RETURNING (xmax = 0) AS \"inserted!\"",
        link.user.as_str(),
        link.odoo_url,
        link.odoo_db,
        link.odoo_login,
        link.odoo_uid,
        &link.api_key_ciphertext,
        &link.api_key_nonce,
    )
    .fetch_one(pool)
    .await?;
    Ok(inserted)
}

pub async fn find_odoo_user_link(
    pool: &PgPool,
    user: &UserId,
) -> Result<Option<OdooUserLinkRow>, sqlx::Error> {
    sqlx::query_as!(
        OdooUserLinkRow,
        "SELECT odoo_url, odoo_db, odoo_login, odoo_uid, api_key_ciphertext, api_key_nonce \
         FROM odoo_user_links WHERE user_id = $1",
        user.as_str(),
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_odoo_link_summary(
    pool: &PgPool,
    user: &UserId,
) -> Result<Option<OdooLinkSummary>, sqlx::Error> {
    sqlx::query_as!(
        OdooLinkSummary,
        "SELECT odoo_url, odoo_login, odoo_uid, linked_at, verified_at \
         FROM odoo_user_links WHERE user_id = $1",
        user.as_str(),
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_odoo_user_link(pool: &PgPool, user: &UserId) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM odoo_user_links WHERE user_id = $1",
        user.as_str()
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

#[derive(Debug, Clone, Copy)]
pub struct CredentialAudit<'a> {
    pub subject: &'a UserId,
    pub provider: &'a str,
    pub action: &'a str,
    pub actor: &'a UserId,
}

pub async fn insert_credential_audit(
    pool: &PgPool,
    audit: CredentialAudit<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO connector_credential_audit (subject_user_id, provider, action, \
         actor_user_id) VALUES ($1, $2, $3, $4)",
        audit.subject.as_str(),
        audit.provider,
        audit.action,
        audit.actor.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(())
}
