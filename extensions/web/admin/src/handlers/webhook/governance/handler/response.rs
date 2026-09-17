//! The envelope a synchronous governance hook answers with.

use super::super::types::{GovernanceDecision, GovernanceResponse, HookSpecificOutput};
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use systemprompt_security::authz::Decision;

// Why: A hook's verdict, rendered as the 200 envelope the hook contract
// requires whether the policy chain reached it or a failure on the way there
// did: an error status reads as "hook unavailable" to the caller and lets the
// call through, so the failure channel is a deny body, not a status.
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
            // Why: the reason is deliberately not returned to the caller. Claude
            // Code renders `permissionDecisionReason` to the user, and a warning
            // rendered as a refusal reads as one; the finding's home is the audit
            // row and `infra logs governance report`.
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
