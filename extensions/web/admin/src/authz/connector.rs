//! Current database-backed connector attributes for authorization.
//!
//! The dimension's values are the MCP server ids the user holds a live,
//! verified connection for on the Connections page (`mcp_connector_accounts`
//! with `status = 'connected'`), so a rule written at `rule_value = <server
//! id>` opens an entity only to the people whose calls to that server will
//! actually work. It differs from the `salesforce` band, which is the
//! admin-linked Salesforce Username used for JWT-bearer minting rather than a
//! consented OAuth grant.

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SubjectAttributeProvider, SubjectDimension,
};

const CONNECTOR_SLUG: &str = "connector";

// Why: above `group` (150) and below `salesforce` (170). A connection rule is
// narrower than group membership — it names people, not a team — but a linked
// Salesforce Username is a stronger, admin-vouched identity, so it still wins.
const CONNECTOR_PRECEDENCE: u16 = 160;

#[must_use]
pub fn connector_rule_type() -> RuleType {
    RuleType::extension(CONNECTOR_SLUG)
        .unwrap_or_else(|e| unreachable!("`{CONNECTOR_SLUG}` is a well-formed slug: {e}"))
}

#[must_use]
pub fn connector_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: connector_rule_type(),
        label: "Connected server",
        precedence: CONNECTOR_PRECEDENCE,
    }
}

#[derive(Debug)]
pub struct ConnectorAttributeProvider {
    pool: Arc<PgPool>,
}

impl ConnectorAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubjectAttributeProvider for ConnectorAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        connector_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        Ok(
            crate::repositories::users::connector_accounts::list_connected_providers(
                &self.pool, user_id,
            )
            .await?,
        )
    }
}
