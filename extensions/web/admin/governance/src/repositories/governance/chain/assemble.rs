//! Top-level chain assembly: resolve the id, fan out the per-table fetches, and
//! roll the parts up into a single [`ChainEnvelope`].
//!
//! The id may be a decision, request, trace or session id. A gateway-only
//! session has no governance decision to name its user, so the identity falls
//! back to the user the request itself was audited under.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::fetch::{find_summary, list_decisions, list_events, list_requests};
use super::resolve::resolve_session_id;
use super::{
    AiRequestSummary, ChainEnvelope, ChainIdentity, ChainTotals, ChainUsageEvent, DecisionStage,
};

fn compute_totals(
    decisions: &[DecisionStage],
    requests: &[AiRequestSummary],
    events: &[ChainUsageEvent],
) -> ChainTotals {
    let mut totals = ChainTotals {
        decision_count: decisions.len() as i64,
        deny_count: decisions.iter().filter(|d| d.decision == "deny").count() as i64,
        event_count: events.len() as i64,
        request_count: requests.len() as i64,
        ..ChainTotals::default()
    };
    for r in requests {
        totals.total_cost_microdollars += r.cost_microdollars;
        totals.total_input_tokens += i64::from(r.input_tokens.unwrap_or(0));
        totals.total_output_tokens += i64::from(r.output_tokens.unwrap_or(0));
    }
    totals
}

fn chain_user(decision_user: UserId, request_user: Option<UserId>) -> UserId {
    if decision_user.as_str().is_empty() {
        request_user.unwrap_or(decision_user)
    } else {
        decision_user
    }
}

pub async fn find_decision_chain(
    pool: &PgPool,
    id: &str,
) -> Result<Option<ChainEnvelope>, sqlx::Error> {
    let Some(session_id) = resolve_session_id(pool, id).await? else {
        return Ok(None);
    };

    let (decisions, slots) = list_decisions(pool, &session_id).await?;
    let (requests, request_slots) = list_requests(pool, &session_id).await?;
    let trace_id = request_slots.trace_id;
    let events = list_events(pool, &session_id).await?;
    let summary = find_summary(pool, &session_id).await?;

    let identity = ChainIdentity {
        user_id: chain_user(slots.user_id, request_slots.user_id),
        agent_id: slots.agent_id,
        agent_scope: slots.agent_scope,
    };

    let totals = compute_totals(&decisions, &requests, &events);

    Ok(Some(ChainEnvelope {
        trace_id,
        session_id,
        identity,
        decisions,
        requests,
        events,
        summary,
        totals,
    }))
}
