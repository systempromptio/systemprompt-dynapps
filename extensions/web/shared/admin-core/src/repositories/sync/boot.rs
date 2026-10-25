//! What boot does with the install's own planes (groups and the staging copy
//! of the gateway routes): look and report, and seed an empty projection.
//!
//! The planes core owns are core's at boot too: nothing here touches them,
//! and they reach the database through `systemprompt admin config apply` and
//! `/admin/sync`. An install plane's file is what its table is expected to
//! contain; every start computes the drift and says so in the log, and seeds
//! the table once, when there is nothing yet to disagree with. Upserting on
//! every start was how console edits vanished overnight; one loop over the
//! two planes is what stops a plane from drifting back to that.
//!
//! A declaration that cannot be read fails the boot. The page turns the same
//! condition into a card headline because an operator came to look; a server
//! starting with half its declarations is a server nobody asked for.

use sqlx::PgPool;

use super::plane::{Actor, SyncMode, SyncPlane};
use super::plane_ids;
use super::registry::find_plane;
use crate::error::{AdminError, AdminResult};

pub const BOOT_ORDER: [&str; 2] = [plane_ids::GROUPS, plane_ids::GATEWAY_ROUTES];

/// One plane's boot outcome, for the log line.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlaneBoot {
    pub declared: usize,
    pub in_db: usize,
    pub seeded: bool,
    pub inserted: usize,
    pub drift_rows: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BootReport {
    pub groups: PlaneBoot,
    pub gateway_routes: PlaneBoot,
}

pub async fn reconcile_plane(pool: &PgPool, plane: &dyn SyncPlane) -> AdminResult<PlaneBoot> {
    let drift = plane.drift(pool).await?;
    if let Some(reason) = drift.unreadable {
        return Err(AdminError::invalid(
            "a declaration could not be read at boot",
            format!("{}: {reason}", plane.source_file()),
        ));
    }
    let mut boot = PlaneBoot {
        declared: drift.declared_count,
        in_db: drift.in_db,
        drift_rows: drift.rows.len(),
        ..PlaneBoot::default()
    };
    if drift.in_db == 0 && drift.declared_count > 0 {
        let outcome = plane.apply(pool, SyncMode::Overwrite, Actor::Boot).await?;
        boot.seeded = true;
        boot.inserted = outcome.inserted;
        boot.drift_rows = 0;
        tracing::info!(
            plane = plane.id(),
            file = plane.source_file(),
            declared = boot.declared,
            inserted = outcome.inserted,
            "sync_seeded: empty projection populated from code"
        );
        return Ok(boot);
    }
    if drift.is_clean {
        tracing::info!(
            plane = plane.id(),
            declared = boot.declared,
            in_db = boot.in_db,
            "sync_in_sync: code and database agree; nothing written"
        );
    } else {
        tracing::warn!(
            plane = plane.id(),
            file = plane.source_file(),
            declared = boot.declared,
            in_db = boot.in_db,
            drift_rows = boot.drift_rows,
            "sync_drift: code and database differ; nothing written — review and \
             synchronise on /admin/sync"
        );
    }
    Ok(boot)
}

pub async fn reconcile_all(pool: &PgPool) -> AdminResult<BootReport> {
    let mut report = BootReport::default();
    for id in BOOT_ORDER {
        let plane = find_plane(id)
            .ok_or_else(|| AdminError::internal(format!("sync plane '{id}' is not registered")))?;
        let boot = reconcile_plane(pool, plane.as_ref()).await?;
        if id == plane_ids::GROUPS {
            report.groups = boot;
        } else {
            report.gateway_routes = boot;
        }
    }
    Ok(report)
}
