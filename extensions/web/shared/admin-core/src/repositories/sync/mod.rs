//! Code ↔ Instance: one sync system for every declared service.
//!
//! A **source** is where declarations come from (`base` is this tree's
//! `services/`, `bundle:<name>` a pinned remote kit), each with a content
//! hash. A **plane** is what those declarations project into the database.
//! Nothing here writes on its own: a plane applies a direction only when an
//! administrator chooses one, and [`state`] records what was applied.

pub mod access_control;
pub mod access_control_rows;
pub mod archive;
pub mod attention;
pub mod boot;
pub mod declaration;
pub mod drift_shape;
pub mod history;
pub mod inventory;
pub mod marketplace_hash;
pub mod marketplace_versions_db;
pub mod plane;
pub mod plane_ids;
pub mod provenance;
pub mod registry;
pub mod source_badges;
pub mod sources;
pub mod sources_db;
pub mod state;
pub mod tree_hash;
