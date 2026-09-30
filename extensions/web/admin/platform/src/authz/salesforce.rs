//! Current database-backed Salesforce attributes for authorization (rung 170).
//!
//! The dimension's values are the Salesforce server ids the user holds a
//! linked Username for, so a link gate written at `rule_value = <server id>`
//! opens exactly that org's server and no other.

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SharedSubjectAttributeProvider, SubjectAttributeProvider,
    SubjectDimension,
};

const SALESFORCE_RULE_TYPE: RuleType = RuleType::extension_static("salesforce");

const SALESFORCE_PRECEDENCE: u16 = 170;

#[must_use]
pub const fn salesforce_rule_type() -> RuleType {
    SALESFORCE_RULE_TYPE
}

#[must_use]
pub const fn salesforce_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: salesforce_rule_type(),
        label: "Salesforce link",
        precedence: SALESFORCE_PRECEDENCE,
    }
}

#[derive(Debug)]
pub struct SalesforceAttributeProvider {
    pool: Arc<PgPool>,
}

impl SalesforceAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubjectAttributeProvider for SalesforceAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        salesforce_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        Ok(
            crate::repositories::connectors::salesforce_identity::list_linked_providers(
                &self.pool, user_id,
            )
            .await?,
        )
    }
}

systemprompt_security::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        Arc::new(SalesforceAttributeProvider::new(Arc::clone(&ctx.pool)));
    provider
});
