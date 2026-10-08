//! Per-user connector state.
//!
//! The account row each hosted MCP server's connection reports, the sealed
//! OAuth grant behind it, and the Salesforce Username an administrator linked
//! for the JWT-bearer grant.

pub mod connector_accounts;
pub mod connector_credentials;
pub mod salesforce_identity;
