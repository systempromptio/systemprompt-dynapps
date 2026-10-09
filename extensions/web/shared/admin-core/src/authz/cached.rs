//! A subject attribute provider behind a per-user [`TtlCache`].
//!
//! Core calls every registered provider's `values_for` on each gateway
//! request (the entitlement guard and the quota resolver both gather
//! subjects), and each astound provider is a database read. Wrapping it at
//! registration keeps the provider itself a plain query and makes the cache
//! one rule for all of them: values are reused for [`AUTHZ_TTL`] and dropped
//! by [`crate::ttl_cache::invalidate_all`].

use std::sync::Arc;

use async_trait::async_trait;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, SharedSubjectAttributeProvider, SubjectAttributeProvider, SubjectDimension,
};

use crate::ttl_cache::{AUTHZ_TTL, TtlCache};

#[derive(Debug)]
pub struct CachedSubjectProvider<P> {
    inner: P,
    values: TtlCache<UserId, Vec<String>>,
}

impl<P> CachedSubjectProvider<P> {
    #[must_use]
    pub fn new(inner: P) -> Self {
        Self {
            inner,
            values: TtlCache::new(AUTHZ_TTL),
        }
    }
}

#[must_use]
pub fn cached_provider<P: SubjectAttributeProvider + 'static>(
    inner: P,
) -> SharedSubjectAttributeProvider {
    Arc::new(CachedSubjectProvider::new(inner))
}

#[async_trait]
impl<P: SubjectAttributeProvider> SubjectAttributeProvider for CachedSubjectProvider<P> {
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
