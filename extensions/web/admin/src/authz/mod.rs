//! Subject-attribute providers whose data this crate owns.
//!
//! Each file declares its dimension, its provider and its own
//! `register_subject_attribute_provider!` call; the precedence ladder and the
//! registry that reads every provider live in
//! `systemprompt_web_admin_core::authz`.

pub mod group;
pub mod project;
pub mod role;
pub mod salesforce;
