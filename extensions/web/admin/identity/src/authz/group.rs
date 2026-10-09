//! Current database-backed group attributes for authorization (rung 150).

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SharedSubjectAttributeProvider, SubjectAttributeProvider,
    SubjectDimension,
};
use systemprompt_web_shared::GroupId;

use systemprompt_web_admin_core::authz::primary::lead_with_primary;

const GROUP_RULE_TYPE: RuleType = RuleType::extension_static("group");

const GROUP_PRECEDENCE: u16 = 150;

#[must_use]
pub const fn group_rule_type() -> RuleType {
    GROUP_RULE_TYPE
}

#[must_use]
pub const fn group_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: group_rule_type(),
        label: "Group",
        precedence: GROUP_PRECEDENCE,
    }
}

#[derive(Debug)]
pub struct GroupAttributeProvider {
    pool: Arc<PgPool>,
}

impl GroupAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubjectAttributeProvider for GroupAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        group_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        let ids: Vec<String> =
            systemprompt_web_admin_core::repositories::principal::group_membership::list_group_ids_for_user(&self.pool, user_id)
                .await?
                .into_iter()
                .map(|id| id.as_str().to_owned())
                .collect();
        let primary =
            systemprompt_web_admin_core::repositories::scope::defaults::find_scope_defaults(
                &self.pool, user_id,
            )
            .await?
            .and_then(|d| d.primary_group_id);
        Ok(lead_with_primary(
            primary.as_ref().map(GroupId::as_str),
            ids,
        ))
    }
}

systemprompt_security::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        systemprompt_web_admin_core::authz::cached::cached_provider(GroupAttributeProvider::new(
            Arc::clone(&ctx.pool),
        ));
    provider
});
