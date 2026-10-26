//! The request parts every consent and bearer handler extracts together: the
//! signing keys a login is checked against, the service-account directory and
//! the booted services view the connector is resolved from.

use axum::Extension;
use axum::extract::FromRequestParts;
use axum::extract::rejection::ExtensionRejection;
use axum::http::request::Parts;
use systemprompt::security::keys::SigningKeys;
use systemprompt_web_admin_core::extension_services::ExtensionServices;

use crate::services::service_accounts::ServiceAccounts;

#[derive(Debug, Clone)]
pub(crate) struct ConsentDeps {
    pub(super) keys: SigningKeys,
    pub(super) service_accounts: ServiceAccounts,
    pub(super) services: ExtensionServices,
}

impl<S: Send + Sync> FromRequestParts<S> for ConsentDeps {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(keys) = Extension::from_request_parts(parts, state).await?;
        let Extension(service_accounts) = Extension::from_request_parts(parts, state).await?;
        let Extension(services) = Extension::from_request_parts(parts, state).await?;
        Ok(Self {
            keys,
            service_accounts,
            services,
        })
    }
}
