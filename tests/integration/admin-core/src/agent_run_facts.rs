//! `repositories::agent_run_facts` — the observed-fact view of a run.
//!
//! Every number is seeded by hand against core's `agent_runs` and
//! `agent_run_events`, the gateway's `ai_requests` on the run's session, and
//! `governance_decisions` on that same session, so each assertion names the
//! row that produced it. `agent_runs` carries no seed rows, so stats assert
//! absolute counts within the one user the test inserts.

use serde_json::{Value, json};
use sqlx::PgPool;
use systemprompt::identifiers::{AgentRunId, UserId};
use systemprompt_web_admin_core::repositories::agent_run_facts::{
    RunFilter, RunScope, find_run_facts, get_agent_run_stats, list_run_facts,
};

use crate::fixtures::{
    DecisionSpec, RequestSpec, insert_decision, insert_request, insert_session, insert_user,
    unclaimed_email, unique, unique_project,
};
use crate::tempdb::TempDb;

const OUTPUT_SCHEMA: &str = r#"{"type":"object","required":["verdict"],
    "properties":{"verdict":{"type":"string"}}}"#;

struct RunSeed<'a> {
    user: &'a UserId,
    session: Option<&'a str>,
    origin_kind: &'a str,
    origin_id: &'a str,
    status: &'a str,
    error_class: Option<&'a str>,
    request: Value,
    structured_output: Option<Value>,
    duration_ms: i64,
}

async fn insert_run(pool: &PgPool, seed: &RunSeed<'_>) -> AgentRunId {
    let id = AgentRunId::generate();
    sqlx::query(
        "INSERT INTO agent_runs (id, origin_kind, origin_id, user_id, host, status, request,
             attempt, gateway_session_id, started_at, finished_at, duration_ms, error_class,
             turns, input_tokens, output_tokens, cost_microdollars, structured_output,
             diff_summary)
         VALUES ($1, $2, $3, $4, 'claude-code', $5, $6, 2, $7, NOW(), NOW(), $8, $9,
                 7, 1000, 200, 4200, $10, '{\"files_changed\": 1}'::jsonb)",
    )
    .bind(id.as_str())
    .bind(seed.origin_kind)
    .bind(seed.origin_id)
    .bind(seed.user.as_str())
    .bind(seed.status)
    .bind(&seed.request)
    .bind(seed.session)
    .bind(seed.duration_ms)
    .bind(seed.error_class)
    .bind(&seed.structured_output)
    .execute(pool)
    .await
    .expect("insert agent run");
    id
}

async fn insert_event(pool: &PgPool, run: &AgentRunId, seq: i64, kind: &str, payload: Value) {
    let tool_name = payload
        .get("name")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    sqlx::query(
        "INSERT INTO agent_run_events (run_id, seq, kind, tool_name, tool_use_id, payload)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(run.as_str())
    .bind(seq)
    .bind(kind)
    .bind(tool_name)
    .bind(payload.get("tool_use_id").and_then(Value::as_str))
    .bind(&payload)
    .execute(pool)
    .await
    .expect("insert agent run event");
}

async fn insert_farm_link(pool: &PgPool, run: &AgentRunId, user: &UserId, project: &str) -> String {
    let farm_id = unique("farm");
    sqlx::query(
        "INSERT INTO agent_farms (id, name, marketplace_id, source_repo, project_id)
         VALUES ($1, $1, 'mp', 'https://example.invalid/repo', $2)",
    )
    .bind(&farm_id)
    .bind(project)
    .execute(pool)
    .await
    .expect("insert farm");
    sqlx::query(
        "INSERT INTO agent_farm_runs (id, agent_run_id, farm_id, agent_id, user_id, triggered_via,
             prompt, permission_mode, max_turns, max_budget_usd, timeout_seconds)
         VALUES ($1, $2, $3, 'agent', $4, 'api', 'go', 'acceptEdits', 10, 1.0, 60)",
    )
    .bind(unique("frun"))
    .bind(run.as_str())
    .bind(&farm_id)
    .bind(user.as_str())
    .execute(pool)
    .await
    .expect("insert farm run");
    farm_id
}

async fn insert_attributed_request(pool: &PgPool, user: &UserId, session: &str, farm: &str) {
    let id = unique("req");
    let mut spec = RequestSpec::completed(&id, user);
    spec.session_id = Some(session);
    insert_request(pool, &spec).await;
    sqlx::query(
        "INSERT INTO ai_request_attributions (request_id, dimension, value, source)
         VALUES ($1, 'farm', $2, 'header')",
    )
    .bind(&id)
    .bind(farm)
    .execute(pool)
    .await
    .expect("insert attribution");
}

async fn seed_full_run(
    pool: &PgPool,
    user: &UserId,
    session: &str,
    project: &str,
    farm_ids: &mut Vec<String>,
) -> AgentRunId {
    let run = insert_run(
        pool,
        &RunSeed {
            user,
            session: Some(session),
            origin_kind: "farm",
            origin_id: "farm-run-1",
            status: "succeeded",
            error_class: Some("ok"),
            request: json!({"prompt": "go", "output_schema": serde_json::from_str::<Value>(OUTPUT_SCHEMA).expect("schema")}),
            structured_output: Some(json!({"verdict": "pass"})),
            duration_ms: 1000,
        },
    )
    .await;
    let tools = [("Bash", "t1"), ("Read", "t2"), ("Bash", "t3")];
    for (seq, (name, id)) in tools.iter().enumerate() {
        let seq = seq as i64;
        insert_event(
            pool,
            &run,
            seq * 2,
            "tool_use",
            json!({"kind": "tool_use", "tool_use_id": id, "name": name, "input": {}}),
        )
        .await;
        let is_error = *id == "t2";
        insert_event(
            pool,
            &run,
            seq * 2 + 1,
            "tool_result",
            json!({"kind": "tool_result", "tool_use_id": id, "content": "", "is_error": is_error}),
        )
        .await;
    }
    insert_event(
        pool,
        &run,
        10,
        "hook",
        json!({"kind": "hook", "name": "govern", "decision": "deny", "policy": "blocklist"}),
    )
    .await;
    insert_event(
        pool,
        &run,
        11,
        "hook",
        json!({"kind": "hook", "name": "govern", "decision": "allow", "policy": null}),
    )
    .await;
    let farm = insert_farm_link(pool, &run, user, project).await;
    farm_ids.push(farm.clone());
    insert_attributed_request(pool, user, session, &farm).await;
    insert_attributed_request(pool, user, session, &farm).await;
    insert_decision(pool, &DecisionSpec::allow(&unique("dec"), user, session)).await;
    let mut deny = DecisionSpec::allow(&unique("dec"), user, session);
    deny.decision = "deny";
    insert_decision(pool, &deny).await;
    run
}

#[tokio::test]
async fn find_run_facts_reports_every_recorded_number() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("facts")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let project = unique_project("proj");
    let run = seed_full_run(&db.pool, &user, &session, project.as_str(), &mut Vec::new()).await;

    let facts = find_run_facts(&db.pool, &run)
        .await
        .expect("query succeeds")
        .expect("run exists");

    assert_eq!(facts.run_id, run);
    assert_eq!(facts.origin_kind, "farm");
    assert_eq!(facts.user_id, user);
    assert_eq!(facts.project_id, Some(project));
    assert_eq!(facts.status, "succeeded");
    assert_eq!(facts.error_class.as_deref(), Some("ok"));
    assert_eq!(facts.attempt, 2);
    assert_eq!(facts.turns, Some(7));
    assert_eq!(facts.duration_ms, Some(1000));
    assert_eq!(facts.input_tokens, Some(1000));
    assert_eq!(facts.output_tokens, Some(200));
    assert_eq!(facts.cost_microdollars_reported, Some(4200));
    assert_eq!(
        facts.cost_microdollars_metered, 10_000,
        "two requests at 5 000 each"
    );
    assert_eq!(facts.tool_use_count, 3);
    assert_eq!(facts.tool_result_count, 3);
    assert_eq!(facts.tool_result_error_count, 1);
    assert_eq!(
        facts.tools_by_name,
        vec![("Bash".to_owned(), 2), ("Read".to_owned(), 1)]
    );
    assert_eq!(facts.hook_allow, 1);
    assert_eq!(facts.hook_deny, 1);
    assert_eq!(facts.governance_allow, 1);
    assert_eq!(facts.governance_deny, 1);
    assert!(!facts.budget_exceeded);
    assert!(!facts.timed_out);
    assert_eq!(facts.structured_output, Some(json!({"verdict": "pass"})));
    assert_eq!(facts.output_schema_valid, Some(true));
    assert_eq!(facts.diff_summary, Some(json!({"files_changed": 1})));
    db.cleanup().await;
}

#[tokio::test]
async fn output_validity_is_false_when_wrong_and_absent_without_a_schema() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("valid")).await;
    let schema: Value = serde_json::from_str(OUTPUT_SCHEMA).expect("schema");
    let invalid = insert_run(
        &db.pool,
        &RunSeed {
            user: &user,
            session: None,
            origin_kind: "workflow_step",
            origin_id: "wf-run/step-a/3",
            status: "failed",
            error_class: Some("output_invalid"),
            request: json!({"output_schema": schema}),
            structured_output: Some(json!({"verdict": 7})),
            duration_ms: 50,
        },
    )
    .await;
    let unschemaed = insert_run(
        &db.pool,
        &RunSeed {
            user: &user,
            session: None,
            origin_kind: "farm",
            origin_id: "farm-run-2",
            status: "timed_out",
            error_class: Some("timeout"),
            request: json!({"prompt": "go"}),
            structured_output: Some(json!({"anything": true})),
            duration_ms: 90,
        },
    )
    .await;

    let invalid = find_run_facts(&db.pool, &invalid)
        .await
        .expect("query")
        .expect("row");
    assert_eq!(invalid.output_schema_valid, Some(false));
    assert_eq!(invalid.attempt, 3, "parsed from the workflow origin id");
    assert_eq!(
        invalid.project_id, None,
        "no workflow_runs table on this instance"
    );
    assert_eq!(invalid.cost_microdollars_metered, 0, "no gateway session");

    let unschemaed = find_run_facts(&db.pool, &unschemaed)
        .await
        .expect("query")
        .expect("row");
    assert_eq!(unschemaed.output_schema_valid, None);
    assert!(unschemaed.timed_out);
    assert_eq!(
        unschemaed.attempt, 2,
        "core's own counter outside a workflow"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn stats_and_list_honour_scope() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("stats")).await;
    let stranger = insert_user(&db.pool, &unique("user"), &unclaimed_email("other")).await;
    let session = unique("session");
    insert_session(&db.pool, &session, &user).await;
    let project = unique_project("proj");
    let mut farm_ids = Vec::new();
    seed_full_run(&db.pool, &user, &session, project.as_str(), &mut farm_ids).await;
    let farm_id = farm_ids.pop().expect("seeded farm");
    insert_run(
        &db.pool,
        &RunSeed {
            user: &user,
            session: None,
            origin_kind: "workflow_step",
            origin_id: "wf/step/1",
            status: "failed",
            error_class: Some("budget_exceeded"),
            request: json!({}),
            structured_output: None,
            duration_ms: 3000,
        },
    )
    .await;

    let own = RunScope::Own(user.clone());
    let stats = get_agent_run_stats(&db.pool, &own, &RunFilter::default())
        .await
        .expect("stats");
    assert_eq!(stats.totals.total, 2);
    assert_eq!(stats.totals.by_status.succeeded, 1);
    assert_eq!(stats.totals.by_status.failed, 1);
    assert_eq!(stats.totals.by_error_class.ok, 1);
    assert_eq!(stats.totals.by_error_class.budget_exceeded, 1);
    assert_eq!(stats.totals.cost_microdollars_reported, 8400);
    assert_eq!(stats.totals.cost_microdollars_metered, 10_000);
    assert_eq!(stats.totals.tool_use_count, 3);
    assert_eq!(stats.totals.hook_deny, 1);
    assert_eq!(stats.totals.p50_duration_ms, Some(2000.0));
    assert_eq!(stats.by_origin_kind.len(), 2);
    let farm = stats
        .by_origin_kind
        .iter()
        .find(|b| b.origin_kind == "farm")
        .expect("farm bucket");
    assert_eq!(farm.stats.total, 1);
    assert_eq!(farm.stats.cost_microdollars_metered, 10_000);

    let listed = list_run_facts(&db.pool, &own, &RunFilter::default())
        .await
        .expect("list");
    assert_eq!(listed.len(), 2);
    let by_project = list_run_facts(
        &db.pool,
        &RunScope::Projects(vec![project]),
        &RunFilter::default(),
    )
    .await
    .expect("list");
    assert_eq!(by_project.len(), 1, "only the farm run has a project");
    let filtered = RunFilter {
        status: Some("failed".to_owned()),
        ..RunFilter::default()
    };
    let by_farm = RunFilter {
        farm_id: Some(farm_id),
        ..RunFilter::default()
    };
    assert_eq!(
        get_agent_run_stats(&db.pool, &RunScope::All, &by_farm)
            .await
            .expect("stats")
            .totals
            .total,
        1,
        "one farm's runs"
    );
    assert_eq!(
        list_run_facts(&db.pool, &RunScope::All, &by_farm)
            .await
            .expect("list")
            .len(),
        1
    );
    assert_eq!(
        list_run_facts(&db.pool, &RunScope::All, &filtered)
            .await
            .expect("list")
            .len(),
        1
    );

    let theirs = RunScope::Own(stranger);
    assert!(
        list_run_facts(&db.pool, &theirs, &RunFilter::default())
            .await
            .expect("list")
            .is_empty()
    );
    assert_eq!(
        get_agent_run_stats(&db.pool, &theirs, &RunFilter::default())
            .await
            .expect("stats")
            .totals
            .total,
        0
    );
    db.cleanup().await;
}
