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
//! decision; they
//! exist because the gateway's quota resolver finds subjects only through this
//! registry, so without them a `subject: role` or `subject: organization`
//! window has nothing to key on. Which providers exist is decided by the crates
//! linked into the binary: the registry discovers them once per database and
//! caches the result, so every resolve call site reads the same provider list
//! and ladder through [`subject_attributes_for`] and [`dimensions`].
//! [`account`], [`catalog`] and [`primary`] are the helpers those call sites
//! share. Every database-backed provider registers through [`cached`], so a
//! user's values are read once per 30 s rather than once per request.

pub mod account;
pub mod cached;
pub mod catalog;
pub mod connector;
pub mod organization;
pub mod primary;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzHookContext, NullAuditSink, SharedSubjectAttributeProvider, SubjectAttributes,
    SubjectDimension, dimensions_of, discover_subject_providers, gather_subject_attributes,
};

struct Registry {
    providers: Vec<SharedSubjectAttributeProvider>,
    dimensions: Vec<SubjectDimension>,
}

static REGISTRIES: OnceLock<Mutex<HashMap<String, &'static Registry>>> = OnceLock::new();

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

fn registry(pool: &PgPool) -> &'static Registry {
    let key = database_key(pool);
    let map = REGISTRIES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(existing) = guard.get(&key) {
        return existing;
    }
    let providers = discover_subject_providers(&AuthzHookContext {
        pool: Arc::new(pool.clone()),
        sink: Arc::new(NullAuditSink),
    });
    let built: &'static Registry = Box::leak(Box::new(Registry {
        dimensions: dimensions_of(&providers),
        providers,
    }));
    guard.insert(key, built);
    built
}

pub fn dimensions(pool: &PgPool) -> &'static [SubjectDimension] {
    &registry(pool).dimensions
}

pub async fn subject_attributes_for(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<SubjectAttributes, sqlx::Error> {
    gather_subject_attributes(&registry(pool).providers, user_id)
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))
}
