//! Data access owned by governance.
//!
//! The record of what the enforcement planes decided (`governance`), the
//! declared policy surface they decided it from (`governance_chain`,
//! `gateway_policies`, `gateway_routes`, `config`), the secrets vault, the
//! per-plugin environment variables, and the sync planes that project the
//! declared side into the database.
//!
//! Callers path-qualify (`repositories::governance::approvals::…`); nothing is
//! re-exported, so the module path is the only name a symbol has.

pub mod config;
pub mod gateway_policies;
pub mod gateway_routes;
pub mod governance;
pub mod governance_chain;
pub mod marketplace;
pub mod secrets;
pub mod sync_planes;
