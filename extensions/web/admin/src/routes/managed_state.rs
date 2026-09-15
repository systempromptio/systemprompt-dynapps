//! Managed-resource services constructed once at the admin composition root.

use sqlx::PgPool;
use systemprompt::marketplace::managed::ManagedRepository;

#[derive(Debug, Clone)]
pub(crate) struct ManagedState {
    pub(crate) owner: systemprompt::identifiers::UserId,
    pub(crate) repository: ManagedRepository,
    pub(crate) snapshots: systemprompt::analytics::snapshots::FeedbackSnapshotsRepository,
    pub(crate) portfolio:
        crate::repositories::analysis::portfolio_snapshots::PortfolioSnapshotsRepository,
}

impl ManagedState {
    pub(crate) fn new(pool: PgPool, owner: systemprompt::identifiers::UserId) -> Self {
        Self {
            owner,
            snapshots:systemprompt::analytics::snapshots::FeedbackSnapshotsRepository::new(pool.clone(), systemprompt::analytics::feedback::FeedbackFactsRepository::new(pool.clone())),
            portfolio:crate::repositories::analysis::portfolio_snapshots::PortfolioSnapshotsRepository::new(pool.clone()),
            repository: ManagedRepository::new(pool),
        }
    }
}
