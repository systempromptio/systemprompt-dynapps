//! The authorization caches one database's enforcement points share.
//!
//! Entity rules, user roles and parent chains are read on every gateway
//! request by the in-process authz hook, the route entitlement guard and the
//! `/govern/authz` webhook; the subject attribute providers cache their own
//! values the same way. All of them sit on one [`Generation`], so
//! [`AuthzCaches::invalidate_all`] after an access-control sync apply or a
//! console rule edit drops every entry on this replica at once.

use std::sync::Arc;

use systemprompt::governance::authz::{AccessRule, EntityRow, ParentChainIndex};
use systemprompt::identifiers::UserId;

use crate::ttl_cache::{AUTHZ_TTL, Generation, TtlCache};

pub type EntityGrant = (Option<EntityRow>, Vec<AccessRule>);

#[derive(Debug)]
pub struct AuthzCaches {
    generation: Arc<Generation>,
    pub entity_grants: TtlCache<(String, String), EntityGrant>,
    pub user_roles: TtlCache<UserId, Vec<String>>,
    pub parent_chains: TtlCache<(), ParentChainIndex>,
}

impl Default for AuthzCaches {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthzCaches {
    #[must_use]
    pub fn new() -> Self {
        let generation = Arc::new(Generation::default());
        Self {
            entity_grants: TtlCache::new(AUTHZ_TTL, Arc::clone(&generation)),
            user_roles: TtlCache::new(AUTHZ_TTL, Arc::clone(&generation)),
            parent_chains: TtlCache::new(AUTHZ_TTL, Arc::clone(&generation)),
            generation,
        }
    }

    #[must_use]
    pub fn generation(&self) -> Arc<Generation> {
        Arc::clone(&self.generation)
    }

    pub fn invalidate_all(&self) {
        self.generation.bump();
    }
}
