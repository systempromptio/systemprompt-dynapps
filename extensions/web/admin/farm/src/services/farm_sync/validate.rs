//! Identifier and path rules shared by every farm definition parser, as
//! `FARM.md` states them.

use crate::error::{FarmError, FarmResult};

pub const ALLOWED_TOOLS: [&str; 13] = [
    "Read",
    "Grep",
    "Glob",
    "Write",
    "Edit",
    "MultiEdit",
    "Bash",
    "WebFetch",
    "WebSearch",
    "TodoWrite",
    "NotebookEdit",
    "Task",
    "Skill",
];

pub const PERMISSION_MODES: [&str; 4] = ["default", "acceptEdits", "plan", "bypassPermissions"];

fn shaped(value: &str, extra: char) -> bool {
    let mut chars = value.chars();
    let starts_lower = chars.next().is_some_and(|c| c.is_ascii_lowercase());
    (3..=64).contains(&value.len())
        && starts_lower
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == extra)
}

#[must_use]
pub fn is_identifier(value: &str) -> bool {
    shaped(value, '_')
}

#[must_use]
pub fn is_farm_id(value: &str) -> bool {
    shaped(value, '-')
}

#[must_use]
pub fn is_case_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

#[must_use]
pub fn is_safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.contains('\0')
        && !value.contains('\\')
        && value
            .split('/')
            .all(|seg| !seg.is_empty() && seg != "." && seg != "..")
}

pub fn definition_error(path: &str, message: impl std::fmt::Display) -> FarmError {
    FarmError::Validation(format!("{path}: {message}"))
}

pub fn require(condition: bool, path: &str, message: impl std::fmt::Display) -> FarmResult<()> {
    if condition {
        Ok(())
    } else {
        Err(definition_error(path, message))
    }
}
