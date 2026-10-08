//! The identity half of the sync system.
//!
//! The `groups` plane projects `services/groups.yaml` into the group, mapping
//! and member-set tables this crate owns, registered with the core registry
//! by an `inventory::submit!` beside its `impl` (order 1).
//!
//! The engine it plugs into — `SyncPlane`, the registry, boot, the archive,
//! sources, state and provenance — lives in
//! `systemprompt_web_admin_core::repositories::sync`.

pub mod groups;
pub mod groups_db;
pub mod groups_drift;
