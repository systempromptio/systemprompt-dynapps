//! Observed facts about an agent run, whatever launched it.
//!
//! A run's "eval" here is nothing but what the tables recorded: core's
//! `agent_runs` row, its `agent_run_events`, the gateway requests its
//! session made (`ai_requests`) and the governance decisions its hooks
//! raised (`governance_decisions`). No model judges anything. The one
//! derived value is `output_schema_valid`: the structured output checked
//! against the schema the request asked for (`validity`).
//!
//! The origin is opaque (`origin_kind` / `origin_id`); only the project a run
//! belongs to is origin-aware, since core's row does not carry one. A farm
//! run reaches its project through the farm linked to it, a workflow step
//! through `workflow_runs` where that table exists (`query::workflow`).

//! `MAX_LIST_LIMIT` bounds every list, whatever the filter asks.

mod query;
mod rows;
mod stats;
mod validity;
mod workflow;

pub use query::{find_run_facts, list_run_facts};
pub use rows::{
    ErrorClassCounts, OriginKindStats, RunFacts, RunFilter, RunScope, RunStats, RunStatsBucket,
    StatusCounts,
};
pub use stats::get_agent_run_stats;
pub use validity::output_schema_valid;

pub const MAX_LIST_LIMIT: i64 = 500;
