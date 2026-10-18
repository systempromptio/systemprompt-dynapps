//! The gateway policy plane: `services/gateway/policies.yaml` projected into
//! `ai_gateway_policies`, and the console editor over the same table.
//!
//! Core re-reads `ai_gateway_policies` on every inference request (a
//! sixty-second cache), so a row written here is live without a restart —
//! this is the one governance plane the console can change in place. The
//! declaration stays the file: [`declared`] reads it, [`drift`] compares it
//! with the rows, [`export`] renders the rows back as the file, and
//! [`sync::gateway_policies`](crate::repositories::sync_planes::gateway_policies)
//! is the plane the sync page drives.
//!
//! A calendar-month window is declared `calendar: month` and core resets it on
//! the month boundary; [`windows`] labels windows for the console.

pub mod declared;
pub mod drift;
pub mod export;
pub mod form;
pub mod form_error;
pub mod rows;
pub mod unedited;
pub mod usage;
pub mod windows;
