//! Code ↔ Instance: one sync system for every declared service, local and
//! external.
//!
//! Two vocabularies. A **source** is where declarations come from and carries
//! a content hash — `base` is this repository's `services/` tree, and
//! `bundle:<name>` is each remote kit the profile pins. A **plane** is what a
//! source's declarations project into the database — today `access_control`
//! (`rules.yaml` → `access_control_rules`); `groups` and `gateway_policies`
//! are designed for and not built. Ownership is by id and decided at
//! composition: a bundle owns its marketplaces, plugins and skills; the base
//! owns everything else, every entitlement included.
//!
//! Nothing here writes on its own. [`sources`] reports what is active and
//! what is pinned; each [`plane::SyncPlane`] reports drift and applies a
//! direction only when an administrator chooses one; [`state`] records what
//! was applied, when, by whom and from which declared hash.

pub mod access_control;
pub mod access_control_rows;
pub mod plane;
pub mod registry;
pub mod source_badges;
pub mod sources;
pub mod state;
pub mod tree_hash;
