//! Validation of identity and event fields at the hook boundary.

use super::{HookEvent, HookEventPayload};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum HookValidationError {
    #[error("session_id must be a nonempty identifier of at most 255 bytes")]
    SessionKey,
    #[error("Unsupported or malformed hook event")]
    UnknownEvent,
    #[error("{event} requires {fields}")]
    MissingFields {
        event: &'static str,
        fields: &'static str,
    },
    #[error("{key} or event_id is required for retry-safe ingestion")]
    MissingDeliveryId { key: &'static str },
}

pub fn validate_session_key(value: &str) -> Result<(), HookValidationError> {
    if value.is_empty()
        || value.len() > 255
        || !value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_:.".contains(&c))
    {
        return Err(HookValidationError::SessionKey);
    }
    Ok(())
}

impl HookEventPayload {
    pub fn validate_ingestion(&self) -> Result<(), HookValidationError> {
        validate_session_key(self.session_id())?;
        if self.common.hook_event_name.is_empty() || matches!(self.event, HookEvent::Unknown(_)) {
            return Err(HookValidationError::UnknownEvent);
        }
        let missing = |event, fields| HookValidationError::MissingFields { event, fields };
        match &self.event {
            HookEvent::UserPromptSubmit(d) if d.prompt.trim().is_empty() => {
                return Err(missing("UserPromptSubmit", "prompt"));
            },
            HookEvent::PreToolUse(d) if d.name.is_empty() || d.use_id.is_empty() => {
                return Err(missing("PreToolUse", "tool_name and tool_use_id"));
            },
            HookEvent::PostToolUse(d) if d.name.is_empty() || d.use_id.is_empty() => {
                return Err(missing("PostToolUse", "tool_name and tool_use_id"));
            },
            HookEvent::PostToolUseFailure(d)
                if d.tool_name.is_empty() || d.tool_use_id.is_empty() =>
            {
                return Err(missing("PostToolUseFailure", "tool_name and tool_use_id"));
            },
            _ => {},
        }
        Ok(())
    }

    pub fn delivery_id(&self) -> Result<&str, HookValidationError> {
        let key = match self.event {
            HookEvent::UserPromptSubmit(_) => "prompt_id",
            HookEvent::PreToolUse(_)
            | HookEvent::PostToolUse(_)
            | HookEvent::PostToolUseFailure(_) => "tool_use_id",
            _ => "event_id",
        };
        // JSON: the hook envelope is an external protocol with event-specific keys.
        self.raw
            .get(key)
            .and_then(serde_json::Value::as_str)
            .or_else(|| self.raw.get("event_id").and_then(serde_json::Value::as_str))
            .filter(|v| !v.is_empty() && v.len() <= 255 && !v.chars().any(char::is_control))
            .ok_or(HookValidationError::MissingDeliveryId { key })
    }
}
