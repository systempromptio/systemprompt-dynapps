//! The installation-wide subject: every user holds the one value `default`.
//!
//! A quota window keyed on `subject: organization` needs a provider to name
//! the bucket it counts into, and under `quota_fault_mode: closed` a window
//! with no provider refuses every request. This is that provider. The value
//! is a constant because this installation is one tenant; a multi-org estate
//! would read it off the user's row instead.

use async_trait::async_trait;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzError, RuleType, SubjectAttributeProvider, SubjectDimension,
};

use crate::authz::rule_type::{ExtensionSlug, extension_rule_type};

const ORGANIZATION_SLUG: ExtensionSlug = ExtensionSlug::new("organization");

pub const ORGANIZATION_DEFAULT: &str = "default";

const ORGANIZATION_PRECEDENCE: u16 = 300;

#[must_use]
pub fn organization_rule_type() -> RuleType {
    extension_rule_type(ORGANIZATION_SLUG)
}

#[must_use]
pub fn organization_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: organization_rule_type(),
        label: "Organization",
        precedence: ORGANIZATION_PRECEDENCE,
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct OrganizationAttributeProvider;

#[async_trait]
impl SubjectAttributeProvider for OrganizationAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        organization_dimension()
    }

    async fn values_for(&self, _user_id: &UserId) -> Result<Vec<String>, AuthzError> {
        Ok(vec![ORGANIZATION_DEFAULT.to_owned()])
    }
}
