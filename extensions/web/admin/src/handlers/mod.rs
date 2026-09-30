//! HTTP handlers for the admin plane.
//!
//! SSR page handlers live in [`ssr`]; everything else here is JSON API or
//! webhook intake. Handlers own status mapping and call repositories for data.

mod dashboard;
pub(crate) mod hooks_track;
pub(crate) mod ssr;

pub(crate) use dashboard::{dashboard_handler, list_events_handler};
