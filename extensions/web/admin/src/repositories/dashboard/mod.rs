//! Persistence for the admin dashboard and control centre.

pub mod aggregates;
pub mod conversation_analytics;
pub mod hooks_track;
pub mod overview;
pub mod queries;
pub mod traffic;
pub mod usage_aggregations;
pub mod usage_rollups;

pub use overview::{get_dashboard_data, get_events_page};
