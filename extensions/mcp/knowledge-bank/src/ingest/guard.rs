//! Single flight per caller: the set of users whose sync is in progress.

use crate::error::KnowledgeBankError;
use std::collections::HashSet;
use std::sync::Mutex;
use systemprompt::identifiers::UserId;

// Why: a std mutex, never held across an await, guarding one set for every
// caller rather than a lock per user, because callers are unbounded. The
// guard removes its entry on drop so a failed run cannot wedge that user's
// syncs until restart.
pub(super) struct RunGuard<'a> {
    running: &'a Mutex<HashSet<UserId>>,
    user_id: UserId,
}

impl<'a> RunGuard<'a> {
    pub(super) fn acquire(
        running: &'a Mutex<HashSet<UserId>>,
        user_id: UserId,
    ) -> Result<Self, KnowledgeBankError> {
        if !lock_running(running).insert(user_id.clone()) {
            return Err(KnowledgeBankError::Backend(
                "your knowledge sync is already in progress".to_owned(),
            ));
        }
        Ok(Self { running, user_id })
    }
}

impl Drop for RunGuard<'_> {
    fn drop(&mut self) {
        lock_running(self.running).remove(&self.user_id);
    }
}

// Why: the set holds no invariant a panic could break, so a poisoned lock is
// recovered rather than propagated.
pub(super) fn lock_running(
    running: &Mutex<HashSet<UserId>>,
) -> std::sync::MutexGuard<'_, HashSet<UserId>> {
    running
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
