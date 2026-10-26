//! The `source` an access-control row carries.
//!
//! Core's vocabulary is closed (`code:<file>`, `bundle:<name>`,
//! `console:<user>`, `migration`) and the governance tables enforce it with a
//! CHECK. Every console write names the
//! administrator who made it; every write projected from the services tree is
//! [`SERVICES_SOURCE`].

use systemprompt::governance::authz::{CONSOLE_SOURCE_PREFIX, SERVICES_SOURCE};
use systemprompt::identifiers::UserId;

#[must_use]
pub fn console_source(actor: &UserId) -> String {
    format!("{CONSOLE_SOURCE_PREFIX}{}", actor.as_str())
}

#[must_use]
pub fn is_console_source(source: &str) -> bool {
    source.starts_with(CONSOLE_SOURCE_PREFIX)
}

#[must_use]
pub fn is_code_source(source: &str) -> bool {
    source == SERVICES_SOURCE || source.starts_with("code:")
}

#[must_use]
pub const fn services_source() -> &'static str {
    SERVICES_SOURCE
}
