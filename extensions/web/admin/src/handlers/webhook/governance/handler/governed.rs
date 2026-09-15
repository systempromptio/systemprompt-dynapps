//! Projection of a raw hook payload onto the policy engine's governed types.

use systemprompt::identifiers::McpToolName;
use systemprompt_security::policy::{GovernedInput, GovernedTarget, McpToolInput};

use crate::types::webhook::HookEventPayload;

pub(super) fn governed_target(
    payload: &HookEventPayload,
) -> Result<GovernedTarget, systemprompt::identifiers::error::IdValidationError> {
    if let Some(name) = payload.tool_name() {
        return Ok(GovernedTarget::Tool {
            tool: McpToolName::try_new(name)?,
        });
    }
    Ok(if payload.prompt().is_some() {
        GovernedTarget::Prompt
    } else {
        GovernedTarget::Unknown
    })
}

pub(super) fn governed_input(payload: &HookEventPayload) -> GovernedInput {
    payload.prompt().map_or_else(
        || {
            GovernedInput::tool_arguments(McpToolInput::new(
                payload.tool_input().cloned().unwrap_or_default(),
            ))
        },
        |text| GovernedInput::prompt_text(text.to_owned()),
    )
}
