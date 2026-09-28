//! The domain half of the sync system: the planes whose declarations project
//! into tables this crate owns, each registered with the core registry by an
//! `inventory::submit!` beside its `impl`.
//!
//! The engine they plug into — `SyncPlane`, the registry, boot, the archive,
//! sources, state, provenance and the access-control plane — lives in
//! `systemprompt_web_admin_core::repositories::sync`.

pub mod gateway_policies;
pub mod gateway_routes;
pub mod governance;
pub mod groups;
pub mod groups_db;
pub mod groups_drift;
