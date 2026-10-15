//! Pruning a farm `agent-farms.yaml` no longer declares, against a real
//! schema: the farm, its agents and run rows go; the spend record stays
//! whole. `ai_requests`, `governance_decisions` and core's `agent_runs` keep
//! every row, farm totals drop the pruned runs cleanly (no nulls, no mixed
//! totals), and a declared farm and a farm with a run in flight are kept.

use std::sync::Arc;

use chrono::{Duration, Utc};
use sqlx::PgPool;
use systemprompt::agent_runs::{AgentRunRepository, ErrorClass, RunOutcome, RunStatus};
use systemprompt_web_admin_farm::repositories::filters::RunFilter;
use systemprompt_web_admin_farm::repositories::{farms, runs};
use systemprompt_web_admin_farm::services::farm_sync::seed_farms_from_yaml;

use crate::farm_runner_lifecycle::{Seeded, launch_one, observe_finish, seed};
use crate::tempdb::TempDb;

const COST: i64 = 1_500;

fn outcome() -> RunOutcome {
    RunOutcome {
        status: RunStatus::Succeeded,
        error_class: ErrorClass::Ok,
        session_id: None,
        cost_microdollars: COST,
        input_tokens: 10,
        output_tokens: 5,
        turns: 1,
        duration_ms: 100,
        structured_output: None,
        exit_code: Some(0),
        error: None,
        final_text: None,
    }
}

async fn spend_row(pool: &PgPool, seeded: &Seeded, id: &str) {
    sqlx::query(
        "INSERT INTO ai_requests (id, request_id, user_id, session_id, trace_id, context_id, \
         provider, model, input_tokens, output_tokens, tokens_used, cost_microdollars, \
         latency_ms, status, actor_kind, actor_id, created_at, updated_at) \
         VALUES ($1, $1, $2, NULL, 'trace', '00000000-0000-4000-8000-000000000001', \
         'anthropic', 'claude-opus-5-5', 10, 5, 15, $3, 100, 'completed', 'user', $2, NOW(), NOW())",
    )
    .bind(id)
    .bind(seeded.user.as_str())
    .bind(COST)
    .execute(pool)
    .await
    .expect("insert ai request");
    sqlx::query(
        "INSERT INTO governance_decisions (id, user_id, session_id, tool_name, decision, policy, \
         reason, actor_kind, actor_id, context_id) \
         VALUES ($1, $2, 'sess', 'mcp__dynapps-odoo__orm_search_read', 'allow', 'scope_check', \
         'r', 'user', $2, 'ctx')",
    )
    .bind(format!("gd-{id}"))
    .bind(seeded.user.as_str())
    .execute(pool)
    .await
    .expect("insert governance decision");
}

async fn count(pool: &PgPool, sql: &'static str, user: &str) -> i64 {
    sqlx::query_scalar(sql)
        .bind(user)
        .fetch_one(pool)
        .await
        .expect("count")
}

fn all_time() -> RunFilter {
    RunFilter {
        since: Some(Utc::now() - Duration::days(1)),
        ..RunFilter::default()
    }
}

async fn seed_only(pool: &Arc<PgPool>, declared: &str) {
    let services = tempfile::tempdir().expect("tempdir");
    let config = services.path().join("web/config");
    std::fs::create_dir_all(&config).expect("mkdir");
    let yaml = format!(
        "farms:\n  - id: {declared}\n    name: Kept\n    marketplace: kept\n    source:\n      github: dynapps/agent-farm\n"
    );
    std::fs::write(config.join("agent-farms.yaml"), yaml).expect("write yaml");
    seed_farms_from_yaml(pool, services.path())
        .await
        .expect("seed");
}

#[tokio::test]
async fn pruning_a_farm_keeps_spend_and_drops_its_runs_from_farm_totals() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let stale = seed(&db.pool).await;
    let busy = seed(&db.pool).await;
    let finished = launch_one(&stale).await;
    observe_finish(&stale.state, &finished, &outcome()).await;
    spend_row(&db.pool, &stale, &format!("req-{}", finished.id)).await;
    let in_flight = launch_one(&busy).await;

    let user = stale.user.as_str();
    let requests = "SELECT COUNT(*) FROM ai_requests WHERE user_id = $1";
    let decisions = "SELECT COUNT(*) FROM governance_decisions WHERE user_id = $1";
    let core_runs = "SELECT COUNT(*) FROM agent_runs WHERE user_id = $1";
    let spend =
        "SELECT COALESCE(SUM(cost_microdollars), 0)::bigint FROM ai_requests WHERE user_id = $1";
    let before = (
        count(&db.pool, requests, user).await,
        count(&db.pool, decisions, user).await,
        count(&db.pool, core_runs, user).await,
        count(&db.pool, spend, user).await,
    );
    let stats = runs::get_run_stats(&db.pool, &all_time())
        .await
        .expect("stats");
    assert!(
        stats.cost_microdollars >= COST,
        "the finished run is counted"
    );

    seed_only(&db.pool, "declared-farm").await;

    assert!(
        farms::find_farm(&db.pool, &stale.farm)
            .await
            .expect("read")
            .is_none(),
        "the undeclared, idle farm is pruned"
    );
    assert!(
        runs::find_run(&db.pool, &finished.id)
            .await
            .expect("read")
            .is_none(),
        "its farm runs go with it"
    );
    assert!(
        farms::find_farm(&db.pool, &busy.farm)
            .await
            .expect("read")
            .is_some(),
        "a farm with a run in flight is kept"
    );
    assert!(
        runs::find_run(&db.pool, &in_flight.id)
            .await
            .expect("read")
            .is_some()
    );
    assert!(
        farms::find_farm(
            &db.pool,
            &systemprompt_web_admin_farm::types::FarmId::new("declared-farm")
        )
        .await
        .expect("read")
        .is_some()
    );

    let after = (
        count(&db.pool, requests, user).await,
        count(&db.pool, decisions, user).await,
        count(&db.pool, core_runs, user).await,
        count(&db.pool, spend, user).await,
    );
    assert_eq!(
        before, after,
        "spend, governance and core run rows are untouched"
    );

    let pruned = runs::get_run_stats(&db.pool, &all_time())
        .await
        .expect("stats");
    assert_eq!(pruned.total, stats.total - 1, "the pruned run drops out");
    assert_eq!(pruned.cost_microdollars, stats.cost_microdollars - COST);
    let core = AgentRunRepository::from_pool(Arc::clone(&db.pool))
        .find_run(&finished.agent_run_id)
        .await
        .expect("read core run");
    assert!(core.is_some(), "core's agent run outlives the farm row");
    assert!(
        runs::find_run_by_agent_run(&db.pool, &finished.agent_run_id)
            .await
            .expect("a dangling agent run reads as no farm run, not an error")
            .is_none()
    );
    db.cleanup().await;
}
