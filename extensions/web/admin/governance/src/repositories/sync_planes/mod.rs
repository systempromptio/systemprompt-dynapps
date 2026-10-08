//! The governance half of the sync system.
//!
//! These are the planes whose declarations project into tables this crate owns,
//! each registered with the core registry by an `inventory::submit!` beside its
//! `impl` (orders 2, 3 and 4).
//!
//! The engine they plug into — `SyncPlane`, the registry, boot, the archive,
//! sources, state and provenance — lives in
//! `systemprompt_web_admin_core::repositories::sync`.

pub mod gateway_policies;
pub mod gateway_routes;
pub mod governance;
