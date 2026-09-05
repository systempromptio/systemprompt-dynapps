//! Loading one tab's rows and shaping them for the template.
//!
//! Each loader returns its rows and the total behind them, and a read that
//! fails degrades to an empty tab rather than an error page: the KPI strip
//! above it comes from different statements and is still worth showing, and a
//! fleet page that refuses to render because one of four tables was slow is
//! worse than one that says a tab is empty.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::repositories::devices::certs::{self, FleetCertRow};
use crate::repositories::devices::links::{self, PendingLinkRow};
use crate::repositories::devices::pats::{self, CredentialQuery, FleetApiKeyRow};
use crate::repositories::devices::sessions::{self, BridgeSessionRow, SessionQuery};

use super::context::{ApiKeyRowView, BridgeSessionRowView, CertRowView, LinkRowView};

pub(super) fn stamp(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M").to_string()
}

pub(super) fn maybe_stamp(value: Option<DateTime<Utc>>) -> String {
    value.map_or_else(|| "—".to_owned(), stamp)
}

fn user_url(user_id: &systemprompt::identifiers::UserId) -> String {
    format!("/admin/users/{}", urlencoding::encode(user_id.as_str()))
}

pub(super) async fn load_sessions(
    pool: &PgPool,
    query: SessionQuery<'_>,
) -> (Vec<BridgeSessionRowView>, i64) {
    let (rows, total) = sessions::list_bridge_sessions_paged(pool, query)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "bridge session listing failed"))
        .unwrap_or_default();
    (rows.iter().map(session_row).collect(), total)
}

fn session_row(row: &BridgeSessionRow) -> BridgeSessionRowView {
    BridgeSessionRowView {
        hostname: row.hostname.clone(),
        os: row.os.clone(),
        version: row.bridge_version.clone(),
        user_name: row.user_name.clone(),
        user_email: row.user_email.clone(),
        user_url: user_url(&row.user_id),
        started_display: stamp(row.started_at),
        heartbeat_display: stamp(row.last_heartbeat_at),
        forwarded_display: crate::handlers::ssr::format::short_num(row.forwarded_total),
        tokens_display: crate::handlers::ssr::format::format_token_total(
            row.tokens_in_total + row.tokens_out_total,
        ),
        status_label: if row.is_stale { "Stale" } else { "Active" },
        status_tone: if row.is_stale { "warn" } else { "ok" },
    }
}

pub(super) async fn load_pats(
    pool: &PgPool,
    query: CredentialQuery<'_>,
    can_manage: bool,
) -> (Vec<ApiKeyRowView>, i64) {
    let (rows, total) = pats::list_api_keys_paged(pool, query)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access token listing failed"))
        .unwrap_or_default();
    (rows.iter().map(|r| pat_row(r, can_manage)).collect(), total)
}

fn pat_row(row: &FleetApiKeyRow, can_manage: bool) -> ApiKeyRowView {
    let expired = row.expires_at.is_some_and(|at| at < Utc::now());
    let (status_label, status_tone) = credential_state(row.revoked_at.is_some(), expired);
    ApiKeyRowView {
        id: row.id.clone(),
        name: row.name.clone(),
        key_prefix: row.key_prefix.clone(),
        user_name: row.user_name.clone(),
        user_url: user_url(&row.user_id),
        created_display: stamp(row.created_at),
        used_display: maybe_stamp(row.last_used_at),
        expires_display: row.expires_at.map_or_else(|| "Never".to_owned(), stamp),
        status_label,
        status_tone,
        can_revoke: can_manage && row.revoked_at.is_none(),
    }
}

// Why: revoked outranks expired. A revoked credential that later passes its
// own expiry is still revoked, and labelling it "Expired" would read as
// having lapsed on its own rather than having been taken away.
const fn credential_state(revoked: bool, expired: bool) -> (&'static str, &'static str) {
    if revoked {
        ("Revoked", "muted")
    } else if expired {
        ("Expired", "warn")
    } else {
        ("Active", "ok")
    }
}

pub(super) async fn load_certs(
    pool: &PgPool,
    query: CredentialQuery<'_>,
    can_manage: bool,
) -> (Vec<CertRowView>, i64) {
    let (rows, total) = certs::list_device_certs_paged(pool, query)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "device certificate listing failed"))
        .unwrap_or_default();
    (
        rows.iter().map(|r| cert_row(r, can_manage)).collect(),
        total,
    )
}

fn cert_row(row: &FleetCertRow, can_manage: bool) -> CertRowView {
    let (status_label, status_tone) = credential_state(row.revoked_at.is_some(), false);
    CertRowView {
        id: row.id.clone(),
        label: row.label.clone(),
        fingerprint: row.fingerprint.clone(),
        user_name: row.user_name.clone(),
        user_url: user_url(&row.user_id),
        enrolled_display: stamp(row.enrolled_at),
        status_label,
        status_tone,
        can_revoke: can_manage && row.revoked_at.is_none(),
    }
}

pub(super) async fn load_links(
    pool: &PgPool,
    sort: &str,
    dir: &str,
    limit: i64,
    offset: i64,
) -> (Vec<LinkRowView>, i64) {
    let (rows, total) = links::list_pending_links_paged(pool, sort, dir, limit, offset)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "pending link listing failed"))
        .unwrap_or_default();
    (rows.iter().map(link_row).collect(), total)
}

fn link_row(row: &PendingLinkRow) -> LinkRowView {
    LinkRowView {
        user_name: row.user_name.clone(),
        user_email: row.user_email.clone(),
        user_url: user_url(&row.user_id),
        created_display: stamp(row.created_at),
        expires_display: stamp(row.expires_at),
        status_label: if row.is_expired { "Expired" } else { "Waiting" },
        status_tone: if row.is_expired { "warn" } else { "accent" },
    }
}
