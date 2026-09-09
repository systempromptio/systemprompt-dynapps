use std::collections::BTreeMap;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use systemprompt::evaluation::experiments::ExperimentSpec;
use systemprompt::evaluation::experiments::execution::FrozenWorkspace;
use systemprompt::evaluation::repository::experiments::{
    EvidenceRepository, ExperimentRepository, RevisionRepository, WorkerRepository,
};
use systemprompt::identifiers::UserId;
use systemprompt_api::routes::evaluation::{EvaluationWorkerState, router};
use tower::ServiceExt;

async fn post(
    app: &Router,
    token: &str,
    path: &str,
    value: serde_json::Value,
) -> axum::response::Response {
    app.clone()
        .oneshot(
            Request::post(path)
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(value.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn evaluation_assignments_and_events_are_fenced_owner_scoped_and_immutable() {
    let db = astound_test_common::TempDb::create()
        .await
        .expect("PostgreSQL required");
    let owner = UserId::new("assignment-owner");
    let stranger = UserId::new("assignment-stranger");
    for user in [&owner, &stranger] {
        sqlx::query("INSERT INTO users(id,name,email) VALUES($1,$1,$2)")
            .bind(user.as_str())
            .bind(format!("{user}@example.invalid"))
            .execute(&*db.pool)
            .await
            .unwrap();
    }
    let workers = WorkerRepository::new((*db.pool).clone());
    let worker = workers
        .create(&owner, "assignment-test", "worker")
        .await
        .unwrap();
    let foreign = workers
        .create(&stranger, "assignment-test", "foreign")
        .await
        .unwrap();
    let peer = workers
        .create(&owner, "assignment-test", "peer")
        .await
        .unwrap();
    let revisions = RevisionRepository::new((*db.pool).clone());
    let case_content = serde_json::from_value(serde_json::json!({
        "kind":"case", "content":{"prompt":"Draft grounded context", "expected_behavior":["Cite sources"],"fixtures":{},"partition":"development"}
    })).unwrap();
    let case = revisions
        .create(&owner, "case", &case_content)
        .await
        .unwrap();
    let rubric = revisions.create(&owner, "rubric", &serde_json::from_value(serde_json::json!({
        "kind":"rubric", "content":{"dimensions":[{"name":"grounding","description":"Cited facts","weight":1}],"pass_threshold_milli":4000,"hard_gates":[]}
    })).unwrap()).await.unwrap();
    let evidence = EvidenceRepository::new((*db.pool).clone());
    let bundle = FrozenWorkspace {
        files: BTreeMap::from([(
            "requirements_context/SKILL.md".into(),
            "Ground claims in source evidence".into(),
        )]),
    };
    let config = FrozenWorkspace {
        files: BTreeMap::from([("client.json".into(), "{}".into())]),
    };
    let bundle_digest = evidence.save_workspace(&owner, &bundle).await.unwrap();
    let config_digest = evidence.save_workspace(&owner, &config).await.unwrap();
    let spec: ExperimentSpec = serde_json::from_value(serde_json::json!({
        "schema_version":1,"name":"Assignments", "cases":[case],"rubric":rubric,
        "variants":[{"client":"claude-code","client_version":"pinned","model":"test-model","provider":"test-provider",
        "skill_bundle_digest":bundle_digest,"configuration_digest":config_digest,"worker_image_digest":"c".repeat(64)}],
        "repetitions":1,"budget_microdollars":100,"execution_mode":"fixture","objective":"quality"
    })).unwrap();
    let experiments = ExperimentRepository::new((*db.pool).clone());
    let experiment = experiments
        .create(&owner, "assignment", &spec)
        .await
        .unwrap();
    let execution = experiments
        .claim(&owner, &worker.id)
        .await
        .unwrap()
        .unwrap();
    let lease = serde_json::json!({"execution_id":execution.id,"worker_id":worker.id,"fencing_token":execution.fencing_token});
    let app = router(
        EvaluationWorkerState::builder((*db.pool).clone())
            .environment("assignment-test".into())
            .build()
            .unwrap(),
    );
    let token = worker.expose_token();
    let response = post(&app, token, "/assignment", lease.clone()).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let body: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 1_000_000).await.unwrap()).unwrap();
    assert_eq!(body["skill_bundle"], serde_json::to_value(&bundle).unwrap());
    assert_eq!(
        body["configuration"],
        serde_json::to_value(&config).unwrap()
    );
    assert_eq!(body["case"], serde_json::to_value(&case_content).unwrap());
    for unauthorized in [foreign.expose_token(), peer.expose_token(), "invalid"] {
        assert_eq!(
            post(&app, unauthorized, "/assignment", lease.clone())
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
    let mut forged = lease.clone();
    forged["worker_id"] = serde_json::json!(foreign.id);
    assert_eq!(
        post(&app, foreign.expose_token(), "/assignment", forged)
            .await
            .status(),
        StatusCode::CONFLICT
    );
    let mut stale = lease.clone();
    stale["fencing_token"] = serde_json::json!(execution.fencing_token + 1);
    assert_eq!(
        post(&app, token, "/assignment", stale.clone())
            .await
            .status(),
        StatusCode::CONFLICT
    );
    let event = serde_json::json!({"lease":lease,"event":{"sequence":0,"stage":"context","summary":"Sources retrieved"}});
    assert_eq!(
        post(&app, token, "/events", event.clone()).await.status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        post(&app, token, "/events", event.clone()).await.status(),
        StatusCode::NO_CONTENT
    );
    let mut changed = event.clone();
    changed["event"]["summary"] = "Changed".into();
    assert_eq!(
        post(&app, token, "/events", changed).await.status(),
        StatusCode::CONFLICT
    );
    let mut skipped = event.clone();
    skipped["event"]["sequence"] = 2.into();
    assert_eq!(
        post(&app, token, "/events", skipped).await.status(),
        StatusCode::CONFLICT
    );
    let mut stale_event = event.clone();
    stale_event["lease"] = stale;
    assert_eq!(
        post(&app, token, "/events", stale_event).await.status(),
        StatusCode::CONFLICT
    );
    let mut approval = event.clone();
    approval["event"]["approved"] = true.into();
    assert_eq!(
        post(&app, token, "/events", approval).await.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    sqlx::query("UPDATE eval_resource_revisions SET digest=$2 WHERE id=$1")
        .bind(case.as_str())
        .bind("0".repeat(64))
        .execute(&*db.pool)
        .await
        .unwrap();
    assert_eq!(
        post(&app, token, "/assignment", lease.clone())
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    sqlx::query("UPDATE eval_resource_revisions SET digest=$2 WHERE id=$1")
        .bind(case.as_str())
        .bind(body["case_digest"].as_str().unwrap())
        .execute(&*db.pool)
        .await
        .unwrap();
    sqlx::query("UPDATE eval_frozen_workspaces SET content=$2 WHERE owner_id=$1 AND digest=$3")
        .bind(owner.as_str())
        .bind(serde_json::json!({"files":{"client.json":"tampered"}}))
        .bind(&config_digest)
        .execute(&*db.pool)
        .await
        .unwrap();
    assert_eq!(
        post(&app, token, "/assignment", lease.clone())
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    experiments.cancel(&owner, &experiment).await.unwrap();
    assert_eq!(
        post(&app, token, "/assignment", lease.clone())
            .await
            .status(),
        StatusCode::CONFLICT
    );
    assert_eq!(
        post(&app, token, "/events", event.clone()).await.status(),
        StatusCode::CONFLICT
    );
    workers.revoke(&owner, &worker.id).await.unwrap();
    assert_eq!(
        post(&app, token, "/assignment", lease).await.status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        post(&app, token, "/events", event).await.status(),
        StatusCode::UNAUTHORIZED
    );
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM eval_execution_events WHERE execution_id=$1")
            .bind(execution.id.as_str())
            .fetch_one(&*db.pool)
            .await
            .unwrap();
    assert_eq!(count, 1);
    db.cleanup().await;
}
