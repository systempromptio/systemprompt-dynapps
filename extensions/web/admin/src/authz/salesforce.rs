//! Current database-backed salesforce attributes for authorization.

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SubjectAttributeProvider, SubjectDimension,
};

const SALESFORCE_SLUG: &str = "salesforce";

// Why: above `group` (150) and below core's `ROLE` (200). A link rule
// must out-rank the role band that grants the marketplace to every user, while
// remaining a narrower scope than a role.
const SALESFORCE_PRECEDENCE: u16 = 170;

pub const SALESFORCE_LINKED_VALUE: &str = "linked";

#[must_use]
pub fn salesforce_rule_type() -> RuleType {
    RuleType::extension(SALESFORCE_SLUG)
        .unwrap_or_else(|e| unreachable!("`{SALESFORCE_SLUG}` is a well-formed slug: {e}"))
}

#[must_use]
pub fn salesforce_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: salesforce_rule_type(),
        label: "Salesforce link",
        precedence: SALESFORCE_PRECEDENCE,
    }
}

#[expect(
    clippy::unused_async,
    reason = "Retains the public invalidation API after removing membership caches"
)]
pub async fn invalidate(_user_id: &UserId) {}

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

    async fn values_for(&self, user_id: &UserId) -> Vec<String> {
        self.try_values_for(user_id).await.unwrap_or_default()
    }

    async fn try_values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        let linked = crate::repositories::users::salesforce_identity::is_salesforce_linked(
            &self.pool, user_id,
        )
        .await?;
        Ok(if linked {
            vec![SALESFORCE_LINKED_VALUE.to_owned()]
        } else {
            Vec::new()
        })
    }
}
