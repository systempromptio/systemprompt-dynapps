//! Per-user connector state.
//!
//! The account row each hosted MCP server's connection reports, the sealed
//! OAuth grant behind it, and the Salesforce Username an administrator linked
//! for the JWT-bearer grant, the machine credentials service accounts hold,
//! a person's linked Odoo account, and the audit trail of both credentials.

pub mod connector_accounts;
pub mod connector_credentials;
pub mod machine_credentials;
pub mod odoo_user_links;
pub mod salesforce_identity;
