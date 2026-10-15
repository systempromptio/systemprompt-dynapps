//! HTTP handlers for the platform domain.
//!
//! SSR page handlers live in [`ssr`]; everything else here is JSON API or an
//! accessor that authenticates its own caller. Handlers own status mapping
//! and call repositories for data.

pub(crate) mod bridge_whoami;
pub(crate) mod connector_auth;
pub(crate) mod devices;
mod jobs;
pub(crate) mod service_accounts;
pub(crate) mod ssr;
pub(crate) mod sync;

pub(crate) use jobs::list_jobs_handler;
