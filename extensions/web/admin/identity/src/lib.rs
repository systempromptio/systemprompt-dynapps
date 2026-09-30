//! Identity domain of the admin dashboard.
//!
//! Who the people on this instance are, how they sign in, and what their
//! groups, projects and roles grant.
//!
//! - [`repositories`] — users and their sessions, devices, federated
//!   identities, roles and settings; groups and projects with their members and
//!   AD mappings; the people-usage rollups; developer login codes; and the
//!   `groups` sync plane with the `groups.yaml` declaration it projects.
//! - [`routes`] — the route fragments the admin composition root merges: the
//!   users, groups, projects, roles and access-control pages and API, the
//!   profile page, and the AD FS and developer sign-in routes.
//! - [`authz`] — the group, project and role subject-attribute providers.
//! - [`services`] — the bridge profile the profile page renders.
//! - [`sign_out`] — ending the browser session a sign-out request carries.
//!
//! The three providers, the `groups` sync plane and the users, groups and
//! projects export datasets register with core through `inventory`, so a
//! binary that serves authorization, sync or exports must link this crate.

pub mod authz;
mod export;
pub(crate) mod handlers;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod sign_out;

pub use handlers::adfs_auth::{
    ADFS_ACS_PATH, ADFS_METADATA_PATH, AdfsConfig, AdfsDeps, AdfsError, AssertionClaims, FlowState,
    clear_state_cookie, group_matches_pattern, permissions_for_roles, read_state_cookie,
    state_cookie,
};
pub use handlers::dev_login::{
    DEV_LOGIN_PATH, dev_login_allowed, dev_login_enabled, dev_login_url,
};
