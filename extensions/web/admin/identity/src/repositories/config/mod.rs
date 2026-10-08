//! The declared directory in `services/groups.yaml`.
//!
//! Its types and the loader that validates and applies it. Seeded once at
//! boot and otherwise only compared against the database by the `groups` sync
//! plane.

pub mod groups_yaml_loader;
pub mod groups_yaml_types;
