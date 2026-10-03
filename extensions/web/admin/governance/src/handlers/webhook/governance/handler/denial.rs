//! The audit row for a call refused before authentication resolved a user.
//!
//! It is spawned rather than awaited: provisioning the anonymous principal is
//! an upsert plus fingerprint work, and awaiting it on every unauthenticated
//! probe would hand anonymous callers a write-amplification lever. The denial
//! is enforced regardless; write failures log under
//! `governance.audit.write_failed`.

use std::sync::Arc;

use axum::http::{HeaderMap, header};
use systemprompt::identifiers::CallId;
use systemprompt::traits::SessionAnalytics;
use systemprompt_security::policy::{
    AuditOrigin, AuditTarget, ChainEntryOutcome, ChainEntryResult, DecisionAudit, record_decision,
};

use systemprompt_security::policy::types::AccessScope;

use super::super::types::AuthDenialParams;
use super::authn::deny_for_auth_failure;
use super::principal_snapshot;

fn header_str(headers: &HeaderMap, name: header::HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned)
}

// Why: core's `compute_fingerprint` falls back to exactly user agent + locale,
// so those are the only signals the anonymous principal is keyed on.
fn anonymous_fingerprint_signals(headers: &HeaderMap) -> SessionAnalytics {
    SessionAnalytics {
        user_agent: header_str(headers, header::USER_AGENT),
        preferred_locale: header_str(headers, header::ACCEPT_LANGUAGE),
        ..SessionAnalytics::default()
    }
}

pub(super) fn spawn_auth_denial(params: &AuthDenialParams<'_>, reason: &str) {
    let pool = Arc::<sqlx::Pool<sqlx::Postgres>>::clone(params.pool);
    let reason = reason.to_owned();
    let session_id = params.session_id.clone();
    let tool_name = params.tool_name.to_owned();
    let plugin_id = params.plugin_id.cloned();
    let session_service = Arc::clone(params.session_service);
    let headers = params.headers.clone();

    tokio::spawn(async move {
        let analytics = anonymous_fingerprint_signals(&headers);
        let user_id = match session_service.ensure_anonymous_user(&analytics).await {
            Ok((uid, _fingerprint)) => uid,
            Err(e) => {
                tracing::error!(
                    target: "governance.audit.write_failed",
                    error = %e,
                    session_id = %session_id,
                    "could not resolve anonymous principal; auth-denial audit dropped",
                );
                return;
            },
        };
        let audit = DecisionAudit {
            id: uuid::Uuid::new_v4().to_string(),
            call_id: CallId::generate(),
            origin: AuditOrigin::Governed,
            decision: deny_for_auth_failure(&reason),
            principal: principal_snapshot(user_id, session_id.clone(), AccessScope::Unknown, None),
            target: AuditTarget {
                tool_name: systemprompt::identifiers::McpToolName::new(tool_name),
                plugin_id,
                tool_use_id: None,
            },
            chain: vec![ChainEntryOutcome {
                policy_id: systemprompt::identifiers::PolicyId::new("authentication"),
                result: ChainEntryResult::Fail,
                detail: reason,
                duration_ms: 0.0,
            }],
            approver: None,
            act_chain: Vec::new(),
            context_id: None,
            trace_id: None,
        };
        if let Err(e) = record_decision(&pool, &audit).await {
            tracing::error!(
                target: "governance.audit.write_failed",
                error = %e,
                session_id = %session_id,
                "governance audit write failed; row dropped",
            );
        }
    });
}
