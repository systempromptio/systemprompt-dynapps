//! The envelope a synchronous governance hook answers with.

use super::super::types::{GovernanceDecision, GovernanceResponse, HookSpecificOutput};
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use systemprompt_security::authz::Decision;

// Why: Claude Code reads a hook's error status as "hook unavailable" and lets
// the call through, so every verdict — failures included — is a 200 envelope.
#[derive(Debug)]
pub(crate) struct HookVerdict {
    decision: Decision,
    hook_event_name: &'static str,
}

impl HookVerdict {
    pub(super) const fn new(decision: Decision, hook_event_name: &'static str) -> Self {
        Self {
            decision,
            hook_event_name,
        }
    }
}

impl IntoResponse for HookVerdict {
    fn into_response(self) -> Response {
        let permission_decision = GovernanceDecision::from_decision(&self.decision);
        let permission_decision_reason = match self.decision {
            Decision::Allow { .. } => None,
            // Why: Claude Code shows `permissionDecisionReason` to the user as a
            // refusal, so a warn-mode reason stays in the audit row only.
            Decision::Warn { reason } => {
                tracing::warn!(%reason, "governance warn-mode finding; allowing the tool call");
                None
            },
            Decision::Deny { reason } => Some(format!("[GOVERNANCE] {reason}")),
            Decision::Pending { reason } => Some(format!("[GOVERNANCE] {reason}")),
        };
        let response = GovernanceResponse {
            hook_specific_output: HookSpecificOutput {
                hook_event_name: self.hook_event_name,
                permission_decision,
                permission_decision_reason,
            },
        };
        (StatusCode::OK, Json(response)).into_response()
    }
}
