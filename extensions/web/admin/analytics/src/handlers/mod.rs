//! HTTP handlers for the analytics domain.
//!
//! SSR page handlers live in [`ssr`]; `dashboard` answers the admin API's
//! counters and events feed and [`hooks_track`] the Claude Code hook intake.
//! Handlers own status mapping and call repositories for data.

mod dashboard;
pub(crate) mod hooks_track;
pub(crate) mod ssr;

pub(crate) use dashboard::{dashboard_handler, list_events_handler};
