//! Managed-resource services constructed once at the admin composition root.

use std::path::PathBuf;

use systemprompt::database::DbPool;
use systemprompt::marketplace::managed::ManagedRepository;

#[derive(Debug, Clone)]
pub struct ManagedState {
    pub owner: systemprompt::identifiers::UserId,
    pub repository: ManagedRepository,
    pub otlp_export: systemprompt::scheduler::OtlpExportStateRepository,
    pub scratch_root: Option<PathBuf>,
}

impl ManagedState {
    pub fn new(
        db: &DbPool,
        owner: systemprompt::identifiers::UserId,
        scratch_root: Option<PathBuf>,
    ) -> Self {
        let pool = db.write_pool().as_ref().clone();
        Self {
            owner,
            scratch_root,
            otlp_export: systemprompt::scheduler::OtlpExportStateRepository::new(pool),
            repository: ManagedRepository::new(db),
        }
    }
}
