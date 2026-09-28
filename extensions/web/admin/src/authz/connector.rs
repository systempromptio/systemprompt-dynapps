//! Current database-backed connector attributes for authorization.
//!
//! The dimension's values are the MCP server ids whose connection is ready
//! for the user — the same [`Connection::readiness`] the bridge manifest
//! gates on — so a rule written at `rule_value = <server id>` opens an entity
//! only to the people whose calls to that server will actually work, and the
//! manifest never carries a server the rule would refuse. It differs from
//! the `salesforce` band, which is the admin-linked Salesforce Username used
//! for JWT-bearer minting rather than a consented OAuth grant.
//!
//! [`Connection::readiness`]: crate::services::connector_accounts::Connection::readiness

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SubjectAttributeProvider, SubjectDimension,
};

use crate::authz::rule_type::{ExtensionSlug, extension_rule_type};
use crate::services::connector_accounts::Connection;

const CONNECTOR_SLUG: ExtensionSlug = ExtensionSlug::new("connector");

const CONNECTOR_PRECEDENCE: u16 = 160;

#[must_use]
pub fn connector_rule_type() -> RuleType {
    extension_rule_type(CONNECTOR_SLUG)
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
        let snapshot = crate::services::connector_accounts::get_connections(&self.pool, user_id)
            .await
            .map_err(|e| AuthzError::Validation(e.to_string()))?;
        Ok(snapshot
            .connections
            .into_iter()
            .filter(Connection::is_ready)
            .map(|c| c.provider)
            .collect())
    }
}
