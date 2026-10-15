//! Platform domain of the admin dashboard.
//!
//! How a person's clients and tools reach this instance, and whether its
//! configuration is in step with code.
//!
//! - [`repositories`] — device credentials and bridge sessions, connector
//!   accounts and their sealed grants, the declared security policy, and the
//!   scheduled-jobs listing.
//! - [`routes`] — the route fragments the admin composition root merges: the
//!   devices, connect, connectors, setup, configuration and sync pages, the
//!   sync, device and connector API, the bridge sign-in pages, and the routers
//!   that mount outside the admin session.
//! - [`connector_oauth`], [`identity_token`] — the hosted-MCP OAuth broker and
//!   the identity tokens minted for MCP servers.
//! - [`machine_credentials`], [`service_accounts`] — the non-interactive
//!   accounts workflow write-backs act as, and the provider credential each
//!   holds.
//! - [`odoo_demo`] — restores the demo-* Odoo records the console's Odoo demo
//!   runs over, as the farm service account.
//! - [`test_support`] — the pure helpers the test workspace exercises.
//!
//! The connector accounts register as core's connection snapshot source
//! through `inventory`, so a binary that serves connections must link this
//! crate.

pub(crate) mod handlers;
pub mod repositories;
pub mod routes;
pub(crate) mod services;

pub use routes::public::{bridge_identity_router, connector_api_router};
pub use services::{
    connector_oauth, identity_token, machine_credentials, odoo_demo, service_accounts,
};

pub mod test_support {
    pub use crate::handlers::ssr::ssr_connectors_cards::{
        ConnectorCardView, card as connector_card,
    };
}
