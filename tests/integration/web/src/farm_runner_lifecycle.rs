//! A farm run's lifecycle over core's `agent_runs`, against a real schema.
//!
//! A launch writes two rows in one transaction: the `agent_runs` row core's
//! claim loop picks up (origin `agent_farm_run`, origin id = the farm run id)
//! and the `agent_farm_runs` row that references it. A cancel before any
//! claim flags both. The finish observer copies a `RunOutcome` onto the farm
//! row, attempt included. No executor is started, so no host process is ever
//! spawned.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::agent_runs::{AgentRunRepository, ErrorClass, RunOutcome, RunStatus};
use systemprompt::identifiers::{MarketplaceId, PluginId, UserId};
use systemprompt_web_admin_farm::repositories::filters::{NewAgent, NewFarm};
use systemprompt_web_admin_farm::repositories::rows::RunRow;
use systemprompt_web_admin_farm::repositories::{agents, farms, runs};
use systemprompt_web_admin_farm::services::runner::ORIGIN_KIND;
use systemprompt_web_admin_farm::services::runs::{LaunchRequest, cancel, launch};
use systemprompt_web_admin_farm::state::{
    FarmPaths, FarmState, GatewayEndpoints, RunnerLimits, build_farm_state,
};
use systemprompt_web_admin_farm::types::{FarmAgentId, FarmId, TriggeredVia};

use crate::tempdb::TempDb;

pub(crate) struct Seeded {
    pub(crate) state: Arc<FarmState>,
    pub(crate) user: UserId,
    pub(crate) farm: FarmId,
    pub(crate) agent: FarmAgentId,
}

pub(crate) async fn seed(pool: &Arc<PgPool>) -> Seeded {
    let tag = uuid::Uuid::new_v4().simple().to_string();
    let user = UserId::new(format!("farm-lifecycle-{tag}"));
    sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $1, $1 || '@test.invalid')")
        .bind(user.as_str())
        .execute(pool.as_ref())
        .await
        .expect("insert user");
    let farm = FarmId::new(format!("farm-{tag}"));
    farms::upsert_farm(
        pool,
        &NewFarm {
            id: farm.clone(),
            name: "Lifecycle".to_owned(),
            description: None,
            marketplace_id: MarketplaceId::new("lifecycle-marketplace"),
            source_repo: "https://example.invalid/kit.git".to_owned(),
            source_ref: "main".to_owned(),
            source_subdir: String::new(),
            permission_mode: "acceptEdits".to_owned(),
            timeout_seconds: 600,
            max_turns: 20,
            max_budget_usd: 2.0,
            model: None,
            run_as_user_id: None,
            project_id: None,
            client_id: None,
        },
    )
    .await
    .expect("upsert farm");
    let agent = FarmAgentId::new("reviewer");
    agents::upsert_agent(
        pool,
        &NewAgent {
            farm_id: farm.clone(),
            id: agent.clone(),
            name: "reviewer".to_owned(),
            description: "reviews".to_owned(),
            model: Some("claude-sonnet-5-5".to_owned()),
            host: "codex-cli".to_owned(),
            tools: vec!["Read".to_owned()],
            plugin_id: PluginId::new("lifecycle-plugin"),
            skills: Vec::new(),
            entry_prompt: "Review the module.".to_owned(),
            fixture: None,
            max_turns: None,
            max_budget_usd: None,
            output_path: None,
            output_format: None,
            source_path: "agents/reviewer.md".to_owned(),
            body: "Body".to_owned(),
            synced_commit_sha: "abc123".to_owned(),
        },
    )
    .await
    .expect("upsert agent");
    let data_root = std::env::temp_dir().join(format!("farm-lifecycle-{tag}"));
    let state = build_farm_state(
        Arc::clone(pool),
        FarmPaths::new(data_root.clone(), data_root.join("services")),
        GatewayEndpoints {
            internal_base: String::new(),
            external_base: String::new(),
        },
        RunnerLimits::default(),
    );
    Seeded {
        state,
        user,
        farm,
        agent,
    }
}

pub(crate) async fn launch_one(seeded: &Seeded) -> RunRow {
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
            client_id: None,
        },
    )
    .await
    .expect("launch")
}

fn repo(pool: &Arc<PgPool>) -> AgentRunRepository {
    AgentRunRepository::from_pool(Arc::clone(pool))
}

#[tokio::test]
async fn launch_queues_an_agent_run_and_a_farm_row_that_references_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let run = launch_one(&seeded).await;

    assert_eq!(run.status, "queued");
    assert_eq!(
        run.host, "codex-cli",
        "the agent's host is copied to the run"
    );
    let agent_run = repo(&db.pool)
        .find_run(&run.agent_run_id)
        .await
        .expect("read agent_runs")
        .expect("launch inserted the agent_runs row");
    assert_eq!(ORIGIN_KIND, "agent_farm_run");
    assert_eq!(agent_run.origin_kind, ORIGIN_KIND);
    assert_eq!(agent_run.origin_id, run.id.as_str());
    assert_eq!(agent_run.status, "queued");
    assert_eq!(agent_run.host, "codex-cli");
    assert_eq!(agent_run.user_id, seeded.user);
    assert_eq!(agent_run.request["prompt"], "Review the module.");
    assert_eq!(agent_run.request["model"], "claude-sonnet-5-5");
    db.cleanup().await;
}

#[tokio::test]
async fn cancel_before_claim_flags_the_agent_run_and_cancels_the_farm_run() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let run = launch_one(&seeded).await;

    let cancelled = cancel(&seeded.state, &run.id, &seeded.user)
        .await
        .expect("cancel");
    assert!(cancelled, "a queued run can be cancelled");
    let repo = repo(&db.pool);
    assert!(
        repo.find_cancel_requested(&run.agent_run_id)
            .await
            .expect("read cancel flag"),
        "the claim loop must see the cancel on agent_runs"
    );
    let farm_row = runs::find_run(&db.pool, &run.id)
        .await
        .expect("read farm run")
        .expect("farm run exists");
    assert_eq!(farm_row.status, "cancelled");
    assert!(farm_row.finished_at.is_some());

    let again = cancel(&seeded.state, &run.id, &seeded.user)
        .await
        .expect("second cancel");
    assert!(!again, "a terminal run is not cancelled twice");
    db.cleanup().await;
}

#[tokio::test]
async fn the_finish_observer_stamps_the_outcome_on_the_farm_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let run = launch_one(&seeded).await;
    let outcome = RunOutcome {
        status: RunStatus::Succeeded,
        error_class: ErrorClass::Ok,
        session_id: None,
        cost_microdollars: 42_000,
        input_tokens: 1_000,
        output_tokens: 200,
        turns: 3,
        duration_ms: 1_500,
        structured_output: None,
        exit_code: Some(0),
        error: None,
        final_text: Some("Done.".to_owned()),
    };

    observe_finish(&seeded.state, &run, &outcome).await;

    let farm_row = runs::find_run(&db.pool, &run.id)
        .await
        .expect("read farm run")
        .expect("farm run exists");
    assert_eq!(farm_row.status, "succeeded");
    assert_eq!(farm_row.error_class.as_deref(), Some("ok"));
    assert_eq!(farm_row.cost_microdollars, 42_000);
    assert!(farm_row.finished_at.is_some());
    assert_eq!(farm_row.attempt, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn the_finish_observer_stamps_a_reclaimed_runs_attempt() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let run = launch_one(&seeded).await;
    assert_eq!(run.attempt, 1);
    sqlx::query("UPDATE agent_runs SET attempt = 2 WHERE id = $1")
        .bind(run.agent_run_id.as_str())
        .execute(db.pool.as_ref())
        .await
        .expect("simulate a reclaim");
    let outcome = systemprompt_web_admin_farm::services::runner::observer::unexecuted_outcome(
        RunStatus::Failed,
        ErrorClass::Timeout,
        "lease expired",
    );

    observe_finish(&seeded.state, &run, &outcome).await;

    let farm_row = runs::find_run(&db.pool, &run.id)
        .await
        .expect("read farm run")
        .expect("farm run exists");
    assert_eq!(farm_row.attempt, 2, "the core row's attempt is stamped");
    db.cleanup().await;
}

// The executor finishes the core row; the farm observer then stamps the
// outcome onto the farm row from that record.
pub(crate) async fn observe_finish(state: &Arc<FarmState>, run: &RunRow, outcome: &RunOutcome) {
    let finished = AgentRunRepository::from_pool(Arc::clone(&state.pool))
        .finish(&run.agent_run_id, outcome, None)
        .await
        .expect("finish the agent run");
    assert!(finished, "the agent run was still open");
    let stamped = systemprompt_web_admin_farm::services::runner::observer::observe_finished(
        state,
        &run.agent_run_id,
    )
    .await
    .expect("observe the finish");
    assert!(stamped, "the observer stamped the farm row");
}
