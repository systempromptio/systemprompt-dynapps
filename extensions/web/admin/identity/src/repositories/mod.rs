//! Data access owned by identity.
//!
//! Users and everything hung off an account (sessions, enrolled devices,
//! federated identities, manual roles, settings, revocation), groups and
//! projects with their members and AD mappings, the role holder and
//! entitlement reads, the people-usage rollups, developer login codes, the
//! `groups.yaml` declaration (`config`) and the `groups` sync plane that
//! projects it (`sync_planes`).
//!
//! Callers path-qualify (`repositories::groups::members::…`); nothing is
//! re-exported, so the module path is the only name a symbol has.

pub mod config;
pub mod dev_login;
pub mod groups;
pub mod people_usage;
pub mod projects;
pub mod provider_credentials;
pub mod roles;
pub mod sync_planes;
pub mod users;
