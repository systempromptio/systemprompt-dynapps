//! `run_as` modes against a real schema: the seed stores each mode and the
//! client's service account, and a launch issues the run's credentials to
//! the launcher (`user`), the client's account (`client`) or the named
//! account (`service_account`), refusing a `client` run it cannot resolve.
//!
//! The helpers are `pub(crate)` so the fake-portal e2e can set a mode, launch
//! and read the credential user without repeating the setup.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::agent_runs::AgentRunRepository;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_farm::config::RunAsMode;
use systemprompt_web_admin_farm::error::{FarmError, FarmResult, RunAsRefusal};
use systemprompt_web_admin_farm::repositories::rows::RunRow;
use systemprompt_web_admin_farm::repositories::{clients, farms};
use systemprompt_web_admin_farm::services::farm_sync::seed_farms_from_yaml;
use systemprompt_web_admin_farm::services::runs::{LaunchRequest, launch};
use systemprompt_web_admin_farm::types::{ClientId, FarmId, TriggeredVia};

use crate::farm_runner_lifecycle::{Seeded, seed};
use crate::tempdb::TempDb;

pub(crate) async fn insert_service_account(pool: &PgPool, tag: &str) -> UserId {
    let id = UserId::new(format!("svc-{tag}-{}", uuid::Uuid::new_v4().simple()));
    sqlx::query(
        "INSERT INTO users (id, name, email, account_kind, status) \
         VALUES ($1, $1, $1 || '@test.invalid', 'service', 'active')",
    )
    .bind(id.as_str())
    .execute(pool)
    .await
    .expect("insert service account");
    id
}

pub(crate) async fn set_farm_run_as(
    pool: &PgPool,
    farm: &FarmId,
    mode: Option<RunAsMode>,
    account: Option<&UserId>,
    client: Option<&ClientId>,
) {
    sqlx::query(
        "UPDATE agent_farms SET run_as_mode = $1, run_as_user_id = $2, client_id = $3 \
         WHERE id = $4",
    )
    .bind(mode.map(RunAsMode::as_str))
    .bind(account.map(UserId::as_str))
    .bind(client.map(ClientId::as_str))
    .bind(farm.as_str())
    .execute(pool)
    .await
    .expect("set farm run_as");
}

pub(crate) async fn launch_for(seeded: &Seeded, client: Option<ClientId>) -> FarmResult<RunRow> {
    launch(
        &seeded.state,
        LaunchRequest {
            farm_id: seeded.farm.clone(),
            agent_id: seeded.agent.clone(),
            user_id: seeded.user.clone(),
            triggered_via: TriggeredVia::Api,
            prompt: None,
            case_prompt: None,
            fixture: None,
            model: None,
            max_turns: None,
            max_budget_usd: None,
            timeout_seconds: None,
            eval_result_id: None,
            client_id: client,
        },
    )
    .await
}

pub(crate) async fn credential_user_of(pool: &Arc<PgPool>, run: &RunRow) -> UserId {
    AgentRunRepository::from_pool(Arc::clone(pool))
        .find_run(&run.agent_run_id)
        .await
        .expect("read agent run")
        .expect("agent run exists")
        .user_id
}

async fn client_with(pool: &PgPool, account: Option<&UserId>) -> ClientId {
    let slug = format!("client-{}", uuid::Uuid::new_v4().simple());
    clients::upsert_client(pool, &slug, "Client", account)
        .await
        .expect("upsert client");
    ClientId::new(slug)
}

#[tokio::test]
async fn user_mode_issues_credentials_to_the_launcher() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let svc = insert_service_account(&db.pool, "user-mode").await;
    set_farm_run_as(
        &db.pool,
        &seeded.farm,
        Some(RunAsMode::User),
        Some(&svc),
        None,
    )
    .await;
    let run = launch_for(&seeded, None).await.expect("launch");
    assert_eq!(credential_user_of(&db.pool, &run).await, seeded.user);
    db.cleanup().await;
}

#[tokio::test]
async fn client_mode_issues_credentials_to_the_clients_account() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let svc = insert_service_account(&db.pool, "client-mode").await;
    let client = client_with(&db.pool, Some(&svc)).await;
    set_farm_run_as(
        &db.pool,
        &seeded.farm,
        Some(RunAsMode::Client),
        None,
        Some(&client),
    )
    .await;
    let run = launch_for(&seeded, None).await.expect("launch");
    assert_eq!(credential_user_of(&db.pool, &run).await, svc);
    assert_eq!(run.user_id, seeded.user, "the launcher is still recorded");

    let other_svc = insert_service_account(&db.pool, "client-override").await;
    let other = client_with(&db.pool, Some(&other_svc)).await;
    let run = launch_for(&seeded, Some(other)).await.expect("launch");
    assert_eq!(
        credential_user_of(&db.pool, &run).await,
        other_svc,
        "a launch's own client wins over the farm's"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn client_mode_refuses_without_a_client_or_its_account() {
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

    let bare = client_with(&db.pool, None).await;
    let err = launch_for(&seeded, Some(bare))
        .await
        .expect_err("no account");
    assert!(
        matches!(
            err,
            FarmError::RunAs(RunAsRefusal::ClientHasNoAccount { .. })
        ),
        "{err}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn service_account_mode_issues_credentials_to_the_named_account() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let svc = insert_service_account(&db.pool, "sa-mode").await;
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
    db.cleanup().await;
}

#[tokio::test]
async fn the_seed_stores_modes_and_the_clients_account() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let svc = insert_service_account(&db.pool, "seed").await;
    let tag = uuid::Uuid::new_v4().simple().to_string();
    let services = tempfile::tempdir().expect("tempdir");
    let config = services.path().join("web/config");
    std::fs::create_dir_all(&config).expect("mkdir");
    let yaml = format!(
        "clients:
  - slug: seeded-{tag}
    name: Seeded
    service_account: {svc}
farms:
  - id: seeded-{tag}
    name: Seeded
    marketplace: seeded
    client: seeded-{tag}
    run_as:
      mode: client
    source:
      github: dynapps/agent-farm
"
    );
    std::fs::write(config.join("agent-farms.yaml"), yaml).expect("write yaml");
    seed_farms_from_yaml(&db.pool, services.path())
        .await
        .expect("seed");

    let farm = farms::find_farm(&db.pool, &FarmId::new(format!("seeded-{tag}")))
        .await
        .expect("read farm")
        .expect("farm seeded");
    assert_eq!(farm.run_as_mode.as_deref(), Some("client"));
    assert_eq!(farm.run_as_user_id, None, "client mode resolves per launch");
    let client = clients::find_client(&db.pool, &format!("seeded-{tag}"))
        .await
        .expect("read client")
        .expect("client seeded");
    assert_eq!(client.service_account_user_id, Some(svc));
    db.cleanup().await;
}
