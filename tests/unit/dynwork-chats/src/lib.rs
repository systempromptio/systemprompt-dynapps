//! Unit tests for `dynwork-chats` pure logic:
//! - thread and message ids: minted shape, ordering, what a client may supply
//! - the changes-feed cursor: round trip and what it refuses
//! - the extension registration and the error keys
//! - the memory store's own edges (page limits, the cursor it hands back)
//!
//! The store contract itself runs against both stores in the integration
//! suite.

#[cfg(test)]
mod errors;
#[cfg(test)]
mod extension;
#[cfg(test)]
mod ids;
#[cfg(test)]
mod memory;
