//! The path segments the console puts in a farm-api URL, checked first so
//! nothing a visitor sends reaches the path unvalidated.

use super::FarmApiError;

pub fn is_task_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

pub fn is_agent_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 32
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub(super) fn task_path(id: &str, suffix: &str) -> Result<String, FarmApiError> {
    if !is_task_id(id) {
        return Err(FarmApiError::BadTaskId(id.to_owned()));
    }
    Ok(format!("/tasks/{id}{suffix}"))
}
