//! Governance domain of the admin dashboard: what the enforcement planes
//! decided, and the configuration they decided it from.
//!
//! - [`repositories`] — the decision record, approvals, the secrets vault and
//!   its audit trail, the governance chain, gateway routes and policies, the
//!   gateway ACL, per-plugin environment variables, and the three sync planes
//!   (`gateway_policies`, `gateway_routes`, `governance`) that project them.
//! - [`routes`] — the route fragments the admin composition root merges: the
//!   governance and gateway pages, the gateway, approval and plugin-env API,
//!   the secrets router, and the `/hooks/govern` and `/govern/authz` webhooks.
//! - [`gateway_entitlement`], [`gateway_safety`] — the gateway guard and the
//!   safety scanners core runs on every `/v1/messages` request.
//!
//! The route guard, the two scanners, the three sync planes and the governance
//! export datasets register with core through `inventory`, so a binary that
//! serves the gateway, sync or exports must link this crate.

mod export;
pub mod gateway_entitlement;
pub mod gateway_safety;
pub(crate) mod handlers;
pub mod repositories;
pub mod routes;
mod services;
