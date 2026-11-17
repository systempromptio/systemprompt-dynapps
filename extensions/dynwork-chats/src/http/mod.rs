//! The HTTP layer of `/api/dynwork/chats/v1`.
//!
//! The threads, the changes feed and the message history. Every handler takes a
//! [`crate::auth::Caller`] first, so the bearer is checked before a path, a
//! query or a body is read.

pub mod changes;
pub mod messages;
pub mod params;
pub mod response;
pub mod threads;
pub mod wire;
