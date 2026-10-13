//! The `farm` subject dimension (rung 270): the farm a gateway request is
//! reported under.
//!
//! A user holds the farms of their runs that are queued, preparing or
//! running, so a run's credentials can attribute every gateway call to its
//! farm while the run lives and lose that value once it ends. The dimension
//! is header-only: the run request binds it as an attribution scope, and a
//! request without that header carries no farm. It changes no access
//! decision. The provider is not cached, since a run's lifetime is shorter
//! than any cache window would be.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SharedSubjectAttributeProvider, SubjectAttributeProvider,
    SubjectDimension,
};

use crate::repositories::runs;

pub const FARM_DIMENSION: &str = "farm";

const FARM_RULE_TYPE: RuleType = RuleType::extension_static(FARM_DIMENSION);

const FARM_PRECEDENCE: u16 = 270;

#[must_use]
pub const fn farm_rule_type() -> RuleType {
    FARM_RULE_TYPE
}

#[must_use]
pub const fn farm_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: farm_rule_type(),
        label: "Farm",
        precedence: FARM_PRECEDENCE,
    }
}

#[derive(Debug)]
pub struct FarmAttributeProvider {
    pool: Arc<PgPool>,
}

impl FarmAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubjectAttributeProvider for FarmAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        farm_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        Ok(runs::list_active_farm_ids_for_user(&self.pool, user_id).await?)
    }

    fn default_attribution(&self) -> bool {
        false
    }
}

systemprompt_security::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        Arc::new(FarmAttributeProvider::new(Arc::clone(&ctx.pool)));
    provider
});
