//! The console editor over `ai_gateway_policies`.
//!
//! Core re-reads `ai_gateway_policies` on every inference request (a
//! sixty-second cache), so a row written here is live without a restart.
//! The declaration (`services/gateway/policies.yaml`), its drift, apply and
//! export are core's `gateway_policies` plane, driven from the sync page.
//!
//! A calendar-month window is declared `calendar: month` and core resets it on
//! the month boundary; [`windows`] labels windows for the console.

pub mod form;
pub mod form_error;
pub mod rows;
pub mod unedited;
pub mod usage;
pub mod windows;
