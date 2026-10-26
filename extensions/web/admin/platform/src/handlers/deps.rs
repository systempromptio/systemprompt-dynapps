//! Request parts several handlers extract together: the signed-in operator
//! with the boot values its writes are checked against, and the core sync
//! planes with the boot values the install planes are built over.

use axum::Extension;
use axum::extract::FromRequestParts;
use axum::extract::rejection::ExtensionRejection;
use axum::http::request::Parts;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;
use systemprompt_web_admin_core::repositories::sync::core_planes::CorePlanes;
use systemprompt_web_admin_core::types::UserContext;

#[derive(Debug, Clone)]
pub(crate) struct Operator {
    pub user_ctx: UserContext,
    pub boot: ExtensionBoot,
}

impl<S: Send + Sync> FromRequestParts<S> for Operator {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(user_ctx) = Extension::from_request_parts(parts, state).await?;
        let Extension(boot) = Extension::from_request_parts(parts, state).await?;
        Ok(Self { user_ctx, boot })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PlaneDeps {
    pub core_planes: CorePlanes,
    pub boot: ExtensionBoot,
}

impl<S: Send + Sync> FromRequestParts<S> for PlaneDeps {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(core_planes) = Extension::from_request_parts(parts, state).await?;
        let Extension(boot) = Extension::from_request_parts(parts, state).await?;
        Ok(Self { core_planes, boot })
    }
}
