//! Data access for the admin surface, one module per domain.
//!
//! Callers path-qualify (`repositories::config::gateway::create_route`);
//! this module re-exports nothing, so the module path is the only name a
//! symbol has and collisions between domains cannot arise.

pub mod analytics;
pub mod config;
pub mod dashboard;
pub mod dev_login;
pub mod devices;
pub mod gateway_policies;
pub mod gateway_routes;
pub mod governance;
pub mod governance_chain;
pub mod groups;
pub mod jobs;
pub mod lifecycle;
pub mod marketplace;
pub mod mcp;
pub mod observability;
pub mod overview;
pub mod people_usage;
pub mod projects;
pub mod reports;
pub mod roles;
pub mod secrets;
pub mod session_state;
pub mod sync;
pub mod traces;
pub mod users;

pub mod analysis;
