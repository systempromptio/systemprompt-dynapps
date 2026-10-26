//! The `WebExtension` value.
//!
//! The site and blog config are loaded from the active services root the
//! extension context hands out, in the router and in the content-pipeline
//! hooks (`extension_impl`), so nothing here reads the profile. The value owns
//! the agent farm's in-flight remote-run registry, which the runner service and
//! the worker routes share.

use std::sync::Arc;

use systemprompt_web_admin_farm::services::runner::fly::registry::RemoteRuns;

use systemprompt::extension::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct WebExtension {
    pub(crate) farm_runs: Arc<RemoteRuns>,
}

impl WebExtension {
    pub const PREFIX: &'static str = "web";

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

register_extension!(WebExtension);

// Why: Postgres FKs cannot hold where `user_id` may be the `system` principal,
// so these tables purge explicitly; the rest cascade from `schema/`.
user_purge_tables!(
    "systemprompt-web",
    [
        ("ai_request_scopes", "user_id"),
        ("conversation_analyses", "user_id"),
        ("admin_usage_daily_rollups", "user_id"),
    ]
);
