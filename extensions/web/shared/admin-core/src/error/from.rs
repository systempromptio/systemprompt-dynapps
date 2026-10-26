//! Domain and core error conversions into `AdminError`. Each impl picks the
//! variant, and the variant alone decides the status, so a typed validation
//! failure becomes a 400 here rather than at the call site.

use super::AdminError;
use crate::repositories::bridge::BridgeRepoError;
use crate::templates::AdminTemplateError;
use systemprompt_web_shared::error::MarketplaceError;

impl From<AdminTemplateError> for AdminError {
    fn from(value: AdminTemplateError) -> Self {
        Self::Internal(Box::new(value))
    }
}

impl From<BridgeRepoError> for AdminError {
    fn from(value: BridgeRepoError) -> Self {
        Self::BridgeRepo(value)
    }
}

impl From<MarketplaceError> for AdminError {
    fn from(value: MarketplaceError) -> Self {
        Self::Marketplace(value)
    }
}

impl From<systemprompt::oauth::OauthError> for AdminError {
    fn from(value: systemprompt::oauth::OauthError) -> Self {
        Self::Unauthenticated(Box::new(value))
    }
}

impl From<systemprompt::governance::authz::AuthzError> for AdminError {
    fn from(value: systemprompt::governance::authz::AuthzError) -> Self {
        match value {
            systemprompt::governance::authz::AuthzError::Validation(msg) => Self::BadRequest(msg),
            other => Self::Internal(Box::new(other)),
        }
    }
}

impl From<crate::authz::AuthzReadError> for AdminError {
    fn from(value: crate::authz::AuthzReadError) -> Self {
        match value {
            crate::authz::AuthzReadError::Database(e) => Self::Database(e),
            other => Self::Internal(Box::new(other)),
        }
    }
}

impl From<systemprompt::models::errors::GlobalConfigError> for AdminError {
    fn from(value: systemprompt::models::errors::GlobalConfigError) -> Self {
        Self::Internal(Box::new(value))
    }
}

impl From<systemprompt::config::ProfileBootstrapError> for AdminError {
    fn from(value: systemprompt::config::ProfileBootstrapError) -> Self {
        Self::Internal(Box::new(value))
    }
}

impl From<systemprompt::loader::ConfigLoadError> for AdminError {
    fn from(value: systemprompt::loader::ConfigLoadError) -> Self {
        Self::Internal(Box::new(value))
    }
}

impl From<systemprompt::analytics::AnalyticsError> for AdminError {
    fn from(error: systemprompt::analytics::AnalyticsError) -> Self {
        match error {
            systemprompt::analytics::AnalyticsError::InvalidArgument(message) => {
                Self::BadRequest(message)
            },
            error => Self::Internal(Box::new(error)),
        }
    }
}

impl From<systemprompt::models::authz::RuleTypeError> for AdminError {
    fn from(value: systemprompt::models::authz::RuleTypeError) -> Self {
        Self::BadRequest(value.0)
    }
}

impl From<systemprompt::models::authz::SubjectSourceError> for AdminError {
    fn from(value: systemprompt::models::authz::SubjectSourceError) -> Self {
        match value {
            systemprompt::models::authz::SubjectSourceError::NotReady => {
                Self::Unavailable("Authorization is not ready yet".into())
            },
            other => Self::Internal(Box::new(other)),
        }
    }
}
