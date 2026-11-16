//! The Entra ID token returned beside every Microsoft 365 access token: who
//! signed in, and in which tenant.
//!
//! An exchange records the tenant, the user's object id and username on the
//! grant. A refresh that returns an ID token must name the same tenant and
//! user, or the grant is dropped; one that returns none keeps the recorded
//! identity, whose tenant `entra::reconcile` re-checks against the allow-list
//! every time the grant is opened. The issuer must be the authority of the
//! token endpoint for exactly that tenant, and the audience the instance's
//! client id, so an ID token for another tenant or another app is refused. A
//! refused token wipes the access and refresh tokens from the grant before the
//! failure is returned, so nothing of it can be stored.

use super::Grant;
use super::entra_tenancy::{SHARED_TENANT, tenant_id};
use super::failure::{FailureKind, FlowFailure, FlowResult};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use serde::Deserialize;
use std::time::Instant;
use systemprompt_web_admin_core::connections::report::VerificationReport;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use url::Url;

const CLOCK_SKEW_SECONDS: i64 = 300;

#[derive(Deserialize)]
struct Claims {
    iss: String,
    aud: String,
    tid: String,
    exp: i64,
    #[serde(default)]
    oid: Option<String>,
    #[serde(default)]
    sub: Option<String>,
    #[serde(default)]
    preferred_username: Option<String>,
    #[serde(default)]
    email: Option<String>,
}

/// The Microsoft 365 account an ID token vouches for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntraIdentity {
    pub tenant: String,
    pub account: String,
    pub label: String,
}

/// Why an ID token was refused; the code is what the logs carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntraRefusal {
    Undecodable,
    Tenant,
    Issuer,
    Audience,
    Expired,
    Subject,
    Changed,
    NotAllowed,
}

impl EntraRefusal {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Undecodable => "id_token_undecodable",
            Self::Tenant => "tenant_mismatch",
            Self::Issuer => "issuer_mismatch",
            Self::Audience => "audience_mismatch",
            Self::Expired => "id_token_expired",
            Self::Subject => "subject_missing",
            Self::Changed => "identity_changed",
            Self::NotAllowed => "tenant_not_allowed",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            Self::NotAllowed => "this Microsoft 365 tenant is not allowed on this instance",
            Self::Changed => "the refreshed token names another tenant or user",
            _ => "Microsoft 365 returned an ID token that does not match this connector",
        }
    }
}

// Why: OpenID Connect Core §3.1.3.7 lets a client trust an ID token received
// straight from the token endpoint over TLS instead of checking its signature;
// that endpoint is the instance's pinned Entra authority, so only claims are
// read here.
pub fn identity(
    id_token: &str,
    token_endpoint: &str,
    client: &str,
    now: i64,
) -> Result<EntraIdentity, EntraRefusal> {
    let claims = decode(id_token).ok_or(EntraRefusal::Undecodable)?;
    let endpoint = Url::parse(token_endpoint)
        .ok()
        .ok_or(EntraRefusal::Issuer)?;
    let tenant = tenant_id(&claims.tid).ok_or(EntraRefusal::Tenant)?;
    let segment = endpoint
        .path_segments()
        .and_then(|mut segments| segments.next())
        .ok_or(EntraRefusal::Tenant)?;
    if segment != SHARED_TENANT && segment != tenant {
        return Err(EntraRefusal::Tenant);
    }
    let issuer = format!("{}/{tenant}/v2.0", endpoint.origin().ascii_serialization());
    if !claims.iss.eq_ignore_ascii_case(&issuer) {
        return Err(EntraRefusal::Issuer);
    }
    if !claims.aud.eq_ignore_ascii_case(client) {
        return Err(EntraRefusal::Audience);
    }
    if claims.exp.saturating_add(CLOCK_SKEW_SECONDS) < now {
        return Err(EntraRefusal::Expired);
    }
    let account = claims
        .oid
        .or(claims.sub)
        .filter(|value| !value.trim().is_empty())
        .ok_or(EntraRefusal::Subject)?;
    let label = claims
        .preferred_username
        .or(claims.email)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| account.clone());
    Ok(EntraIdentity {
        tenant,
        account,
        label,
    })
}

fn decode(id_token: &str) -> Option<Claims> {
    let mut parts = id_token.split('.');
    let payload = parts.nth(1)?;
    if parts.count() != 1 {
        return None;
    }
    let bytes = URL_SAFE_NO_PAD.decode(payload.trim_end_matches('=')).ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub(super) fn admit(grant: &mut Grant, id_token: Option<&str>, refreshing: bool) -> FlowResult<()> {
    let recorded = !grant.resource_id.is_empty() && !grant.account_id.is_empty();
    let Some(id_token) = id_token else {
        return if refreshing && recorded {
            Ok(())
        } else {
            Err(refuse(grant, refreshing, EntraRefusal::Undecodable))
        };
    };
    let now = Utc::now().timestamp();
    let checked = identity(id_token, &grant.token_endpoint, &grant.client, now);
    let identity = checked.map_err(|refusal| refuse(grant, refreshing, refusal))?;
    let changed = identity.tenant != grant.resource_id || identity.account != grant.account_id;
    if refreshing && changed {
        return Err(refuse(grant, refreshing, EntraRefusal::Changed));
    }
    grant.resource_id = identity.tenant;
    grant.account_id = identity.account;
    grant.account_name = identity.label;
    super::entra::GRAPH_RESOURCE.clone_into(&mut grant.resource_name);
    Ok(())
}

pub(super) fn refuse(grant: &mut Grant, refreshing: bool, refusal: EntraRefusal) -> FlowFailure {
    tracing::warn!(target: "connector_oauth", provider = grant.provider.slug(), user_id = %grant.user,
        stage = "token", reason = refusal.code(), refreshing, "Entra ID token refused");
    wipe(grant);
    FlowFailure {
        kind: if refreshing {
            FailureKind::GrantRejected
        } else {
            FailureKind::Permission
        },
        error: Box::new(AdminError::Forbidden(format!(
            "Microsoft 365 sign-in refused: {}",
            refusal.message()
        ))),
    }
}

pub(super) fn wipe(grant: &mut Grant) {
    grant.access_token.clear();
    grant.refresh_token = None;
    grant.expires_at = 0;
}

pub(super) fn verify_reporting(grant: &Grant, report: &mut VerificationReport) -> AdminResult<()> {
    let started = Instant::now();
    let outcome = if grant.account_id.is_empty() || grant.resource_id.is_empty() {
        Err(AdminError::Upstream(
            "Microsoft 365 returned no verified account".into(),
        ))
    } else {
        Ok(format!(
            "{} · tenant {}",
            grant.account_name, grant.resource_id
        ))
    };
    report.record("identity", started, &outcome);
    outcome.map(|_| ())
}
