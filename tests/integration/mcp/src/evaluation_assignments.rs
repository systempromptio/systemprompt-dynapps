use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use std::collections::BTreeMap;
use systemprompt::evaluation::experiments::{ExperimentSpec, content_digest};
use systemprompt::evaluation::repository::experiments::{
    ManagedWorkspaceRegistration, RevisionRepository, WorkerRepository,
};
use systemprompt::identifiers::{ResourceRevisionId, SourceSnapshotId, UserId};
use systemprompt::models::managed::{
    AssetDigest, AssetFile, RevisionBundle, RevisionFiles, RevisionManifest,
};
use systemprompt_api::routes::evaluation::{EvaluationWorkerState, router};
use tower::ServiceExt;

use crate::fixture_admission::evaluation_repositories;

fn managed_workspace(marker: &str) -> (RevisionBundle, String) {
    let bytes = marker.as_bytes().to_vec();
    let asset_digest = AssetDigest::of(&bytes);
    let files = RevisionFiles(BTreeMap::from([(
        "file.txt".to_owned(),
        AssetFile {
            bytes: bytes.clone(),
            media_type: "text/plain".to_owned(),
            executable: false,
        },
    )]));
    let revision = ResourceRevisionId::new(format!("revision-{marker}"));
    let manifest = RevisionManifest::from_files(
        SourceSnapshotId::new(format!("snapshot-{marker}")),
        None,
        &files,
        BTreeMap::new(),
    )
    .expect("fixture revision from retained bytes and modes");
    let bundle = RevisionBundle {
        schema_version: 1,
        assembler_version: "managed-bundle-v1".to_owned(),
        root: revision.clone(),
        revisions: BTreeMap::from([(revision.clone(), manifest)]),
        assets: BTreeMap::from([(asset_digest, bytes)]),
    };
    bundle.verify().expect("complete immutable fixture bundle");
    let restored = bundle
        .revision_files(&revision)
        .expect("verified retained files");
    assert_eq!(restored.0["file.txt"].bytes, marker.as_bytes());
    assert!(!restored.0["file.txt"].executable);
    let digest = bundle
        .digest()
        .expect("typed canonical bundle digest")
        .as_str()
        .to_owned();
    (bundle, digest)
}

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
        "kind":"case", "content":{"prompt":"Draft grounded context", "expected_behavior":["Cite sources"],"fixtures":{},"partition":"development","assertions":["response_present"]}
    })).unwrap();
    let case = revisions
        .create(&owner, "case", &case_content)
        .await
        .unwrap();
    let rubric = revisions.create(&owner, "rubric", &serde_json::from_value(serde_json::json!({
        "kind":"rubric", "content":{"dimensions":[{"name":"grounding","description":"Cited facts","weight":1}],"pass_threshold_milli":4000,"hard_gates":[]}
    })).unwrap()).await.unwrap();
    let dataset_content: systemprompt::evaluation::experiments::resources::ResourceContent =
        serde_json::from_value(serde_json::json!({
            "kind":"dataset", "content":[case]
        }))
        .unwrap();
    let dataset = revisions
        .create(&owner, "dataset", &dataset_content)
        .await
        .unwrap();
    let (bundle, bundle_digest) = managed_workspace("bundle");
    let (candidate, candidate_digest) = managed_workspace("candidate");
    let (config, config_digest) = managed_workspace("configuration");
    let repositories = evaluation_repositories(&db);
    let evidence = repositories.evidence.clone();
    for (revision, manifest, digest, bytes) in [
        ("revision-bundle", &bundle, &bundle_digest, 6),
        ("revision-candidate", &candidate, &candidate_digest, 9),
        ("revision-configuration", &config, &config_digest, 13),
    ] {
        evidence
            .register_managed_workspace(
                &owner,
                &ManagedWorkspaceRegistration {
                    managed_revision_id: &ResourceRevisionId::new(revision),
                    publication_generation: Some(1),
                    manifest,
                    expected_digest: digest,
                    file_count: 1,
                    byte_count: bytes,
                },
            )
            .await
            .unwrap();
    }
    let spec: ExperimentSpec = serde_json::from_value(serde_json::json!({
        "schema_version":1,"name":"Assignments", "cases":[case],"rubric":rubric,"dataset":dataset,
        "variants":[{"client":"claude-code","client_version":"pinned","model":"test-model","provider":"test-provider",
        "skill_bundle_digest":bundle_digest,"configuration_digest":config_digest,"worker_image_digest":"c".repeat(64)},
        {"client":"claude-code","client_version":"pinned","model":"test-model","provider":"test-provider",
        "skill_bundle_digest":candidate_digest,"configuration_digest":config_digest,"worker_image_digest":"c".repeat(64)}],
        "repetitions":1,"budget_microdollars":100,"execution_mode":"fixture","objective":"quality",
        "frozen":{"provider_prices_digest":"d".repeat(64),"tool_configuration_digest":"e".repeat(64),
          "fixture_clock":"2026-09-12T08:00:00Z","fixture_timezone":"UTC","permissions_digest":"f".repeat(64),
          "dataset_digest":content_digest(&dataset_content).unwrap(),"rubric_digest":content_digest(&revisions.get(&owner, &rubric).await.unwrap()).unwrap(),
          "cost_envelope":{"maximum_attempts_per_execution":1,"generation_microdollars_per_attempt":25,
            "judging_microdollars_per_attempt":25,"tool_microdollars_per_attempt":0,"suggestion_calls":0,
            "suggestion_microdollars_per_call":0,"auxiliary_calls":0,"auxiliary_microdollars_per_call":0}}
    })).unwrap();
    // Why: production admission demands a reviewed native proof for the
    // variant's exact client/version/image, and this harness ships none; the
    // assignment fences under test are independent of that gate.
    let experiments = repositories.experiments.clone();
    let budget = repositories
        .budgets
        .create_shared(&owner, "assignment-budget", 100)
        .await
        .unwrap();
    let experiment = experiments
        .create_with_budget(&owner, "assignment", &budget, &spec)
        .await
        .unwrap();
    let execution = experiments
        .claim(&owner, &worker.id)
        .await
        .unwrap()
        .unwrap();
    let lease = serde_json::json!({"execution_id":execution.id,"worker_id":worker.id,"fencing_token":execution.fencing_token});
    let app = router(
        EvaluationWorkerState::builder(repositories.clone())
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
    assert_eq!(
        body["skill_bundle"]["manifest"],
        serde_json::to_value(&bundle).unwrap()
    );
    assert_eq!(
        body["skill_bundle"]["digest"],
        bundle.digest().unwrap().as_str()
    );
    assert_eq!(
        body["configuration"]["manifest"],
        serde_json::to_value(&config).unwrap()
    );
    assert_eq!(
        body["configuration"]["digest"],
        config.digest().unwrap().as_str()
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
    sqlx::query!("ALTER TABLE eval_managed_workspace_assets DISABLE TRIGGER eval_managed_workspace_assets_immutable").execute(&*db.pool).await.unwrap();
    sqlx::query!("UPDATE eval_managed_workspace_assets SET content=$3 WHERE owner_id=$1 AND workspace_digest=$2", owner.as_str(), config_digest, b"tampered".as_slice()).execute(&*db.pool).await.unwrap();
    sqlx::query!("ALTER TABLE eval_managed_workspace_assets ENABLE TRIGGER eval_managed_workspace_assets_immutable").execute(&*db.pool).await.unwrap();
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
