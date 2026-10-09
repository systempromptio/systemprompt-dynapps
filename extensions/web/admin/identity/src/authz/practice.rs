//! The `practice` subject dimension (rung 260): the delivery practice a
//! request is reported under.
//!
//! A practice is a property of a project, so a user holds the practices
//! of the projects they are a live member of, their primary project's first.
//! Core reads the first value as the request's default attribution and exports
//! it as `systemprompt.scope.practice` on every OTLP span; a caller on more
//! than one practice picks another with `x-systemprompt-scope-practice`.
//! It sits between `cost_centre` and `organization` because it changes no
//! access decision: it exists for attribution and `subject: practice` quotas.

use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SharedSubjectAttributeProvider, SubjectAttributeProvider,
    SubjectDimension,
};

const PRACTICE_RULE_TYPE: RuleType = RuleType::extension_static("practice");

const PRACTICE_PRECEDENCE: u16 = 260;

#[must_use]
pub const fn practice_rule_type() -> RuleType {
    PRACTICE_RULE_TYPE
}

#[must_use]
pub const fn practice_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: practice_rule_type(),
        label: "Practice",
        precedence: PRACTICE_PRECEDENCE,
    }
}

#[derive(Debug)]
pub struct PracticeAttributeProvider {
    pool: Arc<PgPool>,
}

impl PracticeAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubjectAttributeProvider for PracticeAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        practice_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        Ok(
            crate::repositories::projects::members::list_practices_for_user(&self.pool, user_id)
                .await?,
        )
    }
}

systemprompt_security::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        systemprompt_web_admin_core::authz::cached::cached_provider(
            PracticeAttributeProvider::new(Arc::clone(&ctx.pool)),
        );
    provider
});
