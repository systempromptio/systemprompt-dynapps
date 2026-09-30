//! Data access for the admin surface, one module per domain.
//!
//! Callers path-qualify (`repositories::dashboard::get_dashboard_data`);
//! this module re-exports nothing, so the module path is the only name a
//! symbol has and collisions between domains cannot arise.

pub mod analytics;
pub mod dashboard;
pub mod observability;
pub mod overview;
pub mod reports;
pub mod session_state;
pub mod traces;

pub mod analysis;
