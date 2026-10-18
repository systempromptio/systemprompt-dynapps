//! Client attribution, per-agent run-as and the task-completion report,
//! against a real schema.
//!
//! A launch takes its farm's client unless it names one; an agent's own
//! run-as account is preferred over the farm's when credentials are minted;
//! a finished run gets a report with its MCP calls per server split into
//! reads and writes, its `DynBrain` delivery, the governance `warn` findings
//! of its context, and the run facts (trigger, turns, tokens, cost).

use systemprompt::agent_runs::{
    AgentRunRepository, ErrorClass, RunEvent, RunEventPayload, RunOutcome, RunStatus,
};
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_farm::config::RunAsMode;
use systemprompt_web_admin_farm::repositories::{agents, clients, reports, runs};
use systemprompt_web_admin_farm::services::runs::{LaunchRequest, launch, resolve_client};
use systemprompt_web_admin_farm::types::{ClientId, TriggeredVia};

use crate::farm_runner_lifecycle::{Seeded, launch_one, observe_finish, seed};
use crate::tempdb::TempDb;

async fn add_clients(seeded: &Seeded, tag: &str) -> (ClientId, ClientId) {
    let pool = seeded.state.pool.as_ref();
    let own = format!("own-{tag}");
    let other = format!("other-{tag}");
    clients::upsert_client(pool, &own, "Own", None)
        .await
        .expect("insert own client");
    clients::upsert_client(pool, &other, "Other", None)
        .await
        .expect("insert other client");
    sqlx::query("UPDATE agent_farms SET client_id = $1 WHERE id = $2")
        .bind(&own)
        .bind(seeded.farm.as_str())
        .execute(pool)
        .await
        .expect("bind the farm to its client");
    (ClientId::new(own), ClientId::new(other))
}

fn request(seeded: &Seeded, client_id: Option<ClientId>) -> LaunchRequest {
    LaunchRequest {
        farm_id: seeded.farm.clone(),
        agent_id: seeded.agent.clone(),
        user_id: seeded.user.clone(),
        triggered_via: TriggeredVia::Mcp,
        prompt: None,
        case_prompt: None,
        fixture: None,
        model: None,
        max_turns: None,
        max_budget_usd: None,
        timeout_seconds: None,
        eval_result_id: None,
        client_id,
    }
}

#[tokio::test]
async fn a_launch_takes_the_farms_client_unless_it_names_one() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let tag = uuid::Uuid::new_v4().simple().to_string();
    let (own, other) = add_clients(&seeded, &tag).await;

    let default_run = launch_one(&seeded).await;
    assert_eq!(default_run.client_id.as_ref(), Some(&own));

    let named = resolve_client(&db.pool, Some(other.as_str()))
        .await
        .expect("resolve client");
    let named_run = launch(&seeded.state, request(&seeded, named))
        .await
        .expect("launch");
    assert_eq!(named_run.client_id.as_ref(), Some(&other));

    let unknown = resolve_client(&db.pool, Some("no-such-client")).await;
    assert!(unknown.is_err(), "an unknown client slug is refused");
    assert!(
        resolve_client(&db.pool, Some("  "))
            .await
            .expect("blank")
            .is_none()
    );
    db.cleanup().await;
}

#[tokio::test]
async fn an_agents_run_as_is_preferred_over_the_launcher() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let svc = UserId::new(format!("svc-{}", uuid::Uuid::new_v4().simple()));
    sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $1, $1 || '@test.invalid')")
        .bind(svc.as_str())
        .execute(db.pool.as_ref())
        .await
        .expect("insert service user");
    agents::update_agent_run_as(
        &db.pool,
        &seeded.farm,
        seeded.agent.as_str(),
        Some(&svc),
        Some(RunAsMode::ServiceAccount),
    )
    .await
    .expect("set agent run_as");

    let run = launch_one(&seeded).await;
    let agent_run = AgentRunRepository::from_pool(std::sync::Arc::clone(&db.pool))
        .find_run(&run.agent_run_id)
        .await
        .expect("read agent run")
        .expect("agent run exists");
    assert_eq!(
        agent_run.user_id, svc,
        "credentials are minted for the agent's account"
    );
    assert_eq!(run.user_id, seeded.user, "the launcher is still recorded");
    db.cleanup().await;
}

fn event(seq: i64, payload: RunEventPayload) -> RunEvent {
    let (tool_name, tool_use_id) = match &payload {
        RunEventPayload::ToolUse {
            tool_use_id, name, ..
        } => (Some(name.clone()), Some(tool_use_id.clone())),
        RunEventPayload::ToolResult { tool_use_id, .. } => (None, Some(tool_use_id.clone())),
        _ => (None, None),
    };
    RunEvent {
        seq,
        kind: payload.kind(),
        tool_name,
        tool_use_id,
        payload,
        created_at: chrono::Utc::now(),
    }
}

fn call(seq: i64, id: &str, name: &str, is_error: bool, content: &str) -> [RunEvent; 2] {
    [
        event(
            seq,
            RunEventPayload::ToolUse {
                tool_use_id: id.to_owned(),
                name: name.to_owned(),
                input: serde_json::json!({}),
            },
        ),
        event(
            seq + 1,
            RunEventPayload::ToolResult {
                tool_use_id: id.to_owned(),
                content: content.to_owned(),
                is_error,
            },
        ),
    ]
}

async fn warn_decision(pool: &sqlx::PgPool, user: &UserId, context: &str, decision: &str) {
    sqlx::query(
        "INSERT INTO governance_decisions (id, user_id, session_id, tool_name, decision, policy, \
         reason, actor_kind, actor_id, context_id) \
         VALUES ($1, $2, 'sess', 'mcp__dynapps-odoo__orm_write', $3, 'scope_check', 'r', \
         'user', $2, $4)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(user.as_str())
    .bind(decision)
    .bind(context)
    .execute(pool)
    .await
    .expect("insert governance decision");
}

#[tokio::test]
async fn a_finished_run_gets_a_report_with_its_mcp_tally_and_facts() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let tag = uuid::Uuid::new_v4().simple().to_string();
    let (own, _) = add_clients(&seeded, &tag).await;
    let run = launch_one(&seeded).await;
    let context = format!("ctx-{tag}");
    sqlx::query("UPDATE agent_farm_runs SET context_id = $1 WHERE id = $2")
        .bind(&context)
        .bind(run.id.as_str())
        .execute(db.pool.as_ref())
        .await
        .expect("bind the run's context");
    warn_decision(&db.pool, &seeded.user, &context, "warn").await;
    warn_decision(&db.pool, &seeded.user, &context, "allow").await;
    warn_decision(&db.pool, &seeded.user, "another-context", "warn").await;
    let events: Vec<RunEvent> = [
        call(1, "t1", "mcp__dynapps-odoo__orm_search_read", false, "[]"),
        call(3, "t2", "mcp__dynapps-odoo__orm_search_read", false, "[]"),
        call(5, "t3", "mcp__dynapps-odoo__orm_write", true, "denied"),
        call(
            7,
            "t4",
            "mcp__dynbrain__document_create",
            false,
            r#"{"id": "doc-1"}"#,
        ),
        call(9, "t5", "Read", false, ""),
    ]
    .into_iter()
    .flatten()
    .collect();
    AgentRunRepository::from_pool(std::sync::Arc::clone(&db.pool))
        .insert_events_batch(&run.agent_run_id, &events)
        .await
        .expect("insert run events");

    let outcome = RunOutcome {
        status: RunStatus::Succeeded,
        error_class: ErrorClass::Ok,
        session_id: None,
        cost_microdollars: 1_000,
        input_tokens: 10,
        output_tokens: 5,
        turns: 1,
        duration_ms: 100,
        structured_output: None,
        exit_code: Some(0),
        error: None,
        final_text: None,
    };
    observe_finish(&seeded.state, &run, &outcome).await;

    let report = reports::find_run_report(&db.pool, &run.id)
        .await
        .expect("read report")
        .expect("the observer wrote a report");
    assert!(report.completed, "a succeeded run with no eval completed");
    assert_eq!(report.client_id.as_ref(), Some(&own));
    assert_eq!(report.eval_passed, None);
    assert_eq!(
        report.mcp_calls,
        serde_json::json!([
            {"server": "dynapps-odoo", "reads": 2, "writes": 1, "failed": 1},
            {"server": "dynbrain", "reads": 0, "writes": 1, "failed": 0},
        ])
    );
    assert_eq!((report.mcp_reads, report.mcp_writes), (2, 2));
    assert_eq!(report.delivery_status, "delivered");
    assert_eq!(report.governance_warnings, 1, "only this run's warn rows");
    assert_eq!(report.triggered_via, "api");
    assert_eq!(report.cost_microdollars, 1_000);
    assert_eq!(report.turns, Some(1));
    assert_eq!(
        (report.input_tokens, report.output_tokens),
        (Some(10), Some(5))
    );
    assert_eq!(report.outputs, serde_json::json!([]));

    let stamped = runs::find_run(&db.pool, &run.id)
        .await
        .expect("read run")
        .expect("run");
    assert_eq!(stamped.client_id.as_ref(), Some(&own));
    db.cleanup().await;
}
