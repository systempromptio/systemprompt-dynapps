//! Entra ID OIDC returns the driver refuses: a token wrong in one way, a
//! state that cannot be correlated, an `IdP` error, a refused client secret,
//! and the identity policy's domain and group rules. Each lands on the login
//! page with its reason and leaves a `refused` audit row.
//!
//! One provider config per process (first-write-wins slot): nextest's
//! process-per-test isolation is required.

use chrono::{Duration, Utc};
use dynapps_test_common::{FakeOidcIdp, IdTokenSpec};
use systemprompt_web_admin_identity::sso::entra_oidc::EntraOidcConfig;

use crate::fixtures::unique;
use crate::sso_harness::profile::OIDC_WRONG_SECRET_KEY;
use crate::sso_harness::saml::DOMAIN;
use crate::sso_harness::{OIDC_TENANT, SsoApp, login_events, oidc};
use crate::tempdb::TempDb;

async fn refused(
    cfg_edit: impl FnOnce(&mut EntraOidcConfig),
    mutate: impl FnOnce(&mut IdTokenSpec),
    reason: &str,
    audit: &str,
) {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let idp = FakeOidcIdp::start(OIDC_TENANT).await;
    let mut cfg = oidc::config(&idp);
    cfg_edit(&mut cfg);
    let app = SsoApp::build(&db.pool, None, Some(cfg));
    let started = oidc::start_default(&app).await;
    let mut spec = oidc::token_for(
        &started,
        &format!("{}@{DOMAIN}", unique("oidc")),
        &unique("sub"),
    );
    mutate(&mut spec);

    oidc::call_back(&app, &idp, &started, spec)
        .await
        .assert_refused(reason);
    assert_eq!(
        login_events(&db.pool).await,
        vec![("refused".to_owned(), Some(audit.to_owned()))]
    );
    db.cleanup().await;
}

async fn invalid(mutate: impl FnOnce(&mut IdTokenSpec)) {
    refused(|_| {}, mutate, "invalid_assertion", "invalid_assertion").await;
}

// The other key's `kid` is not in the tenant's JWKS, so the key lookup —
// not the signature check — is what refuses it.
#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_a_token_signed_by_another_key() {
    invalid(|s| s.sign_with_other_key = true).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_the_wrong_audience() {
    invalid(|s| s.aud = "someone-elses-client".to_owned()).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_an_expired_token() {
    invalid(|s| {
        s.iat = Some(Utc::now() - Duration::hours(2));
        s.exp = Some(Utc::now() - Duration::hours(1));
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_a_nonce_mismatch() {
    invalid(|s| s.nonce = "not-the-nonce-start-issued".to_owned()).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_another_tenant() {
    invalid(|s| s.tid = "99999999-0000-4000-8000-000000000000".to_owned()).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_the_wrong_issuer() {
    invalid(|s| s.wrong_issuer = true).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_an_email_outside_the_allowed_domains() {
    let outsider = format!("{}@not-allowed.test", unique("outsider"));
    refused(
        |_| {},
        |s| {
            s.email = Some(outsider.clone());
            s.preferred_username = Some(outsider);
        },
        "forbidden",
        "forbidden",
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_a_groups_overage() {
    refused(
        |_| {},
        |s| s.groups_overage = true,
        "group_overage",
        "group_overage",
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_a_user_in_no_mapped_group() {
    refused(|_| {}, |s| s.groups.clear(), "no_group", "no_group").await;
}

// The token endpoint answers `401 invalid_client`, which the provider maps to
// an invalid assertion (only a 5xx or an unreachable endpoint is
// `unavailable`).
#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_when_the_idp_rejects_the_client_secret() {
    refused(
        |c| c.client_secret_secret = OIDC_WRONG_SECRET_KEY.to_owned(),
        |_| {},
        "invalid_assertion",
        "invalid_assertion",
    )
    .await;
}
