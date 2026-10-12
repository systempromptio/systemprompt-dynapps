//! Agent farm domain of the admin dashboard: agents authored in an external
//! Git marketplace, synced into the instance, run headlessly by Claude Code in
//! an isolated directory, attributed to the user who launched them, and scored
//! by eval suites.
//!
//! - [`types`] — typed ids, run and eval statuses, and the run event payload.
//! - [`config`] — the `web/config/agent-farms.yaml` loader.
//! - [`state`] — the shared [`state::FarmState`] the public API and runner use.
//! - [`repositories`] — the `agent_farm_*` tables.
//! - [`services`] — farm sync, per-run gateway credentials, the runner, evals.
//! - [`routes`] — the dashboard pages, their fragments, the console JSON API,
//!   and the public `/api/public/agent-farm` router.

pub mod config;
pub mod error;
pub(crate) mod handlers;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod state;
pub mod types;
pub mod views;
