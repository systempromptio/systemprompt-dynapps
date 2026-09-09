use std::collections::BTreeMap;
use systemprompt::evaluation::experiments::resources::{
    CaseContent, Partition, ResourceContent, RubricContent, WeightedDimension,
};
use systemprompt::evaluation::experiments::{
    ExperimentRepository, ExperimentSpec, RevisionRepository,
};
use systemprompt::evaluation::repository::experiments::{
    AdmissionRequest, ExecutionLease, GatewayEvaluationRepository, RequestAdmission,
};
use systemprompt::identifiers::{AiRequestId, EvalWorkerId, ModelId, SessionId, UserId};

#[tokio::test]
async fn attested_evaluation_sessions_cannot_escape_budget_or_settle_incomplete_usage() {
    let db = astound_test_common::TempDb::create()
        .await
        .expect("PostgreSQL required");
    let owner = UserId::new("gateway-evaluation-owner");
    let worker = EvalWorkerId::generate();
    let session = SessionId::generate();
    sqlx::query(
        "INSERT INTO users(id,name,email) VALUES($1,'evaluator','eval-test@example.invalid')",
    )
    .bind(owner.as_str())
    .execute(&*db.pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO user_sessions(session_id,user_id) VALUES($1,$2)")
        .bind(session.as_str())
        .bind(owner.as_str())
        .execute(&*db.pool)
        .await
        .unwrap();
    let revisions = RevisionRepository::new((*db.pool).clone());
    let case = revisions
        .create(
            &owner,
            "case",
            &ResourceContent::Case(CaseContent {
                prompt: "Draft context".into(),
                expected_behavior: vec!["Cite the source".into()],
                fixtures: BTreeMap::new(),
                partition: Partition::Development,
            }),
        )
        .await
        .unwrap();
    let rubric = revisions
        .create(
            &owner,
            "rubric",
            &ResourceContent::Rubric(RubricContent {
                dimensions: vec![WeightedDimension {
                    name: "grounding".into(),
                    description: "Cited facts".into(),
                    weight: 1,
                }],
                pass_threshold_milli: 4000,
                hard_gates: vec![],
            }),
        )
        .await
        .unwrap();
    let spec: ExperimentSpec = serde_json::from_value(serde_json::json!({
        "schema_version":1,"name":"Gateway budget", "cases":[case], "rubric":rubric,
        "variants":[{"client":"claude-code","client_version":"pinned","model":"test-model","provider":"test-provider",
          "skill_bundle_digest":"a".repeat(64),"configuration_digest":"b".repeat(64),"worker_image_digest":"c".repeat(64)}],
        "repetitions":1,"budget_microdollars":100,"execution_mode":"fixture","objective":"quality"
    })).unwrap();
    let experiments = ExperimentRepository::new((*db.pool).clone());
    let id = experiments
        .create(&owner, "budget-case", &spec)
        .await
        .unwrap();
    let execution = experiments.claim(&owner, &worker).await.unwrap().unwrap();
    let lease = ExecutionLease {
        execution_id: execution.id,
        worker_id: worker.clone(),
        fencing_token: execution.fencing_token,
    };
    let gateway = GatewayEvaluationRepository::new((*db.pool).clone());
    gateway
        .bind_session(&owner, &lease, &session)
        .await
        .unwrap();
    let request = AiRequestId::generate();
    sqlx::query("INSERT INTO ai_requests(id,request_id,user_id,session_id,context_id,provider,model,actor_kind,actor_id) VALUES($1,$1,$2,$3,'test-context','test-provider','test-model','user',$2)")
        .bind(request.as_str()).bind(owner.as_str()).bind(session.as_str()).execute(&*db.pool).await.unwrap();
    let model = ModelId::new("test-model");
    let admission = AdmissionRequest {
        owner: &owner,
        session: &session,
        request: &request,
        model: &model,
        bound_microdollars: 80,
    };
    assert!(matches!(
        gateway.admit(&admission).await.unwrap(),
        RequestAdmission::Reserved(_)
    ));
    assert!(gateway.admit(&admission).await.is_err());
    let second = AiRequestId::generate();
    sqlx::query("INSERT INTO ai_requests(id,request_id,user_id,session_id,context_id,provider,model,actor_kind,actor_id) VALUES($1,$1,$2,$3,'test-context','test-provider','test-model','user',$2)")
        .bind(second.as_str()).bind(owner.as_str()).bind(session.as_str()).execute(&*db.pool).await.unwrap();
    assert!(matches!(
        gateway
            .admit(&AdmissionRequest {
                request: &second,
                ..admission
            })
            .await,
        Err(systemprompt::evaluation::EvaluationError::BudgetExhausted { .. })
    ));
    let wrong = ModelId::new("other-model");
    assert!(
        gateway
            .admit(&AdmissionRequest {
                model: &wrong,
                bound_microdollars: 1,
                ..admission
            })
            .await
            .is_err()
    );
    assert!(gateway.settle_recorded(&owner, &request).await.is_err());
    assert_eq!(
        experiments
            .get(&owner, &id)
            .await
            .unwrap()
            .experiment
            .accounting
            .reserved,
        80
    );
    sqlx::query("UPDATE ai_requests SET status='completed',cost_microdollars=30,tokens_used=1,completed_at=NOW() WHERE id=$1")
        .bind(request.as_str()).execute(&*db.pool).await.unwrap();
    assert!(gateway.settle_recorded(&owner, &request).await.unwrap());
    assert!(gateway.settle_recorded(&owner, &request).await.unwrap());
    let account = experiments
        .get(&owner, &id)
        .await
        .unwrap()
        .experiment
        .accounting;
    assert_eq!((account.reserved, account.settled), (0, 30));
    let evidence_repo = systemprompt::evaluation::repository::experiments::EvidenceRepository::new(
        (*db.pool).clone(),
    );
    let artifacts = systemprompt::evaluation::experiments::execution::FrozenWorkspace {
        files: BTreeMap::from([("response.md".into(), "hello".into())]),
    };
    let mut evidence: systemprompt::evaluation::experiments::execution::ExecutionEvidence = serde_json::from_value(serde_json::json!({
        "execution_id": lease.execution_id, "fencing_token": lease.fencing_token,
        "capabilities": {"client":"claude-code", "client_version":"pinned", "adapter_version":"1", "image_digest":"c".repeat(64),"supports_session_resume":false},
        "installed_bundle_digest":"a".repeat(64),"candidate_bundle_digest":"a".repeat(64),
        "workspace_digest": artifacts.digest().unwrap(), "requests":[request],
        "artifacts":[{"relative_path":"response.md","sha256":"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824","bytes":5}],
        "exit_code":0, "elapsed_milliseconds":1, "cleanup_confirmed":true
    })).unwrap();
    let mut tampered = artifacts.clone();
    tampered.files.insert("response.md".into(), "wrong".into());
    assert!(
        evidence_repo
            .submit(&owner, &lease, &evidence, &tampered)
            .await
            .is_err()
    );
    evidence.requests.push(second.clone());
    assert!(
        evidence_repo
            .submit(&owner, &lease, &evidence, &artifacts)
            .await
            .is_err()
    );
    evidence.requests.pop();
    evidence_repo
        .submit(&owner, &lease, &evidence, &artifacts)
        .await
        .unwrap();
    evidence_repo
        .submit(&owner, &lease, &evidence, &artifacts)
        .await
        .unwrap();
    assert_eq!(
        evidence_repo
            .get_artifacts(&owner, &lease.execution_id)
            .await
            .unwrap()
            .files,
        artifacts.files
    );
    assert!(
        evidence_repo
            .get_artifacts(&UserId::new("foreign-owner"), &lease.execution_id)
            .await
            .is_err()
    );
    evidence.elapsed_milliseconds += 1;
    assert!(
        evidence_repo
            .submit(&owner, &lease, &evidence, &artifacts)
            .await
            .is_err()
    );
    experiments.cancel(&owner, &id).await.unwrap();
    assert!(
        gateway
            .admit(&AdmissionRequest {
                request: &second,
                bound_microdollars: 1,
                ..admission
            })
            .await
            .is_err()
    );
    db.cleanup().await;
}
