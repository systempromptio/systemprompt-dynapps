//! Platform domain of the admin dashboard.
//!
//! How a person's clients and tools reach this instance, and whether its
//! configuration is in step with code.
//!
//! - [`repositories`] — device credentials and bridge sessions, connector
//!   accounts and their sealed grants, Salesforce identities, the declared
//!   security policy, and the scheduled-jobs listing.
//! - [`routes`] — the route fragments the admin composition root merges: the
//!   devices, connect, connectors, setup, configuration and sync pages, the
//!   sync, device and connector API, the bridge sign-in pages, and the routers
//!   that mount outside the admin session.
//! - [`connector_oauth`], [`identity_token`], [`salesforce_orgs`] — the
//!   hosted-MCP OAuth broker, the identity tokens minted for MCP servers, and
//!   the Salesforce org registry.
//! - [`test_support`] — the pure helpers the test workspace exercises.
//!
//! The connector accounts register as core's connection snapshot source and
//! the Salesforce link registers as a subject-attribute provider, both
//! through `inventory`, so a binary that serves connections or authorization
//! must link this crate.

pub mod authz;
pub(crate) mod handlers;
pub mod repositories;
pub mod routes;
pub(crate) mod services;

pub use handlers::salesforce_auth::{SalesforceConfig, SalesforceDeps, SalesforceError};
pub use routes::public::{bridge_identity_router, connector_api_router, salesforce_api_router};
pub use services::salesforce_orgs::salesforce_orgs_boot_check;
pub use services::{connector_oauth, identity_token, salesforce_orgs};

pub mod test_support {
    pub use crate::handlers::ssr::ssr_connectors_cards::{
        ConnectorCardView, card as connector_card,
    };
}
