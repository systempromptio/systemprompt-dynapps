//! Farm `run_as` end to end against the fake portal: for each mode the
//! agent run is issued to the expected identity, and the bearer that reaches
//! the portal's Odoo MCP belongs to that identity's portal account -- never
//! the launcher's when the mode says otherwise.
//!
//! Seam: no farm runner process is started. A launch records the run and its
//! credential user exactly as production does; the runner's gateway calls
//! are then reproduced at the point identity is decided -- the broker request
//! core's external-MCP proxy makes for that run (the credential user's JWT
//! plus the `farm:<run id>` caller purpose), through the real
//! `connector_api_router` -- and the returned bearer is forwarded to the fake
//! portal's MCP endpoint (`farm_portal_support`).

use axum::http::StatusCode;
use dynapps_test_common::{ODOO_PATH, PortalEnv};
use systemprompt_web_admin_farm::config::RunAsMode;
use systemprompt_web_admin_farm::error::{FarmError, RunAsRefusal};
use systemprompt_web_admin_farm::repositories::clients;
use systemprompt_web_admin_farm::types::ClientId;

use crate::farm_portal_support::{assert_portal_sees, broker_bearer, connect};
use crate::farm_run_as_modes::{
    credential_user_of, insert_service_account, launch_for, set_farm_run_as,
};
use crate::farm_runner_lifecycle::seed;
use crate::tempdb::TempDb;

#[tokio::test(flavor = "multi_thread")]
async fn user_mode_reaches_the_portal_as_the_launcher() {
    let env = PortalEnv::start().await;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    set_farm_run_as(&db.pool, &seeded.farm, Some(RunAsMode::User), None, None).await;
    connect(&db.pool, &env.portal, &seeded.user).await;
    let run = launch_for(&seeded, None).await.expect("launch");
    assert_eq!(credential_user_of(&db.pool, &run).await, seeded.user);
    assert_portal_sees(&env, &db.pool, run.id.as_str(), &seeded.user).await;

    let (status, bearer) = broker_bearer(&db.pool, &seeded.user, "farm:not-this-run").await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a run the person is not running"
    );
    assert!(bearer.is_none());
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn client_mode_reaches_the_portal_as_the_clients_account() {
    let env = PortalEnv::start().await;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let svc = insert_service_account(&db.pool, "e2e-client").await;
    let slug = format!("client-{}", uuid::Uuid::new_v4().simple());
    clients::upsert_client(&db.pool, &slug, "Client", None, Some(&svc))
        .await
        .expect("upsert client");
    let client = ClientId::new(slug);
    set_farm_run_as(
        &db.pool,
        &seeded.farm,
        Some(RunAsMode::Client),
        None,
        Some(&client),
    )
    .await;
    connect(&db.pool, &env.portal, &seeded.user).await;
    connect(&db.pool, &env.portal, &svc).await;
    let run = launch_for(&seeded, None).await.expect("launch");
    assert_eq!(credential_user_of(&db.pool, &run).await, svc);
    assert_eq!(run.user_id, seeded.user, "the launcher is still recorded");
    assert_portal_sees(&env, &db.pool, run.id.as_str(), &svc).await;
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn service_account_mode_reaches_the_portal_as_the_named_account() {
    let env = PortalEnv::start().await;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let svc = insert_service_account(&db.pool, "e2e-sa").await;
    set_farm_run_as(
        &db.pool,
        &seeded.farm,
        Some(RunAsMode::ServiceAccount),
        Some(&svc),
        None,
    )
    .await;
    connect(&db.pool, &env.portal, &seeded.user).await;
    connect(&db.pool, &env.portal, &svc).await;
    let run = launch_for(&seeded, None).await.expect("launch");
    assert_eq!(credential_user_of(&db.pool, &run).await, svc);
    assert_portal_sees(&env, &db.pool, run.id.as_str(), &svc).await;

    let (status, _) = broker_bearer(&db.pool, &svc, "interactive").await;
    assert!(
        !status.is_success(),
        "a service account's grant needs a farm or workflow purpose"
    );
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn client_mode_without_a_client_is_refused_before_any_portal_call() {
    let env = PortalEnv::start().await;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    set_farm_run_as(&db.pool, &seeded.farm, Some(RunAsMode::Client), None, None).await;
    let err = launch_for(&seeded, None).await.expect_err("no client");
    assert!(
        matches!(err, FarmError::RunAs(RunAsRefusal::NoClient)),
        "{err}"
    );
    assert!(env.portal.requests().is_empty());
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_missing_grant_yields_no_bearer_in_every_mode() {
    let env = PortalEnv::start().await;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    set_farm_run_as(&db.pool, &seeded.farm, Some(RunAsMode::User), None, None).await;
    let run = launch_for(&seeded, None).await.expect("launch");
    let caller = format!("farm:{}", run.id.as_str());
    let (status, bearer) = broker_bearer(&db.pool, &seeded.user, &caller).await;
    assert!(
        !status.is_success(),
        "user mode, launcher never connected: {status}"
    );
    assert!(bearer.is_none());

    let svc = insert_service_account(&db.pool, "e2e-no-grant").await;
    let seeded = seed(&db.pool).await;
    connect(&db.pool, &env.portal, &seeded.user).await;
    set_farm_run_as(
        &db.pool,
        &seeded.farm,
        Some(RunAsMode::ServiceAccount),
        Some(&svc),
        None,
    )
    .await;
    let run = launch_for(&seeded, None).await.expect("launch");
    assert_eq!(credential_user_of(&db.pool, &run).await, svc);
    let caller = format!("farm:{}", run.id.as_str());
    let (status, bearer) = broker_bearer(&db.pool, &svc, &caller).await;
    assert!(
        !status.is_success(),
        "service account never connected: {status}"
    );
    assert!(
        bearer.is_none(),
        "the launcher's grant is never lent to the account"
    );
    assert!(env.portal.bearers_seen(ODOO_PATH).is_empty());
    db.cleanup().await;
}
