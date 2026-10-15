//! Per-user connector state.
//!
//! The account row each hosted MCP server's connection reports, the sealed
//! OAuth grant behind it, the machine credentials service accounts hold,
//! a person's linked Odoo account, the audit trail of both credentials, and
//! whether a farm run a person's bearer names is active.

pub mod connector_accounts;
pub mod connector_credentials;
pub mod farm_run_callers;
pub mod machine_credentials;
pub mod odoo_user_links;
