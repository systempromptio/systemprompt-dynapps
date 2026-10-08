//! HTTP handlers for the identity domain.
//!
//! SSR page handlers live in [`ssr`]; everything else here is JSON API or
//! the sign-in flows. Handlers own status mapping and call repositories for
//! data.

pub(crate) mod access_control;
pub(crate) mod adfs_auth;
pub(crate) mod dev_login;
pub(crate) mod entity_access;
pub(crate) mod gateway_access;
pub(crate) mod groups;
pub(crate) mod projects;
pub(crate) mod roles;
pub(crate) mod scope_defaults;
pub(crate) mod self_service;
pub(crate) mod ssr;
mod user_sessions;
mod users;
pub(crate) mod users_bootstrap;

pub(crate) use user_sessions::{
    list_user_sessions_handler, revoke_all_user_sessions_handler, revoke_user_session_handler,
};
pub(crate) use users::{
    delete_user_handler, list_users_handler, update_user_handler, user_detail_handler,
    user_usage_handler,
};
