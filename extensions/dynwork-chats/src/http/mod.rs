//! The HTTP layer of `/api/dynwork/chats/v1`.
//!
//! The threads, the changes feed, the message history, the models list,
//! prompts, `interrupt` and the `/log` stream. Every handler takes a
//! [`crate::auth::Caller`] first, so the bearer is checked before a path, a
//! query or a body is read.

pub mod changes;
pub mod log;
pub mod messages;
pub mod models;
pub mod params;
pub mod prompt;
pub mod response;
pub mod threads;
pub mod wire;
