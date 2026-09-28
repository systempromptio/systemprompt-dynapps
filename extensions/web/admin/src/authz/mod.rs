//! Subject dimensions this extension adds to core's authorization resolver.
//!
//! Core resolves `user` and `role` and deliberately knows nothing else. Every
//! other dimension an operator wants to write rules against is a tenant
//! concept, declared here: a [`SubjectDimension`] describing where it sits in
//! the precedence ladder, and a [`SubjectAttributeProvider`][p] that looks up
//! the values a user holds for it.
//!
//! We declare five: [`project`], [`group`], [`connector`], [`salesforce`] and
//! [`organization`]. They form a ladder with core's — user (0), `project`
//! (140), `group` (150), `connector` (160), `salesforce` (170), role (200),
//! `organization` (300)
//! — where a lower number is the narrower, higher-priority scope. Groups
//! carry people and marketplace entitlement; projects are work attribution.
//! Both are DB rows an operator edits, and the directory's AD groups map into
//! them rather than being a dimension of their own. A fifth provider,
//! [`role`], re-exposes core's own `role` dimension: the resolver ignores it,
//! but the gateway's quota resolver finds subjects only through this registry,
//! so without it a `subject: role` window has nothing to key on. The same
//! goes for `organization`, which is why it exists as a dimension at all.
//! Adding another — cost centre, clearance,
//! jurisdiction — means writing a provider beside them and one
//! `register_subject_attribute_provider!` call; no core change, and no edit to
//! the resolve call sites, because they all read the registry through
//! [`subject_attributes_for`] and [`dimensions`].
//!
//! [p]: systemprompt_security::authz::SubjectAttributeProvider

mod account;
pub use account::account_scope;
pub(crate) mod catalog;
pub mod connector;
pub mod group;
pub mod organization;
pub mod primary;
pub mod project;
pub mod role;
mod rule_type;
pub mod salesforce;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzHookContext, NullAuditSink, SharedSubjectAttributeProvider, SubjectAttributes,
    SubjectDimension, dimensions_of, discover_subject_providers, gather_subject_attributes,
};

use crate::authz::connector::ConnectorAttributeProvider;
use crate::authz::group::GroupAttributeProvider;
use crate::authz::organization::OrganizationAttributeProvider;
use crate::authz::project::ProjectAttributeProvider;
use crate::authz::role::RoleAttributeProvider;
use crate::authz::salesforce::SalesforceAttributeProvider;

systemprompt_security::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        Arc::new(GroupAttributeProvider::new(Arc::clone(&ctx.pool)));
    provider
});

systemprompt_security::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        Arc::new(ProjectAttributeProvider::new(Arc::clone(&ctx.pool)));
    provider
});

systemprompt_security::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        Arc::new(ConnectorAttributeProvider::new(Arc::clone(&ctx.pool)));
    provider
});

systemprompt_security::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        Arc::new(SalesforceAttributeProvider::new(Arc::clone(&ctx.pool)));
    provider
});

systemprompt_security::register_subject_attribute_provider!(|_ctx| {
    let provider: SharedSubjectAttributeProvider = Arc::new(OrganizationAttributeProvider);
    provider
});

systemprompt_security::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        Arc::new(RoleAttributeProvider::new(Arc::clone(&ctx.pool)));
    provider
});

struct Registry {
    providers: Vec<SharedSubjectAttributeProvider>,
    dimensions: Vec<SubjectDimension>,
}

static REGISTRIES: OnceLock<Mutex<HashMap<String, &'static Registry>>> = OnceLock::new();

// Why: providers capture the pool they are built with, so the registry is
// keyed per database; a process that opens several (the integration suite's
// throwaway databases) would otherwise bind every caller to the first pool.
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

pub async fn subject_attributes_for(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<SubjectAttributes, sqlx::Error> {
    gather_subject_attributes(&registry(pool).providers, user_id)
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))
}
