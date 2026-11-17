//! Unit tests for `dynwork-chats` pure logic:
//! - thread and message ids: minted shape, ordering, what a client may supply
//! - the changes-feed cursor: round trip and what it refuses
//! - the extension registration, the error keys and their HTTP statuses
//! - the HTTP helpers: `If-Match`, page sizes, the thread body, the wire shapes
//!   of a thread and a message
//! - the memory store's own edges (page limits, the cursor it hands back)
//!
//! The store contract itself runs against both stores in the integration
//! suite.

#[cfg(test)]
mod errors;
#[cfg(test)]
mod extension;
#[cfg(test)]
mod http;
#[cfg(test)]
mod ids;
#[cfg(test)]
mod memory;
