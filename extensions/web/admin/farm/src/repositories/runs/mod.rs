//! Queries over run rows from queue to terminal status: reads and the paged
//! list, lifecycle writes, aggregate statistics, and the gateway rows a run
//! joins to by its session.

mod links;
mod reads;
mod stats;
mod writes;

pub use links::{find_run_context_id, list_run_requests};
pub use reads::{
    find_run, list_active_runs, list_recent_runs, list_runs_paged,
    list_runs_to_requeue,
};
pub use stats::{get_run_stats, list_daily_run_counts};
pub use writes::{
    insert_run, update_orphaned_runs_failed, update_run_cancel_requested,
    update_run_claude_session, update_run_finished, update_run_pid,
    update_run_preparing, update_run_started,
};
