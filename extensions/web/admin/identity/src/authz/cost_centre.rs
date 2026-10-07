//! The `cost_centre` subject dimension (rung 250): the finance code a request
//! is charged to.
//!
//! A cost centre is a property of a project, so a user holds the cost centres
//! of the projects they are a live member of, their primary project's first.
//! Core reads the first value as the request's default attribution and exports
//! it as `systemprompt.scope.cost_centre` on every OTLP span; a caller on more
//! than one cost centre picks another with `x-systemprompt-scope-cost_centre`.
//! It sits between `role` and `organization` because it changes no access
//! decision: it exists for attribution and `subject: cost_centre` quotas.

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SharedSubjectAttributeProvider, SubjectAttributeProvider,
    SubjectDimension,
};

const COST_CENTRE_RULE_TYPE: RuleType = RuleType::extension_static("cost_centre");

const COST_CENTRE_PRECEDENCE: u16 = 250;

#[must_use]
pub const fn cost_centre_rule_type() -> RuleType {
    COST_CENTRE_RULE_TYPE
}

#[must_use]
pub const fn cost_centre_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: cost_centre_rule_type(),
        label: "Cost centre",
        precedence: COST_CENTRE_PRECEDENCE,
    }
}

#[derive(Debug)]
pub struct CostCentreAttributeProvider {
    pool: Arc<PgPool>,
}

impl CostCentreAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubjectAttributeProvider for CostCentreAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        cost_centre_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        Ok(
            crate::repositories::projects::members::list_cost_centres_for_user(&self.pool, user_id)
                .await?,
        )
    }
}

systemprompt_security::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        Arc::new(CostCentreAttributeProvider::new(Arc::clone(&ctx.pool)));
    provider
});
