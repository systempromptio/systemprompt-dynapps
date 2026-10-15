//! Farm behaviour behind the routes: syncing a farm from its Git source,
//! queueing runs onto core's `agent_runs` executor, and scoring eval suites.

pub mod entitlement;
pub mod evals;
pub mod farm_sync;
pub mod runner;
pub mod runs;
