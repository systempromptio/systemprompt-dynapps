//! Managed-resource services constructed once at the admin composition root.

use sqlx::PgPool;
use systemprompt::marketplace::managed::ManagedRepository;

#[derive(Debug, Clone)]
pub(crate) struct ManagedState {
    pub(crate) repository: ManagedRepository,
}

impl ManagedState {
    pub(crate) const fn new(pool: PgPool) -> Self {
        Self {
            repository: ManagedRepository::new(pool),
        }
    }
}
