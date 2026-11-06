//! Document classes: the policy for how far an agent may act on a
//! document.
//!
//! `catalog` is the declared class matrix, `resolve` decides which class
//! governs a target and whether a change of class is allowed, and `policy`
//! is the `AgentAccessPolicy` core asks for an agent's cap.

pub mod catalog;
pub mod movement;
pub mod policy;
mod policy_hooks;
pub mod resolve;

pub use catalog::{CatalogError, ClassCatalog, DeclaredClasses, DocumentClass};
pub use resolve::{
    ChangeKind, ChangeRequest, ClassChain, ClassSource, ResolvedClass, check_change, resolve,
};
