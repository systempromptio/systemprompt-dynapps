//! Shared experiment repositories constructed once for the admin router.

use sqlx::PgPool;
use systemprompt::evaluation::experiments::{ExperimentRepository, RevisionRepository};

#[derive(Debug)]
pub(crate) struct EvaluationState {
    pub(crate) experiments: ExperimentRepository,
    pub(crate) revisions: RevisionRepository,
}

impl EvaluationState {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self {
            experiments: ExperimentRepository::new(pool.clone()),
            revisions: RevisionRepository::new(pool),
        }
    }
}
