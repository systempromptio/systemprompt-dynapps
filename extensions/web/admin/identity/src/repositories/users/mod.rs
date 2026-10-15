//! Persistence for users: identity, access, activity, devices, and usage.

pub mod enrolment;
pub mod federated;
mod federated_provision;
pub mod mutations;
pub mod queries;
pub mod revocation;
pub mod role_grants;
pub mod roles;
pub mod roster;
pub mod sessions;
pub mod user_settings;

pub use mutations::{archive_user, create_user, restore_user, update_user};
