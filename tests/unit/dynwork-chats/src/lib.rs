//! Unit tests for `dynwork-chats` pure logic:
//! - thread and message ids: minted shape, ordering, what a client may supply
//! - the changes-feed cursor: round trip and what it refuses
//! - the extension registration, the error keys and their HTTP statuses
//! - the HTTP helpers: `If-Match`, page sizes, the thread body, the wire shapes
//!   of a thread and a message
//! - model admission: the hook built once and on first use, and every failure
//!   (roles, secrets, hook) refusing instead of listing the catalog
//! - the change hint: which `X-Dynwork-Peer` values name a desktop
//! - the memory store's own edges (page limits, the cursor it hands back)
//! - replies: the event envelope per type, derived ids, finish reasons, the
//!   history sent to the model, usage folding, the prompt and log inputs
//! - model errors: the client gets a kind and a fixed message, never the
//!   provider's text
//!
//! The store contract itself runs against both stores in the integration
//! suite.

#[cfg(test)]
mod admission;
#[cfg(test)]
mod catalog;
#[cfg(test)]
mod config;
#[cfg(test)]
mod errors;
#[cfg(test)]
mod extension;
#[cfg(test)]
mod hint;
#[cfg(test)]
mod http;
#[cfg(test)]
mod ids;
#[cfg(test)]
mod memory;
#[cfg(test)]
mod model_errors;
#[cfg(test)]
mod reply;
