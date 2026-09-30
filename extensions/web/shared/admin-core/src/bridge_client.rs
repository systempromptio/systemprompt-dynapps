//! The desktop bridge as the console names it.
//!
//! The binary every install and login command runs, and the issuer and gateway
//! URL those commands point at. The profile page, the connect page and the
//! device-link approval all print these, so they read them from one place.

use systemprompt::models::Config;

pub const BRIDGE_BINARY: &str = "astound-bridge";

pub fn read_config_strings() -> (Option<String>, Option<String>) {
    Config::get().map_or((None, None), |c| {
        (
            Some(c.jwt_issuer.clone()),
            Some(c.api_external_url.trim_end_matches('/').to_owned()),
        )
    })
}
