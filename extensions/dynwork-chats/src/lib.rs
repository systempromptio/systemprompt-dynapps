//! `DynWork` cloud chats, a systemprompt extension.
//!
//! Chat threads are stored, and answered, in the user's own systemprompt
//! instance, so `DynWork` Mobile and every `DynWork` desktop read the same
//! threads whether a desktop is on or not. This crate is the store: threads,
//! messages and the live events of a running reply, behind one trait with a
//! memory and a Postgres implementation. The HTTP API, the replies and the
//! `/log` stream come in later phases of the plan.

pub mod clock;
pub mod error;
pub mod extension;
pub mod ids;
pub mod schema;
pub mod store;

pub use extension::ChatsExtension;

systemprompt::extension::register_extension!(ChatsExtension);
