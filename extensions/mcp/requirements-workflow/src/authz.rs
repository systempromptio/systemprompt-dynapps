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
}

impl MembershipProvider {
    pub fn groups(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            project: false,
        }
    }
    pub fn projects(pool: Arc<PgPool>) -> Self {
        Self {
            pool,
            project: true,
        }
    }
}

#[async_trait]
impl SubjectAttributeProvider for MembershipProvider {
    fn dimension(&self) -> SubjectDimension {
        let (slug, label, precedence) = if self.project {
            ("project", "Project", 140)
        } else {
            ("group", "Group", 150)
        };
        SubjectDimension {
            rule_type: RuleType::extension(slug).expect("Fixed membership dimension"),
            label,
            precedence,
        }
    }

    async fn values_for(&self, user: &UserId) -> Vec<String> {
        let result = if self.project {
            sqlx::query_scalar("SELECT project_id FROM project_members WHERE user_id=$1")
                .bind(user.as_str())
                .fetch_all(&*self.pool)
                .await
        } else {
            sqlx::query_scalar("SELECT group_id FROM group_members WHERE user_id=$1")
                .bind(user.as_str())
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
