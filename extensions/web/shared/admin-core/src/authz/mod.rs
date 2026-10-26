//! Subject dimensions this installation adds to core's authorization
//! resolver, and the per-database registry that reads them.
//!
//! Core resolves `user` and `role` and deliberately knows nothing else. Every
//! other dimension an operator writes rules against is a tenant concept: a
//! `SubjectDimension` naming its rung, and a `SubjectAttributeProvider` that
//! looks up the values a user holds for it, joined with one
//! `register_subject_attribute_provider!` call beside the provider.
//!
//! The precedence ladder, lower is the narrower, higher-priority scope:
//!
//! | rung | dimension | provider lives in |
//! |---|---|---|
//! | 0 | user | core resolver |
//! | 140 | `project` | the crate owning projects |
//! | 150 | `group` | the crate owning groups |
//! | 160 | `connector` | [`connector`] |
//! | 200 | role | core resolver; re-exposed by the crate owning roles |
//! | 250 | `cost_centre` | the crate owning projects |
//! | 260 | `practice` | the crate owning projects |
//! | 300 | `organization` | [`organization`] |
//!
//! `role`, `cost_centre`, `practice` and `organization` change no access
//! decision; they exist because the gateway's quota resolver finds subjects
//! only through the registered providers, so without them a `subject: role`
//! or `subject: organization` window has nothing to key on. Core discovers the
//! providers once at boot and serves their values through one
//! `SubjectAttributeSource`, which every call site reads through
//! [`crate::extension_services::ExtensionServices`]. [`account`], [`catalog`]
//! and [`primary`] are the helpers those call sites share.
//!
//! [`AuthzCaches`] holds the rule reads (entity grants, user roles, parent
//! chains) for 30 s. Core's hook, guard and router factories share no state an
//! extension can hand between them, so the caches are one process-wide slot
//! per database, keyed by database and built once per key, and one
//! `invalidate_all` drops every entry whichever factory's code issued it.

pub mod account;
pub mod caches;
pub mod catalog;
pub mod connector;
pub mod organization;
pub mod primary;

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use sqlx::PgPool;

pub use caches::{AuthzCaches, EntityGrant};

/// Why an authorization read could not be answered from this database.
#[derive(Debug, thiserror::Error)]
pub enum AuthzReadError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("services configuration unavailable: {0}")]
    Services(#[from] systemprompt::loader::ConfigLoadError),
    #[error("subject attributes unavailable: {0}")]
    Subjects(#[from] systemprompt::governance::authz::AuthzError),
    #[error("subject attributes unavailable: {0}")]
    SubjectSource(#[from] systemprompt::models::authz::SubjectSourceError),
    #[error("subject lookup task failed: {0}")]
    Join(#[from] tokio::task::JoinError),
}

static CACHES: OnceLock<Mutex<HashMap<String, &'static AuthzCaches>>> = OnceLock::new();

#[must_use]
pub fn database_key(pool: &PgPool) -> String {
    let options = pool.connect_options();
    format!(
        "{}:{}/{}",
        options.get_host(),
        options.get_port(),
        options.get_database().unwrap_or_default()
    )
}

pub fn caches(pool: &PgPool) -> &'static AuthzCaches {
    let key = database_key(pool);
    let map = CACHES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    guard
        .entry(key)
        .or_insert_with(|| Box::leak(Box::new(AuthzCaches::new())))
}
