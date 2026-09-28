//! The access-control model as data: what the file declares, what the
//! database holds, how the two differ, and how either becomes the other.
//!
//! The rule CRUD lives in [`crate::repositories::entitlement`] and
//! stays there. This module owns the declarative side: [`declared`] projects
//! `rules.yaml` into database vocabulary, [`drift`] compares that against the
//! tables, [`review`] regroups that drift by entity for a person to settle,
//! [`sync`] writes the declared set in (seed, insert-only, overwrite), and
//! [`export`] renders the tables back out as the file. [`rules`] is the
//! read surface they and the console page share; [`rules_yaml_types`] and
//! [`rules_yaml_loader`] parse the file itself.

pub mod declared;
pub mod declared_load;
pub mod drift;
pub mod export;
pub mod orphan;
pub mod review;
pub mod rules;
pub mod rules_yaml_loader;
pub mod rules_yaml_types;
pub mod sync;
pub mod validity;
