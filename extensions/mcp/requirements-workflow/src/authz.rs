//! Tenant membership dimensions for the standalone MCP binary.
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt::security::authz::{RuleType, SubjectAttributeProvider, SubjectDimension};

#[derive(Debug)]
pub struct MembershipProvider {
    pool: Arc<PgPool>,
    project: bool,
    dimension: SubjectDimension,
}

impl MembershipProvider {
    pub fn groups(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            project: false,
            dimension: SubjectDimension {
                rule_type: RuleType::from("group"),
                label: "Group",
                precedence: 150,
            },
        }
    }
    pub fn projects(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            project: true,
            dimension: SubjectDimension {
                rule_type: RuleType::from("project"),
                label: "Project",
                precedence: 140,
            },
        }
    }
}

#[async_trait]
impl SubjectAttributeProvider for MembershipProvider {
    fn dimension(&self) -> SubjectDimension {
        self.dimension.clone()
    }

    async fn values_for(&self, user: &UserId) -> Vec<String> {
        let result = if self.project {
            sqlx::query_scalar!(
                "SELECT project_id FROM project_members WHERE user_id=$1",
                user.as_str()
            )
            .fetch_all(&*self.pool)
            .await
        } else {
            sqlx::query_scalar!(
                "SELECT group_id FROM group_members WHERE user_id=$1",
                user.as_str()
            )
            .fetch_all(&*self.pool)
            .await
        };
        match result {
            Ok(values) => values,
            Err(error) => {
                tracing::warn!(%error, "Workflow membership lookup failed");
                Vec::new()
            },
        }
    }
}
