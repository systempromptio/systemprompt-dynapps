//! Data access shared by every admin domain.
//!
//! Callers path-qualify (`repositories::scope::membership::get_subject_scope`);
//! this module re-exports nothing, so the module path is the only name a
//! symbol has.

pub mod access_rules;
pub mod bridge;
pub mod conversations;
pub mod dashboard_read;
pub mod entitlement;
pub mod filter_options;
pub mod gateway_catalog;
pub mod ingestion;
pub mod liveness;
pub mod principal;
pub mod scope;
pub mod sync;
pub mod user_activity;
pub mod user_usage;
