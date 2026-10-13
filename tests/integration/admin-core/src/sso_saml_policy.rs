//! Entra ID SAML sign-ins whose response is genuine but which the driver
//! still refuses: lost correlation, and the identity policy's domain, group
//! and provisioning rules. Each lands on the login page with its own reason
//! and leaves a `refused` audit row.
//!
//! One provider config per process (first-write-wins slot): nextest's
//! process-per-test isolation is required.

use dynapps_test_common::{SamlIdpFixture, SamlResponseSpec};
use systemprompt_web_admin_identity::EntraSamlConfig;

use crate::fixtures::unique;
use crate::sso_harness::saml::{self, DOMAIN};
use crate::sso_harness::{SAML_ACS, SsoApp, federated_user, login_events};
use crate::tempdb::TempDb;

/// Runs one solicited flow under `cfg_edit`, spoils the response with
/// `mutate`, and asserts the refusal `reason` and its audit row.
async fn refused_by_policy(
    cfg_edit: impl FnOnce(&mut EntraSamlConfig),
    mutate: impl FnOnce(&mut SamlResponseSpec),
    reason: &str,
) {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let idp = SamlIdpFixture::default();
    let mut cfg = saml::config(&idp);
    cfg_edit(&mut cfg);
    let app = SsoApp::build(&db.pool, Some(cfg), None);
    let started = saml::start_default(&app).await;
    let mut spec = saml::response_for(&started, &format!("{}@{DOMAIN}", unique("policy")));
    mutate(&mut spec);

    saml::post_back(&app, &idp, &started, &spec)
        .await
        .assert_refused(reason);
    assert!(
        federated_user(&db.pool, &spec.name_id).await.is_none(),
        "no identity linked"
    );
    assert_eq!(
        login_events(&db.pool).await,
        vec![("refused".to_owned(), Some(reason.to_owned()))]
    );
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn saml_refuses_an_email_outside_the_allowed_domains() {
    refused_by_policy(
        |_| {},
        |s| {
            s.name_id = format!("{}@not-allowed.test", unique("outsider"));
            s.email = Some(s.name_id.clone());
        },
        "forbidden",
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn saml_refuses_a_groups_overage() {
    refused_by_policy(
        |_| {},
        |s| {
            s.groups.clear();
            s.groups_overage_link = true;
        },
        "group_overage",
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn saml_refuses_a_user_in_no_mapped_group() {
    refused_by_policy(|_| {}, |s| s.groups.clear(), "no_group").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn saml_refuses_an_unknown_user_when_provisioning_is_off() {
    refused_by_policy(
        |c| c.policy.auto_provision = false,
        |_| {},
        "not_provisioned",
    )
    .await;
}

// With IdP-initiated sign-in off, a real RelayState returned without the
// browser's binder cookie cannot be correlated.
#[tokio::test(flavor = "multi_thread")]
async fn saml_refuses_a_return_without_the_binder_cookie() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let idp = SamlIdpFixture::default();
    let mut cfg = saml::config(&idp);
    cfg.allow_idp_initiated = false;
    let app = SsoApp::build(&db.pool, Some(cfg), None);
    let started = saml::start_default(&app).await;
    let spec = saml::response_for(&started, &format!("{}@{DOMAIN}", unique("unbound")));
    let body = saml::acs_body(&idp.issue_response(&spec), Some(&started.relay_state));

    app.post_form(SAML_ACS, &body, None)
        .await
        .assert_refused("correlation_lost");
    assert_eq!(
        login_events(&db.pool).await,
        vec![("refused".to_owned(), Some("state_lost".to_owned()))]
    );
    db.cleanup().await;
}
