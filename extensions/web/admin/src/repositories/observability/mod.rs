//! What the profile declares under `observability.otlp` and what the core
//! `otlp_export` job has done with it.
//!
//! Core owns the exporter and its `otlp_export_state` table; this module only
//! reads both and hands the page a typed view. The two reads are kept apart
//! because they fail apart: an unset block is "not configured", an unreadable
//! table is an error, and the page says which. The state repository is
//! core's, built once in the admin composition root and injected.

pub mod view;

use systemprompt::config::ProfileBootstrap;
use systemprompt::models::profile::OtlpExportConfig;
use systemprompt::scheduler::{OtlpExportState, OtlpExportStateRepository};

use systemprompt_web_admin_core::error::AdminError;

pub fn find_declared_export() -> Result<Option<OtlpExportConfig>, AdminError> {
    Ok(ProfileBootstrap::get()?.observability.otlp().cloned())
}

pub async fn list_export_states(
    repository: &OtlpExportStateRepository,
) -> Result<Vec<OtlpExportState>, AdminError> {
    repository.list_states().await.map_err(AdminError::internal)
}
