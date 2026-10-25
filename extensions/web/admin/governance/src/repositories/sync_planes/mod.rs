//! The governance half of the sync system.
//!
//! The gateway-routes plane, an install plane whose declaration projects into
//! a table this crate owns, registered by an `inventory::submit!` beside its
//! `impl`. The governance chain and the gateway policies are core's planes.
//!
//! The engine they plug into — `SyncPlane`, the registry, boot, the archive,
//! sources, state and provenance — lives in
//! `systemprompt_web_admin_core::repositories::sync`.

pub mod gateway_routes;
mod gateway_routes_drift;
