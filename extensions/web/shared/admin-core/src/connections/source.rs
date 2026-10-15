//! Where a user's [`ConnectionSnapshot`] comes from.
//!
//! Building a snapshot reads connector credentials and entitlement, neither
//! of which this crate owns. The crate
//! that does registers one [`ConnectionSnapshotSource`] with
//! `inventory::submit!`, and every reader here calls [`get_connections`].
//! The same source names a connector for display
//! ([`connector_display_name`]), since the label is read from the server's
//! connector settings too.

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

    fn display_name(&self, provider: &str) -> String;
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

pub fn connector_display_name(provider: &str) -> String {
    inventory::iter::<ConnectionSourceRegistration>
        .into_iter()
        .next()
        .map_or_else(
            || provider.to_owned(),
            |source| source.0.display_name(provider),
        )
}
