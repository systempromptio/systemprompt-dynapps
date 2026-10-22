//! Data access owned by the platform domain.
//!
//! Device credentials and bridge sessions (`devices`), per-user connector
//! state (`connectors`),
//! the scheduled-jobs listing (`jobs`), and service-account group membership
//! (`service_account_group`).
//!
//! Callers path-qualify (`repositories::devices::pats::…`); nothing is
//! re-exported, so the module path is the only name a symbol has.

pub mod connectors;
pub mod devices;
pub mod jobs;
pub mod service_account_group;

pub mod data_console;
