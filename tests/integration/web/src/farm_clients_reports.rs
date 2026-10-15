//! Client attribution, per-agent run-as and the task-completion report,
//! against a real schema.
//!
//! A launch takes its farm's client unless it names one; an agent's own
//! run-as account is preferred over the farm's when credentials are minted;
//! a finished run gets a report whose Odoo writes are the odoo MCP audit rows
//! carrying its farm run id (read-only tools left out).

use systemprompt::agent_runs::{AgentRunRepository, ErrorClass, RunOutcome, RunStatus};
use systemprompt::identifiers::UserId;
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
    clients::upsert_client(pool, &other, "Other", Some(42))
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
    agents::update_agent_run_as(&db.pool, &seeded.farm, seeded.agent.as_str(), Some(&svc))
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

async fn audit_odoo(pool: &sqlx::PgPool, user: &UserId, farm_run: &str, tool: &str) {
    let rules = serde_json::json!({
        "server": "odoo",
        "tool": tool,
        "model": "crm.lead",
        "record_ids": [7],
        "farm_run_id": farm_run,
    });
    sqlx::query(
        "INSERT INTO governance_decisions (id, user_id, session_id, tool_name, decision, policy, \
         reason, evaluated_rules, actor_kind, actor_id, context_id) \
         VALUES ($1, $2, 'sess', $3, 'allow', 'odoo_tool_audit', 'ok', $4, 'user', $2, 'ctx')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(user.as_str())
    .bind(format!("mcp__odoo__{tool}"))
    .bind(rules)
    .execute(pool)
    .await
    .expect("insert odoo audit row");
}

#[tokio::test]
async fn a_finished_run_gets_a_report_with_its_odoo_writes() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let tag = uuid::Uuid::new_v4().simple().to_string();
    let (own, _) = add_clients(&seeded, &tag).await;
    let run = launch_one(&seeded).await;
    audit_odoo(&db.pool, &seeded.user, run.id.as_str(), "crm_lead_update").await;
    audit_odoo(&db.pool, &seeded.user, run.id.as_str(), "crm_lead_search").await;
    audit_odoo(&db.pool, &seeded.user, "another-run", "crm_lead_delete").await;

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
    let writes = report.odoo_writes.as_array().expect("array");
    assert_eq!(writes.len(), 1, "only this run's write tools: {writes:?}");
    assert_eq!(writes[0]["tool"], "crm_lead_update");
    assert_eq!(writes[0]["record_ids"], serde_json::json!([7]));
    assert_eq!(report.outputs, serde_json::json!([]));

    let stamped = runs::find_run(&db.pool, &run.id)
        .await
        .expect("read run")
        .expect("run");
    assert_eq!(stamped.client_id.as_ref(), Some(&own));
    db.cleanup().await;
}
