//! The `WebExtension` value and the configuration it owns.
//!
//! Config loading lives in `systemprompt_web_site::config_loader` and
//! `systemprompt_web_shared::config`, beside the types they deserialise; this
//! type loads both once when core's registry constructs the extension and
//! hands them to the `Extension` impl in `extension_impl` and to the router.
//! It also owns the agent farm's in-flight remote-run registry, which the
//! runner service and the worker routes share.

use std::sync::Arc;

use crate::config::BlogConfigValidated;
use systemprompt_web_admin_farm::services::runner::fly::registry::RemoteRuns;
use systemprompt_web_site::config_loader::SiteConfig;

use systemprompt::extension::prelude::*;

#[derive(Debug, Clone)]
pub struct WebExtension {
    pub(crate) site: Arc<SiteConfig>,
    pub(crate) blog: Option<Arc<BlogConfigValidated>>,
    pub(crate) farm_runs: Arc<RemoteRuns>,
}

impl Default for WebExtension {
    fn default() -> Self {
        Self::new()
    }
}

impl WebExtension {
    pub const PREFIX: &'static str = "web";

    #[must_use]
    pub fn new() -> Self {
        let blog = match BlogConfigValidated::load_from_profile_or_none() {
            Ok(config) => config,
            Err(error) => {
                tracing::error!(
                    %error,
                    "Blog config error: link generation and content APIs will run unconfigured"
                );
                None
            },
        };
        Self {
            site: Arc::new(SiteConfig::load()),
            blog,
            farm_runs: Arc::new(RemoteRuns::default()),
        }
    }

    #[must_use]
    pub fn site_config(&self) -> &SiteConfig {
        &self.site
    }

    #[must_use]
    pub fn blog_config(&self) -> Option<Arc<BlogConfigValidated>> {
        self.blog.clone()
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
