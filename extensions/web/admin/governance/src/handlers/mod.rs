//! HTTP handlers for the governance domain.
//!
//! SSR page handlers live in [`ssr`]; everything else here is JSON API or
//! webhook intake. Handlers own status mapping and call repositories for data.

pub(crate) mod approvals;
pub(crate) mod gateway;
pub(crate) mod gateway_catalog;
pub(crate) mod plugins_env;
pub(crate) mod secrets;
pub(crate) mod ssr;
pub(crate) mod webhook;

pub(crate) use gateway::gateway_config_view;
