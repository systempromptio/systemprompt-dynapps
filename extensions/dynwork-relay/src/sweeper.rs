//! The revocation watch, which cuts revoked devices within seconds.
//!
//! Every `revocation_poll` it reads the devices revoked since the last look
//! and cuts their tunnels, so a revocation made by any path (another route,
//! an operator, a later sign-in flow) lands within seconds. It also deletes
//! pairings and mobile sign-in grants that expired a day ago, and drops ring
//! buffers whose envelopes have all aged out.

use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio_util::sync::CancellationToken;

use crate::audit::AuditEvent;
use crate::state::RelayState;

const OVERLAP: chrono::Duration = chrono::Duration::seconds(10);
const PAIRING_RETENTION: chrono::Duration = chrono::Duration::days(1);
const PURGE_EVERY: Duration = Duration::from_secs(600);

pub fn spawn(state: &RelayState) {
    let watched = state.clone();
    state
        .tasks
        .spawn_cancellable("dynwork_relay_revocations", move |token| {
            watch(watched, token)
        });
}

pub async fn watch(state: RelayState, token: CancellationToken) {
    let mut cursor = state.clock.now();
    let mut last_purge = tokio::time::Instant::now();
    loop {
        tokio::select! {
            () = token.cancelled() => return,
            () = tokio::time::sleep(state.limits.revocation_poll) => {},
        }
        sweep(&state, &mut cursor).await;
        state.hub.sweep_rings(std::time::Instant::now());
        if last_purge.elapsed() >= PURGE_EVERY {
            last_purge = tokio::time::Instant::now();
            purge(&state).await;
        }
    }
}

pub async fn sweep(state: &RelayState, cursor: &mut DateTime<Utc>) -> usize {
    let revoked = match state.store.list_revoked_since(*cursor - OVERLAP).await {
        Ok(revoked) => revoked,
        Err(error) => {
            tracing::warn!(error = %error, "dynwork relay revocation sweep failed");
            return 0;
        },
    };
    let mut cut = 0;
    for device in revoked {
        let closed = state.hub.revoke(device.device);
        if closed > 0 {
            state.audit.record(&AuditEvent::Revoke {
                user: device.user,
                device: device.device,
            });
        }
        cut += closed;
        if device.revoked_at > *cursor {
            *cursor = device.revoked_at;
        }
    }
    cut
}

async fn purge(state: &RelayState) {
    match state
        .store
        .delete_expired_pairings(state.clock.now() - PAIRING_RETENTION)
        .await
    {
        Ok(deleted) => tracing::debug!(deleted, "dynwork relay expired pairings deleted"),
        Err(error) => tracing::warn!(error = %error, "dynwork relay pairing purge failed"),
    }
    purge_grants(state).await;
}

async fn purge_grants(state: &RelayState) {
    match state
        .store
        .delete_expired_grants(state.clock.now() - PAIRING_RETENTION)
        .await
    {
        Ok(deleted) => tracing::debug!(deleted, "dynwork mobile expired grants deleted"),
        Err(error) => tracing::warn!(error = %error, "dynwork mobile grant purge failed"),
    }
}
