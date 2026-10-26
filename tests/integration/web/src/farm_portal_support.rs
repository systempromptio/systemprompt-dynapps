//! Fake-portal plumbing for the farm `run_as` e2e: a connected
//! `dynapps-odoo` grant per identity (issued by the fake to
//! `portal:<user id>`), the broker request core makes for a farm run's MCP
//! call, and the forwarded `whoami` call.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use dynapps_test_common::fake_portal::env::BROKER_SECRET;
use dynapps_test_common::{FakePortal, ODOO_PATH, PortalEnv, Principal};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use systemprompt::database::Database;
use systemprompt::identifiers::UserId;
use systemprompt::users::UserRepository;
use systemprompt_web_admin_platform::connector_api_router;
use systemprompt_web_admin_platform::connector_oauth::{self as oauth, Grant, Provider};
use systemprompt_web_admin_platform::repositories::connectors::{
    connector_accounts as accounts, connector_credentials as credentials,
};
use systemprompt_web_admin_platform::service_accounts::ServiceAccounts;
use tower::ServiceExt;

pub(crate) const PROVIDER: &str = "dynapps-odoo";

pub(crate) fn portal_account(user: &UserId) -> String {
    format!("portal:{user}")
}

// A connected grant for `user`, issued by the fake to `portal:<user>`.
pub(crate) async fn connect(pool: &PgPool, env: &PortalEnv, user: &UserId) {
    let portal = &env.portal;
    let provider = Provider::try_from(PROVIDER.to_owned()).expect("provider");
    let tokens = portal.issue_tokens(&portal_account(user));
    let grant = Grant {
        user: user.as_str().to_owned(),
        configuration_binding: oauth::generic::binding(&env.services, &provider).expect("binding"),
        authorization_issuer: portal.base_url().to_owned(),
        token_auth_method: "none".into(),
        provider: provider.clone(),
        client: "farm-e2e-client".into(),
        client_secret: String::new(),
        verifier: String::new(),
        access_token: tokens.access_token,
        refresh_token: Some(tokens.refresh_token),
        expires_at: chrono::Utc::now().timestamp() + 3600,
        token_endpoint: format!("{}/token", portal.base_url()),
        generation: 0,
        session: None,
        auth_method: "oauth".into(),
        account_id: String::new(),
        account_name: String::new(),
        resource_id: String::new(),
        resource_name: String::new(),
        authorization_scheme: "Bearer".into(),
    };
    let mut tx = pool.begin().await.expect("tx");
    let mut account = accounts::get_locked_account(&mut tx, user, PROVIDER)
        .await
        .expect("account");
    account.status = "connected".into();
    account.auth_method = Some("oauth".into());
    account.verified_at = Some(chrono::Utc::now());
    credentials::store(
        &mut tx,
        user,
        PROVIDER,
        &oauth::seal(env.boot.secrets().as_ref(), &grant).expect("seal"),
    )
    .await
    .expect("store grant");
    accounts::update_account(&mut tx, user, &account)
        .await
        .expect("update account");
    tx.commit().await.expect("commit");
}

// The broker request core makes for a farm run's MCP call: the credential
// user's headless JWT and the run's `farm:<run id>` purpose.
pub(crate) async fn broker_bearer(
    pool: &Arc<PgPool>,
    env: &PortalEnv,
    user: &UserId,
    caller: &str,
) -> (StatusCode, Option<String>) {
    let database = Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    let router = connector_api_router(
        Arc::clone(pool),
        ServiceAccounts::new(
            UserRepository::new(&database),
            Arc::clone(env.boot.secrets()),
        ),
    )
    .layer(axum::Extension(env.boot.clone()))
    .layer(axum::Extension(dynapps_test_common::extension_services(
        pool,
        env.services.view().clone(),
    )));
    let jwt = env.mint(Principal {
        user_id: user.as_str(),
        email: &format!("{user}@test.invalid"),
        session_id: None,
    });
    let request = Request::get(format!("/connectors/{PROVIDER}/token"))
        .header(header::AUTHORIZATION, format!("Bearer {jwt}"))
        .header("x-systemprompt-credential-broker", BROKER_SECRET)
        .header("x-systemprompt-mcp-server", PROVIDER)
        .header("x-systemprompt-caller", caller)
        .body(Body::empty())
        .expect("request");
    let response = router.oneshot(request).await.expect("router");
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let token = serde_json::from_slice::<Value>(&body)
        .ok()
        .and_then(|v| v["access_token"].as_str().map(str::to_owned))
        // The broker answers the complete Authorization value; core forwards
        // it as-is, so the scheme is checked and the token compared bare.
        .map(|value| {
            value
                .strip_prefix("Bearer ")
                .expect("broker answers a Bearer authorization value")
                .to_owned()
        });
    (status, token)
}

// The forwarded MCP call: initialize, then `whoami` with the bearer.
pub(crate) async fn whoami(portal: &FakePortal, bearer: &str) -> String {
    let http = reqwest::Client::builder()
        .add_root_certificate(reqwest::Certificate::from_pem(&portal.ca_pem()).expect("CA"))
        .build()
        .expect("client");
    let endpoint = portal.endpoint(ODOO_PATH);
    let init = http
        .post(&endpoint)
        .bearer_auth(bearer)
        .json(&json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26"}}))
        .send()
        .await
        .expect("initialize");
    assert!(init.status().is_success(), "initialize: {}", init.status());
    let session = init.headers()["mcp-session-id"]
        .to_str()
        .expect("session")
        .to_owned();
    let call: Value = http
        .post(&endpoint)
        .bearer_auth(bearer)
        .header("Mcp-Session-Id", session)
        .json(&json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"whoami","arguments":{}}}))
        .send()
        .await
        .expect("tools/call")
        .json()
        .await
        .expect("json");
    call["result"]["content"][0]["text"]
        .as_str()
        .expect("text")
        .to_owned()
}

pub(crate) async fn assert_portal_sees(
    env: &PortalEnv,
    pool: &Arc<PgPool>,
    run_id: &str,
    expected: &UserId,
) {
    let (status, bearer) = broker_bearer(pool, env, expected, &format!("farm:{run_id}")).await;
    assert_eq!(status, StatusCode::OK);
    let bearer = bearer.expect("bearer");
    assert_eq!(
        env.portal.subject_of(&bearer),
        Some(portal_account(expected))
    );
    assert_eq!(
        whoami(&env.portal, &bearer).await,
        format!("{}@dynapps-odoo", portal_account(expected))
    );
    assert_eq!(
        env.portal.subjects_seen(ODOO_PATH).last(),
        Some(&portal_account(expected))
    );
}
