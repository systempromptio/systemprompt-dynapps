//! What every tool needs beyond the request: the pool, the composed services
//! tree, and (with the `farm` feature) the farm state the launch and cancel
//! services run over.

use std::path::PathBuf;
use std::sync::Arc;

use sqlx::PgPool;

#[cfg(feature = "farm")]
use systemprompt_web_admin_farm::state::FarmState;

#[derive(Debug)]
pub struct Env {
    pub pool: Arc<PgPool>,
    pub services_path: PathBuf,
    pub services: systemprompt_web_admin_core::extension_services::ExtensionServices,
    #[cfg(feature = "farm")]
    pub farm: Option<Arc<FarmState>>,
}

impl Env {
    #[cfg(feature = "farm")]
    pub fn farm_state(&self) -> Result<&Arc<FarmState>, rmcp::ErrorData> {
        self.farm.as_ref().ok_or_else(|| {
            systemprompt_mcp_shared::reply::failure(
                "the agent farm is not configured on this server",
            )
        })
    }
}
