//! Current database-backed connector attributes for authorization (rung 160).
//!
//! The dimension's values are the MCP server ids whose connection is ready
//! for the user — the same [`Connection::readiness`] the bridge manifest
//! gates on — so a rule written at `rule_value = <server id>` opens an entity
//! only to the people whose calls to that server will actually work, and the
//! manifest never carries a server the rule would refuse.
//!
//! [`Connection::readiness`]: crate::connections::Connection::readiness

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::governance::authz::{
    AuthzError, RuleType, SharedSubjectAttributeProvider, SubjectAttributeProvider,
    SubjectDimension,
};
use systemprompt::identifiers::UserId;
use systemprompt::traits::RepositoryError;

use crate::connections::Connection;
use crate::connections::source::get_connections;

const CONNECTOR_RULE_TYPE: RuleType = RuleType::extension_static("connector");

const CONNECTOR_PRECEDENCE: u16 = 160;

#[must_use]
pub const fn connector_rule_type() -> RuleType {
    CONNECTOR_RULE_TYPE
}

#[must_use]
pub const fn connector_dimension() -> SubjectDimension {
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
        let snapshot = get_connections(&self.pool, user_id)
            .await
            .map_err(connection_read_failed)?;
        Ok(snapshot
            .connections
            .into_iter()
            .filter(Connection::is_ready)
            .map(|c| c.provider)
            .collect())
    }
}

// Why: core's `AuthzError` has no variant for an install-side read failure,
// so the admin error travels as the repository cause.
fn connection_read_failed(error: crate::error::AdminError) -> AuthzError {
    AuthzError::Repository(RepositoryError::Decode {
        context: "connector attribute read".to_owned(),
        source: Box::new(error),
    })
}

systemprompt::governance::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        Arc::new(ConnectorAttributeProvider::new(Arc::clone(&ctx.pool)));
    provider
});
