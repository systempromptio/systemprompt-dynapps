//! Requirements workflow domain model shared by the MCP server and the admin
//! review screen: versioned phases, validated transitions, review receipts and
//! portable ed25519 receipt verification. Pure types, no database access.
pub mod model;
pub mod signing;

mod transitions;
