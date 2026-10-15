//! Entra ID `OpenID` Connect sign-in end to end through core's SSO driver,
//! against the loopback fake `IdP`: discovery, a real `start` (state, nonce,
//! PKCE), code redemption with the client secret from the secrets store, and
//! ID-token validation against the fake's JWKS.
//!
//! The provider config lives in a process-wide first-write-wins slot, so
//! these tests depend on nextest running each test in its own process.

use dynapps_test_common::FakeOidcIdp;

use crate::fixtures::unique;
use crate::sso_harness::oidc::{DOMAIN, USERS_GROUP};
use crate::sso_harness::{
    BINDER, OIDC_CALLBACK, OIDC_TENANT, SsoApp, directory_groups, federated_user, login_events,
    map_directory_group, oidc,
};
use crate::tempdb::TempDb;

#[tokio::test(flavor = "multi_thread")]
async fn oidc_sign_in_provisions_links_audits_and_maps_groups() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let idp = FakeOidcIdp::start(OIDC_TENANT).await;
    let app = SsoApp::build(&db.pool, Some(oidc::config(&idp)));
    map_directory_group(&db.pool, "dyn-users", USERS_GROUP).await;

    let started = oidc::start(&app, "/admin/auth/entra/start?redirect=/admin/profile").await;
    assert!(
        started.redirect.location.starts_with(idp.base_url()),
        "start redirects to the IdP's authorize endpoint: {:?}",
        started.redirect
    );
    for marker in [
        "code_challenge_method=S256",
        &format!("client_id={}", oidc::CLIENT_ID),
    ] {
        assert!(
            started.redirect.location.contains(marker),
            "authorize URL lacks {marker}"
        );
    }
    let binder = started
        .redirect
        .set_cookies
        .iter()
        .find(|c| c.starts_with(&format!("{BINDER}=")))
        .expect("binder cookie");
    assert!(
        binder.contains(&format!("Path={OIDC_CALLBACK}")),
        "{binder}"
    );

    let email = format!("{}@{DOMAIN}", unique("oidc"));
    let sub = unique("pairwise-sub");
    let reply = oidc::call_back(
        &app,
        &idp,
        &started,
        oidc::token_for(&started, &email, &sub),
    )
    .await;
    reply.assert_signed_in("/admin/profile");

    let token_calls: Vec<_> = idp
        .requests()
        .into_iter()
        .filter(|r| r.path == "token")
        .collect();
    assert_eq!(token_calls.len(), 1, "the code is redeemed exactly once");
    assert!(
        token_calls[0].body.contains("code_verifier"),
        "PKCE verifier sent"
    );

    let user_id = federated_user(&db.pool, &sub)
        .await
        .or(federated_user(&db.pool, &format!("oid-{sub}")).await)
        .expect("a federated_identities row links the subject");
    assert_eq!(
        login_events(&db.pool).await,
        vec![("success".to_owned(), None)]
    );
    assert_eq!(
        directory_groups(&db.pool, &user_id).await,
        vec!["dyn-users".to_owned()]
    );
    db.cleanup().await;
}
