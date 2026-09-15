//! Shared experiment repositories constructed once for the admin router.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::evaluation::experiments::{ExperimentRepository, RevisionRepository};
use systemprompt::evaluation::repository::experiments::{
    BudgetRepository, EvaluationLifecycleRepository, EvaluationRepositories, EvaluationSeams,
    EvidenceRepository, WorkerRepository,
};

#[derive(Debug)]
pub(crate) struct EvaluationState {
    pub(crate) optimization: systemprompt::system::optimization::SkillOptimizationOrchestrator,
    pub(crate) owner: systemprompt::identifiers::UserId,
    pub(crate) campaigns: systemprompt::evaluation::campaigns::repository::CampaignRepository,
    pub(crate) pool: PgPool,
    pub(crate) experiments: ExperimentRepository,
    pub(crate) workers: WorkerRepository,
    pub(crate) evidence: EvidenceRepository,
    pub(crate) revisions: RevisionRepository,
    pub(crate) budgets: BudgetRepository,
    pub(crate) lifecycle: EvaluationLifecycleRepository,
}

/// Why a state can fail to build: every core repository now opens its own
/// handles from the shared [`DbPool`], and each constructor reports that.
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error(transparent)]
    Managed(#[from] systemprompt::marketplace::managed::ManagedError),
    #[error(transparent)]
    Evaluation(#[from] systemprompt::evaluation::EvaluationError),
    #[error(transparent)]
    Ai(#[from] systemprompt::ai::error::RepositoryError),
    #[error(transparent)]
    Users(#[from] systemprompt::users::UserError),
}

impl EvaluationState {
    pub(crate) fn new(
        db: &DbPool,
        owner: systemprompt::identifiers::UserId,
    ) -> Result<Self, StateError> {
        let pool = db.write_pool().as_ref().clone();
        let managed = systemprompt::marketplace::managed::ManagedRepository::new(db)?;
        let seams = EvaluationSeams {
            trace: Arc::new(systemprompt::ai::repository::AiRequestRepository::new(db)?),
            sessions: Arc::new(
                systemprompt::users::UsersAiSessionProvider::from_repository(
                    systemprompt::users::SessionRepository::new(db)?,
                ),
            ),
            managed_revisions: Arc::new(managed.clone()),
        };
        let repositories = EvaluationRepositories::new(db, seams)?;
        Ok(Self {
            optimization: systemprompt::system::optimization::SkillOptimizationOrchestrator::new(
                managed,
                repositories.clone(),
                repositories.revisions.clone(),
            ),
            owner,
            campaigns: repositories.campaigns,
            pool,
            experiments: repositories.experiments,
            workers: repositories.workers,
            evidence: repositories.evidence,
            revisions: repositories.revisions,
            budgets: repositories.budgets,
            lifecycle: repositories.lifecycle,
        })
    }
}
