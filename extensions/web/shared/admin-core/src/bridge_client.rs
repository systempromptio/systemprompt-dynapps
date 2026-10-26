//! The desktop bridge as the console names it.
//!
//! The binary every install and login command runs, and the issuer and gateway
//! URL those commands point at. The profile page, the connect page and the
//! device-link approval all print these, so they read them from one place.

use systemprompt::manifest::Config;

pub const BRIDGE_BINARY: &str = "dynapps-bridge";

#[must_use]
pub fn read_config_strings(config: &Config) -> (Option<String>, Option<String>) {
    (
        Some(config.jwt_issuer.clone()),
        Some(config.api_external_url.trim_end_matches('/').to_owned()),
    )
}
