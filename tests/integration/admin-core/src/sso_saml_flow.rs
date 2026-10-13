//! Entra ID SAML sign-in end to end through core's SSO driver: a real
//! `start`, a response signed by the fake `IdP` answering the `AuthnRequest`
//! the SP actually issued, and the ACS post a browser would make.
//!
//! The provider config lives in a process-wide first-write-wins slot, so
//! these tests depend on nextest running each test in its own process; run
//! under plain `cargo test` they would share the first test's config.

use dynapps_test_common::SamlIdpFixture;

use crate::fixtures::unique;
use crate::sso_harness::saml::{self, DOMAIN};
use crate::sso_harness::{
    BINDER, SAML_ACS, SsoApp, directory_groups, federated_user, login_events, map_directory_group,
};
use crate::tempdb::TempDb;

#[tokio::test(flavor = "multi_thread")]
async fn saml_sign_in_provisions_links_audits_and_maps_groups() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let idp = SamlIdpFixture::default();
    let app = SsoApp::build(&db.pool, Some(saml::config(&idp)), None);
    map_directory_group(&db.pool, "dyn-users", saml::USERS_GROUP).await;
    map_directory_group(&db.pool, "dyn-unrelated", "Someone-Elses-Group").await;

    let started = saml::start(&app, "/admin/auth/adfs/start?redirect=/admin/profile").await;
    assert!(
        started.redirect.location.starts_with(&idp.sso_url),
        "start redirects to the IdP: {:?}",
        started.redirect
    );
    let binder_cookie = started
        .redirect
        .set_cookies
        .iter()
        .find(|c| c.starts_with(&format!("{BINDER}=")))
        .expect("binder cookie");
    assert!(
        binder_cookie.contains(&format!("Path={SAML_ACS}")),
        "{binder_cookie}"
    );
    assert!(binder_cookie.contains("HttpOnly"), "{binder_cookie}");

    let email = format!("{}@{DOMAIN}", unique("saml"));
    let sub = unique("persistent-sub");
    let mut spec = saml::response_for(&started, &email);
    spec.name_id = sub.clone();
    spec.email = Some(email.clone());

    let reply = saml::post_back(&app, &idp, &started, &spec).await;
    reply.assert_signed_in("/admin/profile");
    assert!(
        reply
            .set_cookies
            .iter()
            .any(|c| c.starts_with(&format!("{BINDER}=;"))
                || c.starts_with(&format!("{BINDER}=")) && c.contains("Max-Age=0")),
        "the binder cookie is cleared: {:?}",
        reply.set_cookies
    );

    let user_id = federated_user(&db.pool, &sub)
        .await
        .expect("a federated_identities row links the persistent NameID");
    let stored_email: String = sqlx::query_scalar("SELECT email FROM users WHERE id = $1")
        .bind(&user_id)
        .fetch_one(&*db.pool)
        .await
        .expect("provisioned user");
    assert_eq!(stored_email, email);
    assert_eq!(
        login_events(&db.pool).await,
        vec![("success".to_owned(), None)],
        "one success audit row"
    );
    assert_eq!(
        directory_groups(&db.pool, &user_id).await,
        vec!["dyn-users".to_owned()],
        "only the mapped directory group becomes an adfs membership"
    );
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn saml_metadata_publishes_the_sp_descriptor() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let idp = SamlIdpFixture::default();
    let app = SsoApp::build(&db.pool, Some(saml::config(&idp)), None);
    let reply = app.get("/saml/metadata", None).await;
    assert_eq!(reply.status.as_u16(), 200, "{reply:?}");
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "the saml crate's IdP role cannot mint an unsolicited (IdP-initiated) response; \
            see tests/common/src/sso/saml_idp.rs module head"]
async fn saml_idp_initiated_sign_in_succeeds_when_allowed() {
    let spec = dynapps_test_common::SamlResponseSpec::new(
        &saml::acs_url(),
        &saml::entity_id(),
        None,
        "unsolicited@sso.test",
    );
    SamlIdpFixture::issue_unsolicited_response(&spec)
        .expect("fixture can mint an unsolicited response");
}
