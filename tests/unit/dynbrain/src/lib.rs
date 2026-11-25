//! Unit tests for `dynbrain` pure logic:
//! - the access rule: what an ungranted article gives, space grants, own grants
//!   adding to the inherited ones or replacing a subject's level, `none`,
//!   `inherit_access` off cutting, what an article inherits, the owner and
//!   administrators, the subtree a rewrite covers
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
