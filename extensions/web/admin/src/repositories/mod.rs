//! Data access for the admin surface, one module per domain.
//!
//! Callers path-qualify (`repositories::config::gateway::create_route`);
//! this module re-exports nothing, so the module path is the only name a
//! symbol has and collisions between domains cannot arise.

pub mod analytics;
pub mod bridge;
pub mod config;
pub mod dashboard;
pub mod demo;
pub mod dev_login;
pub mod external_agents;
pub mod governance;
pub mod groups;
pub mod jobs;
pub mod marketplace;
pub mod mcp;
pub mod people_usage;
pub mod projects;
pub mod reports;
pub mod scope;
pub mod secrets;
pub mod traces;
pub mod users;
