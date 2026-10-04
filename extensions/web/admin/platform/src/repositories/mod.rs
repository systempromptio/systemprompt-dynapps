//! Data access owned by the platform domain.
//!
//! Device credentials and bridge sessions (`devices`), per-user connector
//! state (`connectors`), the declared security policy (`security_policy`),
//! and the scheduled-jobs listing (`jobs`).
//!
//! Callers path-qualify (`repositories::devices::pats::…`); nothing is
//! re-exported, so the module path is the only name a symbol has.

pub mod connectors;
pub mod devices;
pub mod jobs;
pub mod security_policy;

pub mod data_console;
