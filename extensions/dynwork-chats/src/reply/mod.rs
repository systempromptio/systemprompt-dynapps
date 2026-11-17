//! Server-side replies.
//!
//! A prompt starts a reply ([`start::start`]) that runs as a task this
//! extension owns ([`runner`]), streaming from the model behind
//! [`model::ReplyModel`] and writing `OpenCode` v2 session events
//! ([`events`]) to the store and to the thread's live feed
//! ([`registry::Replies`]); `/log` followers replay the store and then follow
//! the feed. When the reply ends it is compacted into its assistant message
//! ([`document`]). A reply left unfinished by an earlier process is settled
//! on the first touch of its thread ([`recover`]).

pub mod document;
pub mod events;
pub mod model;
pub mod recover;
pub mod registry;
pub mod runner;
pub mod start;
pub mod tokens;
