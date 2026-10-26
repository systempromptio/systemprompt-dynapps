//! The access-control model as data: what the file declares, what the
//! database holds, and how the two differ.
//!
//! The rule CRUD lives in [`crate::repositories::entitlement`] and
//! stays there. This module owns the declarative side: [`declared`] projects
//! `rules.yaml` into database vocabulary, [`drift`] compares that against the
//! tables, and [`review`] regroups that drift by entity for a person to
//! settle. Writing the declaration in and rendering the tables back out is
//! core's access-control plane. [`rules`] is the
//! read surface they and the console page share; [`rules_yaml_types`] and
//! [`rules_yaml_loader`] parse the file itself.

pub mod declared;
pub mod declared_load;
pub mod drift;
pub mod orphan;
pub mod review;
pub mod rules;
pub mod rules_yaml_loader;
pub mod rules_yaml_types;
mod rules_yaml_validate;
pub mod validity;
