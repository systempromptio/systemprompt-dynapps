//! Entra ID SAML responses that are wrong on the wire — signature, signer,
//! audience, lifetime, destination, replay — each refused with a redirect to
//! the login page and an audit row, never a session.
//!
//! One provider config per process (first-write-wins slot): nextest's
//! process-per-test isolation is required.

use chrono::{Duration, Utc};
use dynapps_test_common::{SamlIdpFixture, SamlResponseSpec, Tamper};

use crate::fixtures::unique;
use crate::sso_harness::saml::{self, DOMAIN};
use crate::sso_harness::{SAML_ACS, SsoApp, federated_user, login_events};
use crate::tempdb::TempDb;

// Starts a flow, lets `mutate` spoil the response, posts it, and asserts an
// `invalid_assertion` refusal with a matching audit row and no link.
async fn refused_on_the_wire(mutate: impl FnOnce(&mut SamlResponseSpec)) {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let idp = SamlIdpFixture::default();
    let app = SsoApp::build(&db.pool, Some(saml::config(&idp)), None);
    let started = saml::start_default(&app).await;
    let email = format!("{}@{DOMAIN}", unique("wire"));
    let mut spec = saml::response_for(&started, &email);
    mutate(&mut spec);

    saml::post_back(&app, &idp, &started, &spec)
        .await
        .assert_refused("invalid_assertion");
    assert!(
        federated_user(&db.pool, &spec.name_id).await.is_none(),
        "no identity linked"
    );
    assert_eq!(
        login_events(&db.pool).await,
        vec![("refused".to_owned(), Some("invalid_assertion".to_owned()))]
    );
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn saml_refuses_a_tampered_assertion() {
    refused_on_the_wire(|s| s.tamper = Tamper::BreakSignature).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn saml_refuses_an_assertion_signed_by_another_key() {
    refused_on_the_wire(|s| s.tamper = Tamper::SignWithOtherKey).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn saml_refuses_the_wrong_audience() {
    refused_on_the_wire(|s| s.audience = "https://someone-else.test/saml/metadata".to_owned())
        .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn saml_refuses_an_expired_assertion() {
    refused_on_the_wire(|s| {
        s.issue_instant = Utc::now() - Duration::hours(1);
        s.not_on_or_after = Utc::now() - Duration::minutes(30);
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn saml_refuses_the_wrong_destination() {
    refused_on_the_wire(|s| {
        s.destination = "https://elsewhere.test/admin/auth/adfs/acs".to_owned();
    })
    .await;
}

// A replayed POST never reaches the assertion replay guard: its RelayState
// was consumed by the first, so the return is uncorrelated, and the saml
// crate refuses a response carrying `InResponseTo` with no request to match
// (`UnsolicitedNotAllowed` -> `correlation_lost`). The guard still records
// the first sight. The same assertion under a fresh `start` fails its
// `InResponseTo` check instead.
#[tokio::test(flavor = "multi_thread")]
async fn saml_refuses_a_replayed_response() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let idp = SamlIdpFixture::default();
    let app = SsoApp::build(&db.pool, Some(saml::config(&idp)), None);
    let started = saml::start_default(&app).await;
    let spec = saml::response_for(&started, &format!("{}@{DOMAIN}", unique("replay")));
    let response = idp.issue_response(&spec);
    let body = saml::acs_body(&response, Some(&started.relay_state));

    app.post_form(SAML_ACS, &body, Some(&started.binder))
        .await
        .assert_signed_in("/admin");
    app.post_form(SAML_ACS, &body, Some(&started.binder))
        .await
        .assert_refused("correlation_lost");
    let fresh = saml::start_default(&app).await;
    let rebound = saml::acs_body(&response, Some(&fresh.relay_state));
    app.post_form(SAML_ACS, &rebound, Some(&fresh.binder))
        .await
        .assert_refused("invalid_assertion");

    let guarded: i64 =
        sqlx::query_scalar("SELECT count(*) FROM sso_replay_guard WHERE scope = 'sso:entra-saml'")
            .fetch_one(&*db.pool)
            .await
            .expect("read sso_replay_guard");
    assert_eq!(
        guarded, 1,
        "the first sight of the assertion id is recorded"
    );
    let events = login_events(&db.pool).await;
    let outcomes: Vec<&str> = events.iter().map(|(o, _)| o.as_str()).collect();
    assert_eq!(outcomes, ["success", "refused", "refused"], "{events:?}");
    assert_eq!(
        federated_user_count(&db.pool).await,
        1,
        "one link, not three"
    );
    db.cleanup().await;
}

async fn federated_user_count(pool: &sqlx::PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM federated_identities")
        .fetch_one(pool)
        .await
        .expect("count federated_identities")
}
