//! A subject attribute provider behind a per-user [`TtlCache`].
//!
//! Core calls every registered provider's `values_for` on each gateway
//! request (the entitlement guard and the quota resolver both gather
//! subjects), and each local provider is a database read. The per-database
//! registry wraps every discovered provider here, so the cache is one rule for
//! all of them: values are reused for [`AUTHZ_TTL`] and dropped together by
//! [`crate::authz::AuthzCaches::invalidate_all`], whose [`Generation`] each
//! wrapper shares.

use std::sync::Arc;

use async_trait::async_trait;
use systemprompt::governance::authz::{
    AuthzError, SharedSubjectAttributeProvider, SubjectAttributeProvider, SubjectDimension,
};
use systemprompt::identifiers::UserId;

use crate::ttl_cache::{AUTHZ_TTL, Generation, TtlCache};

#[derive(Debug)]
pub struct CachedSubjectProvider {
    inner: SharedSubjectAttributeProvider,
    values: TtlCache<UserId, Vec<String>>,
}

impl CachedSubjectProvider {
    #[must_use]
    pub fn new(inner: SharedSubjectAttributeProvider, generation: Arc<Generation>) -> Self {
        Self {
            inner,
            values: TtlCache::new(AUTHZ_TTL, generation),
        }
    }
}

#[must_use]
pub fn cached_provider(
    inner: SharedSubjectAttributeProvider,
    generation: Arc<Generation>,
) -> SharedSubjectAttributeProvider {
    Arc::new(CachedSubjectProvider::new(inner, generation))
}

#[async_trait]
impl SubjectAttributeProvider for CachedSubjectProvider {
    fn dimension(&self) -> SubjectDimension {
        self.inner.dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        if let Some(values) = self.values.get(user_id) {
            return Ok(values);
        }
        let values = self.inner.values_for(user_id).await?;
        self.values.insert(user_id.clone(), values.clone());
        Ok(values)
    }
}
