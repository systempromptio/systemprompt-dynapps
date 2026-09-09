use std::collections::BTreeMap;
use systemprompt::evaluation::experiments::resources::{
    CaseContent, Partition, ResourceContent, RubricContent, WeightedDimension,
};
use systemprompt::evaluation::experiments::{
    BudgetRepository, ExperimentRepository, ExperimentSpec, ReservationAdmission,
    RevisionRepository,
};
use systemprompt::identifiers::{AiRequestId, UserId};

#[tokio::test]
async fn evaluation_budget_admission_is_atomic_and_settlement_is_idempotent() {
    let db = astound_test_common::TempDb::create_empty()
        .await
        .expect("PostgreSQL required");
    sqlx::raw_sql(include_str!(
        "../../../../.vendor/systemprompt-core/crates/domain/evaluation/schema/experiments.sql"
    ))
    .execute(&*db.pool)
    .await
    .unwrap();
    let repository = BudgetRepository::new((*db.pool).clone());
    let owner = UserId::new("budget-owner");
    let stranger = UserId::new("other-owner");
    let account = repository.create(&owner, 5_000_000).await.unwrap();
    let (left, right) = tokio::join!(
        repository.reserve(&owner, &account, "left", 3_000_000),
        repository.reserve(&owner, &account, "right", 3_000_000)
    );
    assert_ne!(left.is_ok(), right.is_ok());
    let ReservationAdmission::Admitted(reservation) = left.or(right).unwrap() else {
        panic!("New reservation required");
    };
    assert!(
        repository
            .reserve(&stranger, &account, "foreign", 1)
            .await
            .is_err()
    );
    let request = AiRequestId::generate();
    repository
        .settle(&owner, &reservation, &request, 1_000_000)
        .await
        .unwrap();
    repository
        .settle(&owner, &reservation, &request, 1_000_000)
        .await
        .unwrap();
    assert!(
        repository
            .settle(&owner, &reservation, &request, 2_000_000)
            .await
            .is_err()
    );
    let (reserved, settled): (i64, i64) =
        sqlx::query_as("SELECT reserved,settled FROM eval_budget_accounts WHERE id=$1")
            .bind(account.as_str())
            .fetch_one(&*db.pool)
            .await
            .unwrap();
    assert_eq!((reserved, settled), (0, 1_000_000));
    let ReservationAdmission::Admitted(overage) = repository
        .reserve(&owner, &account, "overage", 1)
        .await
        .unwrap()
    else {
        panic!("New reservation required");
    };
    assert!(matches!(
        repository
            .reserve(&owner, &account, "overage", 1)
            .await
            .unwrap(),
        ReservationAdmission::AlreadyReserved(_)
    ));
    repository
        .settle(&owner, &overage, &AiRequestId::generate(), 2)
        .await
        .unwrap();
    assert!(
        repository
            .reserve(&owner, &account, "after-overage", 1)
            .await
            .is_err()
    );
    db.cleanup().await;
}

#[tokio::test]
async fn evaluation_experiments_pin_revisions_and_reject_cross_owner_access() {
    let db = astound_test_common::TempDb::create_empty()
        .await
        .expect("PostgreSQL required");
    sqlx::raw_sql(include_str!(
        "../../../../.vendor/systemprompt-core/crates/domain/evaluation/schema/experiments.sql"
    ))
    .execute(&*db.pool)
    .await
    .unwrap();
    let owner = UserId::new("experiment-owner");
    let stranger = UserId::new("other-owner");
    let revisions = RevisionRepository::new((*db.pool).clone());
    let case = ResourceContent::Case(CaseContent {
        prompt: "Write a specification".into(),
        expected_behavior: vec!["Cite source".into()],
        fixtures: BTreeMap::new(),
        partition: Partition::Development,
    });
    let case_id = revisions
        .create(&owner, "requirements-context", &case)
        .await
        .unwrap();
    assert_eq!(
        case_id,
        revisions
            .create(&owner, "requirements-context", &case)
            .await
            .unwrap()
    );
    assert!(revisions.get(&stranger, &case_id).await.is_err());
    let rubric = ResourceContent::Rubric(RubricContent {
        dimensions: vec![WeightedDimension {
            name: "grounding".into(),
            description: "Source-supported claims".into(),
            weight: 1,
        }],
        pass_threshold_milli: 4000,
        hard_gates: vec![],
    });
    let rubric_id = revisions
        .create(&owner, "requirements-quality", &rubric)
        .await
        .unwrap();
    let spec: ExperimentSpec = serde_json::from_value(serde_json::json!({"schema_version":1,"name":"UK POC","cases":[case_id],"rubric":rubric_id,"variants":[{"client":"claude-code","client_version":"pinned","model":"claude-haiku-4-5","provider":"anthropic","skill_bundle_digest":"a".repeat(64),"configuration_digest":"b".repeat(64),"worker_image_digest":"c".repeat(64)}],"repetitions":1,"budget_microdollars":5_000_000,"execution_mode":"fixture","objective":"quality"})).unwrap();
    let experiments = ExperimentRepository::new((*db.pool).clone());
    let id = experiments.create(&owner, "poc", &spec).await.unwrap();
    assert_eq!(id, experiments.create(&owner, "poc", &spec).await.unwrap());
    assert_eq!(
        experiments.get(&owner, &id).await.unwrap().executions.len(),
        1
    );
    assert!(experiments.get(&stranger, &id).await.is_err());
    assert!(experiments.create(&stranger, "poc", &spec).await.is_err());
    let mut changed = spec.clone();
    changed.name = "Changed".into();
    assert!(experiments.create(&owner, "poc", &changed).await.is_err());
    let claimed = experiments.claim(&owner, &owner).await.unwrap().unwrap();
    use systemprompt::evaluation::repository::experiments::{
        ExecutionCompletion, ExecutionLease, TerminalOutcome,
    };
    let lease = ExecutionLease {
        execution_id: claimed.id,
        worker_id: owner.clone(),
        fencing_token: claimed.fencing_token,
    };
    experiments.heartbeat(&owner, &lease).await.unwrap();
    let completion = ExecutionCompletion {
        outcome: TerminalOutcome::Completed,
        summary: "Fixture completed; semantic judgment remains separate".into(),
    };
    let mut stale = lease.clone();
    stale.fencing_token += 1;
    assert!(
        experiments
            .complete(&owner, &stale, &completion)
            .await
            .is_err()
    );
    experiments
        .complete(&owner, &lease, &completion)
        .await
        .unwrap();
    experiments
        .complete(&owner, &lease, &completion)
        .await
        .unwrap();
    assert!(experiments.heartbeat(&owner, &lease).await.is_err());
    let another = experiments
        .create(&owner, "cancel-test", &spec)
        .await
        .unwrap();
    experiments.cancel(&owner, &another).await.unwrap();
    assert!(experiments.claim(&owner, &owner).await.unwrap().is_none());
    let expired_id = experiments
        .create(&owner, "expiry-test", &spec)
        .await
        .unwrap();
    let expired = experiments.claim(&owner, &owner).await.unwrap().unwrap();
    sqlx::query(
        "UPDATE eval_executions SET lease_expires_at=NOW()-INTERVAL '1 second' WHERE id=$1",
    )
    .bind(expired.id.as_str())
    .execute(&*db.pool)
    .await
    .unwrap();
    assert!(experiments.claim(&owner, &owner).await.unwrap().is_none());
    let expired_detail = experiments.get(&owner, &expired_id).await.unwrap();
    assert_eq!(
        serde_json::to_value(expired_detail.experiment.status).unwrap(),
        "completed"
    );
    assert_eq!(
        serde_json::to_value(expired_detail.executions[0].status).unwrap(),
        "error"
    );
    db.cleanup().await;
}
