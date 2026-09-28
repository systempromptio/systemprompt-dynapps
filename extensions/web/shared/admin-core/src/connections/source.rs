//! Where a user's [`ConnectionSnapshot`] comes from.
//!
//! Building a snapshot reads connector credentials, the Salesforce org
//! registry and group entitlement, none of which this crate owns. The crate
//! that does registers one [`ConnectionSnapshotSource`] with
//! `inventory::submit!`, and every reader here calls [`get_connections`].

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::ConnectionSnapshot;
use crate::error::{AdminError, AdminResult};

/// `#[async_trait]` rather than a native `async fn`: the registration holds a
/// `&'static dyn ConnectionSnapshotSource`, and a native async trait method is
/// not object-safe.
#[async_trait::async_trait]
pub trait ConnectionSnapshotSource: Send + Sync {
    async fn snapshot(&self, pool: &PgPool, user: &UserId) -> AdminResult<ConnectionSnapshot>;
}

#[derive(Clone, Copy)]
pub struct ConnectionSourceRegistration(pub &'static dyn ConnectionSnapshotSource);

impl std::fmt::Debug for ConnectionSourceRegistration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ConnectionSourceRegistration")
    }
}

inventory::collect!(ConnectionSourceRegistration);

pub async fn get_connections(pool: &PgPool, user: &UserId) -> AdminResult<ConnectionSnapshot> {
    let source = inventory::iter::<ConnectionSourceRegistration>
        .into_iter()
        .next()
        .ok_or_else(|| AdminError::internal("no connection snapshot source is registered"))?;
    source.0.snapshot(pool, user).await
}
