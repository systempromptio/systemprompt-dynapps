//! Which callers may spend a service account's credential.
//!
//! Core forwards the run's declared purpose as `x-systemprompt-caller`. A
//! service account exists to serve workflow write-backs
//! (`workflow:<definition_id>:write`) and agent farms (`farm:<id>`); any other
//! purpose, or none, is refused so an interactive path cannot borrow it.

#[must_use]
pub fn allowed(caller: Option<&str>) -> bool {
    let Some(caller) = caller else {
        return false;
    };
    if caller.len() > 256 || caller.chars().any(char::is_control) {
        return false;
    }
    if let Some(rest) = caller.strip_prefix("workflow:") {
        return rest
            .strip_suffix(":write")
            .is_some_and(|definition| !definition.is_empty());
    }
    caller
        .strip_prefix("farm:")
        .is_some_and(|farm| !farm.is_empty())
}
