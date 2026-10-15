//! The Dynapps portal connectors end to end against the fake portal
//! (`dynapps_test_common::FakePortal`), with `dynapps-odoo` and `dynbrain`
//! pointed at it:
//!
//! - a person connects: the real `start` and `callback` handlers run RFC 9728 /
//!   8414 discovery, RFC 7591 registration, PKCE S256, the code exchange and
//!   MCP verification, and store a grant bound to the portal account;
//! - a gateway MCP call works with that person's token: `tools/list` and
//!   `tools/call` at `/mcp/<odoo path>`;
//! - an inactive account is refused, at consent and at the broker;
//! - an expired token is refreshed: the portal sees a `refresh_token` grant and
//!   the next MCP call carries the new bearer.
//!
//! Seam: core's `/api/v1/mcp/dynapps-odoo/mcp` route is not built here -- it
//! needs core's whole API runtime (`AppContext`, governance, the proxy). Its
//! two halves are driven instead: the broker request core's
//! `fetch_external_bearer` makes (same path and headers, through the real
//! `connector_api_router`), then the forwarded MCP exchange with the bearer
//! the broker returned. Per-server ACL (`mcp_server/dynapps-odoo` in
//! `rules.yaml`) is enforced by core before that proxy and is out of reach;
//! "unentitled" here is the broker's own refusal of an inactive account.

use axum::http::StatusCode;
use dynapps_test_common::{DYNBRAIN_PATH, ODOO_PATH};
use systemprompt_web_admin_platform::connector_oauth::{self as oauth, Provider};
use systemprompt_web_admin_platform::repositories::connectors::{
    connector_accounts as accounts, connector_credentials as credentials,
};

use crate::portal_harness::{Harness, Person};

async fn stored_grant(h: &Harness, person: &Person, provider: &str) -> oauth::Grant {
    let provider = Provider::try_from(provider.to_owned()).expect("provider");
    let mut tx = h.db.pool.begin().await.expect("tx");
    let row = credentials::lock(&mut tx, &person.id, provider.slug())
        .await
        .expect("lock grant")
        .expect("grant stored");
    tx.commit().await.expect("commit");
    oauth::open(&row.grant, &person.id, &provider).expect("open grant")
}

#[tokio::test(flavor = "multi_thread")]
async fn a_person_connects_and_the_gateway_call_carries_their_portal_bearer() {
    let Some(h) = Harness::create_or_skip().await else {
        return;
    };
    let alice = h.person("portal-alice").await;
    let done = h.connect(&alice, "dynapps-odoo", "portal-alice").await;
    assert_eq!(done, "/admin/connectors#connector-dynapps-odoo");

    let account = accounts::list_accounts(&h.db.pool, &alice.id)
        .await
        .expect("accounts")
        .into_iter()
        .find(|a| a.provider == "dynapps-odoo")
        .expect("account row");
    assert_eq!(account.status, "connected");
    let grant = stored_grant(&h, &alice, "dynapps-odoo").await;
    assert_eq!(
        h.env.portal.subject_of(&grant.access_token).as_deref(),
        Some("portal-alice")
    );
    assert_eq!(h.env.portal.token_grant_types(), ["authorization_code"]);

    let (status, bearer) = h.broker_token(&alice.jwt, "dynapps-odoo", None).await;
    assert_eq!(status, StatusCode::OK);
    let bearer = bearer.expect("access_token");
    assert_eq!(bearer, grant.access_token);
    let (tools, whoami) = h.mcp_whoami(ODOO_PATH, &bearer).await;
    assert!(tools.contains(&"whoami".to_owned()), "{tools:?}");
    assert_eq!(whoami, "portal-alice@dynapps-odoo");
    let subjects = h.env.portal.subjects_seen(ODOO_PATH);
    assert!(!subjects.is_empty());
    assert!(subjects.iter().all(|s| s == "portal-alice"), "{subjects:?}");
    assert!(
        h.env.portal.bearers_seen(DYNBRAIN_PATH).is_empty(),
        "a dynapps-odoo connect never touches DynBrain"
    );
    h.db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn each_person_spends_only_their_own_portal_account() {
    let Some(h) = Harness::create_or_skip().await else {
        return;
    };
    let alice = h.person("portal-alice").await;
    let bob = h.person("portal-bob").await;
    h.connect(&alice, "dynbrain", "portal-alice").await;
    h.connect(&bob, "dynbrain", "portal-bob").await;
    for (person, subject) in [(&alice, "portal-alice"), (&bob, "portal-bob")] {
        let (status, bearer) = h.broker_token(&person.jwt, "dynbrain", None).await;
        assert_eq!(status, StatusCode::OK);
        let (_, whoami) = h.mcp_whoami(DYNBRAIN_PATH, &bearer.expect("bearer")).await;
        assert_eq!(whoami, format!("{subject}@dynbrain"));
    }
    h.db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn an_inactive_account_is_refused_at_consent_and_at_the_broker() {
    let Some(h) = Harness::create_or_skip().await else {
        return;
    };
    let carol = h.person("portal-carol").await;
    h.connect(&carol, "dynapps-odoo", "portal-carol").await;
    sqlx::query("UPDATE users SET status = 'inactive' WHERE id = $1")
        .bind(carol.id.as_str())
        .execute(h.db.pool.as_ref())
        .await
        .expect("deactivate");
    let before = h.env.portal.requests().len();

    let (status, location, _) = h
        .get("/connectors/dynapps-odoo/start", &carol.jwt, &[])
        .await;
    // An inactive account no longer has a live session, so consent refuses it
    // as unauthenticated before any entitlement check.
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(location.is_none(), "no redirect to the portal");
    let (status, bearer) = h.broker_token(&carol.jwt, "dynapps-odoo", None).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(bearer.is_none(), "no credential material is returned");
    assert_eq!(
        h.env.portal.requests().len(),
        before,
        "a refusal never reaches the portal"
    );
    h.db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_person_with_no_grant_gets_no_bearer() {
    let Some(h) = Harness::create_or_skip().await else {
        return;
    };
    let dave = h.person("portal-dave").await;
    let (status, bearer) = h.broker_token(&dave.jwt, "dynapps-odoo", None).await;
    assert!(!status.is_success(), "{status}");
    assert!(bearer.is_none());
    h.db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn an_expired_token_is_refreshed_and_the_new_bearer_is_forwarded() {
    let Some(h) = Harness::create_or_skip().await else {
        return;
    };
    let erin = h.person("portal-erin").await;
    h.connect(&erin, "dynapps-odoo", "portal-erin").await;
    let mut grant = stored_grant(&h, &erin, "dynapps-odoo").await;
    let old = grant.access_token.clone();
    grant.expires_at = 0;
    let mut tx = h.db.pool.begin().await.expect("tx");
    credentials::store(
        &mut tx,
        &erin.id,
        "dynapps-odoo",
        &oauth::seal(&grant).expect("seal"),
    )
    .await
    .expect("store expired grant");
    tx.commit().await.expect("commit");
    h.env.portal.expire(&old);

    let stale = h
        .http
        .post(h.env.portal.endpoint(ODOO_PATH))
        .bearer_auth(&old)
        .json(&serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}))
        .send()
        .await
        .expect("stale call");
    assert_eq!(stale.status(), StatusCode::UNAUTHORIZED);
    let challenge = stale.headers()["www-authenticate"]
        .to_str()
        .expect("challenge");
    assert!(
        challenge.contains(&format!(
            "resource_metadata=\"{}/.well-known/oauth-protected-resource{ODOO_PATH}\"",
            h.env.portal.base_url()
        )),
        "{challenge}"
    );

    let (status, bearer) = h.broker_token(&erin.jwt, "dynapps-odoo", None).await;
    assert_eq!(status, StatusCode::OK);
    let fresh = bearer.expect("refreshed bearer");
    assert_ne!(fresh, old, "the broker never hands out the expired token");
    assert_eq!(
        h.env.portal.token_grant_types().last().map(String::as_str),
        Some("refresh_token")
    );
    assert_eq!(
        h.env.portal.subject_of(&fresh).as_deref(),
        Some("portal-erin")
    );
    let (_, whoami) = h.mcp_whoami(ODOO_PATH, &fresh).await;
    assert_eq!(whoami, "portal-erin@dynapps-odoo");
    assert_eq!(
        h.env.portal.bearers_seen(ODOO_PATH).last(),
        Some(&fresh),
        "the forwarded call carried the refreshed bearer"
    );
    let rotated = stored_grant(&h, &erin, "dynapps-odoo").await;
    assert_eq!(rotated.access_token, fresh);
    assert!(rotated.expires_at > chrono::Utc::now().timestamp());
    h.db.cleanup().await;
}
