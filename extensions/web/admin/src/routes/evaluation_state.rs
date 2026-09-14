//! Shared experiment repositories constructed once for the admin router.

use sqlx::PgPool;
use systemprompt::evaluation::experiments::{ExperimentRepository, RevisionRepository};
use systemprompt::evaluation::repository::experiments::{
    BudgetRepository, EvaluationLifecycleRepository, EvidenceRepository, WorkerRepository,
};

#[derive(Debug)]
pub(crate) struct EvaluationState {
    pub(crate) pool: PgPool,
    pub(crate) experiments: ExperimentRepository,
    pub(crate) workers: WorkerRepository,
    pub(crate) evidence: EvidenceRepository,
    pub(crate) revisions: RevisionRepository,
    pub(crate) budgets: BudgetRepository,
    pub(crate) lifecycle: EvaluationLifecycleRepository,
}

impl EvaluationState {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self {
            pool: pool.clone(),
            experiments: ExperimentRepository::new(pool.clone()),
            workers: WorkerRepository::new(pool.clone()),
            evidence: EvidenceRepository::new(pool.clone()),
            revisions: RevisionRepository::new(pool.clone()),
            budgets: BudgetRepository::new(pool.clone()),
            lifecycle: EvaluationLifecycleRepository::new(pool),
        }
    }
}
