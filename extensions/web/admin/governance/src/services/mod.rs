//! Services behind the per-plugin secrets API: plugin JWT validation and the
//! resolution-token flow over the secrets vault.

pub(crate) mod auth;
pub(crate) mod secret_service;
