//! Entra ID OIDC callbacks that cannot be tied to a `start` in this browser —
//! no state, a state never issued, a state already consumed — and the `IdP`
//! reporting the user declined. None of them reaches the token endpoint
//! except where noted.
//!
//! One provider config per process (first-write-wins slot): nextest's
//! process-per-test isolation is required.

use dynapps_test_common::FakeOidcIdp;

use crate::fixtures::unique;
use crate::sso_harness::oidc::DOMAIN;
use crate::sso_harness::{OIDC_CALLBACK, OIDC_TENANT, SsoApp, login_events, oidc};
use crate::tempdb::TempDb;

async fn app_and_idp(db: &TempDb) -> (SsoApp, FakeOidcIdp) {
    let idp = FakeOidcIdp::start(OIDC_TENANT).await;
    (SsoApp::build(&db.pool, Some(oidc::config(&idp))), idp)
}

fn token_calls(idp: &FakeOidcIdp) -> usize {
    idp.requests().iter().filter(|r| r.path == "token").count()
}

#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_a_callback_without_state() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let (app, idp) = app_and_idp(&db).await;
    let started = oidc::start_default(&app).await;
    let reply = app
        .get(
            &format!("{OIDC_CALLBACK}?code=anything"),
            Some(&started.binder),
        )
        .await;
    reply.assert_refused("correlation_lost");
    assert_eq!(token_calls(&idp), 0, "no redemption without correlation");
    assert_eq!(
        login_events(&db.pool).await,
        vec![("refused".to_owned(), Some("unsolicited".to_owned()))]
    );
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_a_state_this_instance_never_issued() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let (app, idp) = app_and_idp(&db).await;
    let started = oidc::start_default(&app).await;
    let reply = app
        .get(
            &oidc::callback_uri("anything", "never-issued"),
            Some(&started.binder),
        )
        .await;
    reply.assert_refused("correlation_lost");
    assert_eq!(token_calls(&idp), 0);
    assert_eq!(
        login_events(&db.pool).await,
        vec![("refused".to_owned(), Some("state_lost".to_owned()))]
    );
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_a_replayed_callback() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let (app, idp) = app_and_idp(&db).await;
    let started = oidc::start_default(&app).await;
    let spec = oidc::token_for(
        &started,
        &format!("{}@{DOMAIN}", unique("replay")),
        &unique("sub"),
    );
    let code = idp.issue_code(spec, &started.code_challenge);
    let uri = oidc::callback_uri(&code, &started.state);

    app.get(&uri, Some(&started.binder))
        .await
        .assert_signed_in("/admin");
    app.get(&uri, Some(&started.binder))
        .await
        .assert_refused("correlation_lost");
    assert_eq!(
        token_calls(&idp),
        1,
        "the consumed state stops the replay before redemption"
    );
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_a_callback_with_another_browsers_binder() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let (app, idp) = app_and_idp(&db).await;
    let started = oidc::start_default(&app).await;
    let other = oidc::start_default(&app).await;
    let code = idp.issue_code(
        oidc::token_for(
            &started,
            &format!("{}@{DOMAIN}", unique("swap")),
            &unique("sub"),
        ),
        &started.code_challenge,
    );
    app.get(
        &oidc::callback_uri(&code, &started.state),
        Some(&other.binder),
    )
    .await
    .assert_refused("correlation_lost");
    assert_eq!(token_calls(&idp), 0);
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn oidc_refuses_when_the_user_declines_at_the_idp() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let (app, idp) = app_and_idp(&db).await;
    let started = oidc::start_default(&app).await;
    let uri = format!(
        "{OIDC_CALLBACK}?error=access_denied&error_description=declined&state={}",
        started.state
    );
    app.get(&uri, Some(&started.binder))
        .await
        .assert_refused("invalid_assertion");
    assert_eq!(token_calls(&idp), 0);
    assert_eq!(
        login_events(&db.pool).await,
        vec![("refused".to_owned(), Some("invalid_assertion".to_owned()))]
    );
    db.cleanup().await;
}
