//! The public run API under `/api/public/agent-farm`: list agents, launch,
//! list, read, follow and cancel runs, launch and read eval runs, and read
//! eval history.
//!
//! The caller is the request's [`UserContext`], resolved from a session
//! cookie or a personal access token; a run launched with a session is
//! recorded as a dashboard run, one launched with a token as an API run.

mod agents;
mod evals;
mod events;
mod history;
mod launch;
mod run_json;
mod runs;
mod shared;

pub(crate) use agents::list_agents;
pub(crate) use evals::{get_eval_run, run_eval_suite};
pub(crate) use events::list_run_events;
pub(crate) use history::{eval_history, list_suite_runs};
pub(crate) use launch::launch_run;
pub(crate) use runs::{cancel_run, get_run, list_runs};
