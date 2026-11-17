//! The `WebExtension` value.
//!
//! The site and blog config are loaded from the active services root the
//! extension context hands out, in the router and in the content-pipeline
//! hooks (`extension_impl`), so nothing here reads the profile.

use systemprompt::extension::prelude::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct WebExtension;

impl WebExtension {
    pub const PREFIX: &'static str = "web";

    #[must_use]
    pub const fn new() -> Self {
        Self
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
