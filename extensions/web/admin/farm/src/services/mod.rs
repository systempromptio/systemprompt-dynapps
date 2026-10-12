//! Farm behaviour behind the routes: syncing a farm from its Git source,
//! minting per-run gateway credentials, running Claude Code, and scoring
//! eval suites.

pub mod entitlement;
pub mod evals;
pub mod farm_sync;
pub mod gateway_auth;
pub mod runner;
pub mod runs;
