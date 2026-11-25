//! Unit tests for `dynbrain` pure logic:
//! - the inheritance rule: what an ungranted article gives, inheritance down
//!   the tree, own grants replacing the parent's, `inherit = false` cutting
//!   inheritance, highest level wins, the subtree a rewrite covers
//! - levels and subject kinds as the schema stores them
//! - typed properties: the definitions a space may declare and the values a
//!   variant may carry, for every type
//! - language codes, their text-search configuration and the content hash
//! - the extension registration, its schema, and the vocabulary rule: no file
//!   of the crate names the source system the proxy mode talks to
//!
//! The repository runs against Postgres in the integration suite.

#[cfg(test)]
mod access;
#[cfg(test)]
mod basics;
#[cfg(test)]
mod properties;
