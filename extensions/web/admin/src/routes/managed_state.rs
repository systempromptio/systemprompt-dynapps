//! Managed-resource services constructed once at the admin composition root.

use systemprompt::database::DbPool;
use systemprompt::marketplace::managed::ManagedRepository;

use super::evaluation_state::StateError;

#[derive(Debug, Clone)]
pub(crate) struct ManagedState {
    pub(crate) owner: systemprompt::identifiers::UserId,
    pub(crate) repository: ManagedRepository,
    pub(crate) snapshots: systemprompt::analytics::snapshots::FeedbackSnapshotsRepository,
    pub(crate) portfolio:
        crate::repositories::analysis::portfolio_snapshots::PortfolioSnapshotsRepository,
}

impl ManagedState {
    pub(crate) fn new(
        db: &DbPool,
        owner: systemprompt::identifiers::UserId,
    ) -> Result<Self, StateError> {
        let pool = db.write_pool().as_ref().clone();
        Ok(Self {
            owner,
            snapshots:systemprompt::analytics::snapshots::FeedbackSnapshotsRepository::new(pool.clone(), systemprompt::analytics::feedback::FeedbackFactsRepository::new(pool.clone())),
            portfolio:crate::repositories::analysis::portfolio_snapshots::PortfolioSnapshotsRepository::new(pool),
            repository: ManagedRepository::new(db)?,
        })
    }
}
