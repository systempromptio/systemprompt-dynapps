//! The boot-time values an extension reads after its router is built: the
//! profile, the resolved config, the active services root, the deployment's
//! token authority and the live secrets store.
//!
//! Core hands these out through the [`ExtensionContext`] only while the router
//! is being built, so they are taken once there and carried as one value in
//! router state and as a request extension. The secrets store reads live on
//! every call, so a reload is seen without rebuilding anything.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Extension;
use axum::extract::FromRequestParts;
use axum::extract::rejection::ExtensionRejection;
use axum::http::HeaderMap;
use axum::http::request::Parts;
use systemprompt::extension::prelude::ExtensionContext;
use systemprompt::manifest::services::root::ActiveServicesRoot;
use systemprompt::manifest::{Config, Profile};
use systemprompt::models::auth::TokenAuthority;
use systemprompt::traits::SecretStore;

use crate::extension_services::ExtensionServices;

/// The profile, config, services root, token authority and secrets an
/// extension was booted with.
#[derive(Clone)]
pub struct ExtensionBoot {
    profile: Arc<Profile>,
    config: Arc<Config>,
    services_root: ActiveServicesRoot,
    token_authority: Arc<dyn TokenAuthority>,
    secrets: Arc<dyn SecretStore>,
}

impl std::fmt::Debug for ExtensionBoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionBoot")
            .field("services_root", &self.services_root)
            .finish_non_exhaustive()
    }
}

impl ExtensionBoot {
    #[must_use]
    pub fn new(
        profile: Arc<Profile>,
        config: Arc<Config>,
        services_root: ActiveServicesRoot,
        token_authority: Arc<dyn TokenAuthority>,
        secrets: Arc<dyn SecretStore>,
    ) -> Self {
        Self {
            profile,
            config,
            services_root,
            token_authority,
            secrets,
        }
    }

    #[must_use]
    pub fn from_context(ctx: &dyn ExtensionContext) -> Self {
        Self::new(
            ctx.profile(),
            ctx.app_config(),
            ctx.services_root(),
            ctx.token_authority(),
            ctx.secrets(),
        )
    }

    #[must_use]
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    #[must_use]
    pub const fn config_arc(&self) -> &Arc<Config> {
        &self.config
    }

    #[must_use]
    pub fn services_root(&self) -> &Path {
        &self.services_root.path
    }

    #[must_use]
    pub const fn active_services_root(&self) -> &ActiveServicesRoot {
        &self.services_root
    }

    #[must_use]
    pub fn baked_services_root(&self) -> PathBuf {
        PathBuf::from(&self.profile.paths.services)
    }

    #[must_use]
    pub fn token_authority(&self) -> &Arc<dyn TokenAuthority> {
        &self.token_authority
    }

    #[must_use]
    pub fn secrets(&self) -> &Arc<dyn SecretStore> {
        &self.secrets
    }
}

/// The boot values and the booted services view, extracted together by the
/// handlers that read both.
#[derive(Debug, Clone)]
pub struct BootServices {
    pub boot: ExtensionBoot,
    pub services: ExtensionServices,
}

impl<S: Send + Sync> FromRequestParts<S> for BootServices {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(boot) = Extension::from_request_parts(parts, state).await?;
        let Extension(services) = Extension::from_request_parts(parts, state).await?;
        Ok(Self { boot, services })
    }
}

/// The boot values a write's origin is checked against, extracted with the
/// request headers.
#[derive(Debug, Clone)]
pub struct WriteRequest {
    pub boot: ExtensionBoot,
    pub headers: HeaderMap,
}

impl WriteRequest {
    pub fn require_origin(&self) -> crate::error::AdminResult<()> {
        crate::request::require_write_origin(&self.boot, &self.headers)
    }
}

impl<S: Send + Sync> FromRequestParts<S> for WriteRequest {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(boot) = Extension::from_request_parts(parts, state).await?;
        let Ok(headers) = HeaderMap::from_request_parts(parts, state).await;
        Ok(Self { boot, headers })
    }
}
