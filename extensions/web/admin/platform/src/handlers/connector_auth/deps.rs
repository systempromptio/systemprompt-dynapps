//! The request parts every consent and bearer handler extracts together: the
//! boot values a login is checked against and secrets are read from, the
//! service-account directory and the booted services view the connector is
//! resolved from.

use axum::Extension;
use axum::extract::FromRequestParts;
use axum::extract::rejection::ExtensionRejection;
use axum::http::request::Parts;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;
use systemprompt_web_admin_core::extension_services::ExtensionServices;

use crate::services::service_accounts::ServiceAccounts;

#[derive(Debug, Clone)]
pub(crate) struct ConsentDeps {
    pub(super) boot: ExtensionBoot,
    pub(super) service_accounts: ServiceAccounts,
    pub(super) services: ExtensionServices,
}

impl<S: Send + Sync> FromRequestParts<S> for ConsentDeps {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(boot) = Extension::from_request_parts(parts, state).await?;
        let Extension(service_accounts) = Extension::from_request_parts(parts, state).await?;
        let Extension(services) = Extension::from_request_parts(parts, state).await?;
        Ok(Self {
            boot,
            service_accounts,
            services,
        })
    }
}
