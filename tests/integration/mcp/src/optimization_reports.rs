//! Database measurement metadata must not break campaign reports.

use systemprompt::evaluation::campaigns::{CampaignPolicy, OptimizationObjective, report};
use systemprompt::evaluation::repository::experiments::RevisionRepository;
use systemprompt::identifiers::{EvalBudgetId, EvalExperimentId, UserId};

pub(crate) async fn assert_retained_measurement_report(
    db: &astound_test_common::TempDb,
    owner: &UserId,
    experiment: &EvalExperimentId,
    budget: &EvalBudgetId,
) {
    sqlx::query("INSERT INTO eval_execution_measurements(execution_id,quality_milli,latency_ms,input_tokens,output_tokens,attempted_cost_microdollars,accounting_status,verified_success) SELECT id,4500,100,CASE WHEN variant_index=0 THEN 1000 ELSE 800 END,20,10,'complete',true FROM eval_executions WHERE experiment_id=$1")
        .bind(experiment.as_str()).execute(&*db.pool).await.unwrap();
    let repositories = crate::fixture_admission::evaluation_repositories(db);
    let (resource_id, baseline_revision_id) =
        crate::fixture_admission::seed_managed_baseline(db, owner, "report-baseline").await;
    let policy = CampaignPolicy {
        name: "Retained measurement report".to_owned(),
        resource_id,
        baseline_revision_id,
        budget_id: budget.clone(),
        objective: OptimizationObjective::Tokens,
        minimum_quality_milli: 4000,
        minimum_pairs: 2,
        maximum_iterations: 1,
        automatic: false,
    };
    let campaign = repositories
        .campaigns
        .create(owner, owner, "report", &policy)
        .await
        .unwrap();
    repositories
        .campaigns
        .attach_experiment(owner, owner, &campaign, experiment)
        .await
        .unwrap();
    let revisions = RevisionRepository::new((*db.pool).clone());
    let measured = report::build(&repositories, &revisions, owner, &campaign, experiment)
        .await
        .unwrap();
    assert!(!measured.eligible_for_publication);
    assert_eq!(measured.development.pairs, 1);
    assert!(measured.development.mean_improvement.is_none());
    assert!(measured.holdout.mean_improvement.is_none());
    assert!(
        !measured
            .limitations
            .iter()
            .any(|message| message.contains("incomplete outcome"))
    );
    assert!(
        measured
            .limitations
            .iter()
            .any(|message| message.contains("fresh independent holdout"))
    );
}
