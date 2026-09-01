//! Subject dimensions this extension adds to core's authorization resolver.
//!
//! Core resolves `user` and `role` and deliberately knows nothing else. Every
//! other dimension an operator wants to write rules against is a tenant
//! concept, declared here: a [`SubjectDimension`] describing where it sits in
//! the precedence ladder, and a [`SubjectAttributeProvider`][p] that looks up
//! the values a user holds for it.
//!
//! We declare three: [`adfs_group`], [`atlassian`], and [`salesforce`]. They
//! form a ladder with core's — user (0), `adfs_group` (150), `atlassian`
//! (160), `salesforce` (170), role (200) — where a lower number is the
//! narrower, higher-priority scope. A user's *project* (commerce | core) is
//! not a dimension of its own: it is the name of an AD group, and
//! `adfs_group` already binds rules to those. Adding another — cost centre,
//! clearance, jurisdiction — means writing a provider beside them and one
//! `register_subject_attribute_provider!` call; no core change, and no edit to
//! the resolve call sites, because they all read the registry through
//! [`subject_attributes_for`] and [`dimensions`].
//!
//! [p]: systemprompt_security::authz::SubjectAttributeProvider

pub mod adfs_group;
pub mod atlassian;
pub mod salesforce;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzHookContext, NullAuditSink, SharedSubjectAttributeProvider, SubjectAttributes,
    SubjectDimension, dimensions_of, discover_subject_providers, gather_subject_attributes,
};

use crate::authz::adfs_group::AdfsGroupAttributeProvider;
use crate::authz::atlassian::AtlassianAttributeProvider;
use crate::authz::salesforce::SalesforceAttributeProvider;

systemprompt_security::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        Arc::new(AdfsGroupAttributeProvider::new(Arc::clone(&ctx.pool)));
    provider
});

systemprompt_security::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        Arc::new(AtlassianAttributeProvider::new(Arc::clone(&ctx.pool)));
    provider
});

systemprompt_security::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        Arc::new(SalesforceAttributeProvider::new(Arc::clone(&ctx.pool)));
    provider
});

struct Registry {
    providers: Vec<SharedSubjectAttributeProvider>,
    dimensions: Vec<SubjectDimension>,
}

static REGISTRIES: OnceLock<Mutex<HashMap<String, &'static Registry>>> = OnceLock::new();

// Why: keyed per database, not once per process. The providers capture the
// pool they are built with, so a single OnceLock would bind every later
// caller to whichever pool arrived first — wrong in any process that talks
// to more than one database, which is exactly what the integration suite's
// per-test throwaway databases do. One registry per distinct database; the
// leak is bounded by the number of distinct databases a process ever opens
// (one in production).
// Why: shared with the marketplace-parent cache in the authz webhook so the
// two per-database keying schemes cannot drift apart.
pub(crate) fn database_key(pool: &PgPool) -> String {
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

// Why: The subject's values for every registered dimension. The one async step
// in the authorization path; call it once per request and reuse the result
// across entities.
pub async fn subject_attributes_for(pool: &PgPool, user_id: &UserId) -> SubjectAttributes {
    gather_subject_attributes(&registry(pool).providers, user_id).await
}
