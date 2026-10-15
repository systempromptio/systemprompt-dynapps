//! Boot-time farm sync of checkouts missing from this machine.
//!
//! Every farm whose checkout for its synced commit is absent from the data
//! root (a fresh volume, a new machine, a farm never synced) is synced once,
//! in the background. A farm whose checkout is present is left alone, so a
//! restart costs one directory check per farm.

use std::sync::Arc;

use crate::repositories::farms;
use crate::repositories::rows::FarmRow;
use crate::state::FarmState;

use super::{FarmSourceCapture, NativeCapture, sync_farm};

const SCRATCH_DIR: &str = "scratch";

#[must_use]
pub fn checkout_missing(state: &FarmState, farm: &FarmRow) -> bool {
    farm.synced_commit_sha
        .as_deref()
        .is_none_or(|sha| !state.paths.farm_dir(farm.id.as_str(), sha).is_dir())
}

pub async fn sync_missing_checkouts(state: &FarmState) -> usize {
    let farms = match farms::list_farms(&state.pool).await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!(error = %e, "listing farms for the boot sync");
            return 0;
        },
    };
    let mut synced = 0;
    for farm in farms.iter().filter(|f| checkout_missing(state, f)) {
        let capture: Arc<dyn FarmSourceCapture> =
            Arc::new(NativeCapture::new(state.paths.data_root.join(SCRATCH_DIR)));
        match sync_farm(state, capture, &farm.id, None).await {
            Ok(report) => {
                synced += 1;
                tracing::info!(farm = farm.id.as_str(), commit = %report.commit, "farm checkout restored at boot");
            },
            Err(e) => tracing::warn!(farm = farm.id.as_str(), error = %e, "boot farm sync failed"),
        }
    }
    synced
}
