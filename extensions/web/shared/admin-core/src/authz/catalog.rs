//! Current subject and parent policies reused across one catalog request.

use crate::error::{AdminError, AdminResult};
use crate::extension_services::ExtensionServices;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::governance::authz::{
    AccessControlRepository, BulkKeepQuery, ChainSources, EntityKind, ParentChainIndex,
    SubjectAttributes, allowed_ids,
};
use systemprompt::identifiers::UserId;

pub struct CatalogAccess {
    repo: AccessControlRepository,
    user: UserId,
    roles: Vec<String>,
    attributes: SubjectAttributes,
    chains: ParentChainIndex,
    services: ExtensionServices,
}

impl std::fmt::Debug for CatalogAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CatalogAccess")
            .field("user", &self.user)
            .field("roles", &self.roles)
            .finish_non_exhaustive()
    }
}

impl CatalogAccess {
    pub async fn load(
        pool: &PgPool,
        services: &ExtensionServices,
        user: &UserId,
    ) -> AdminResult<Self> {
        let identity = crate::repositories::principal::identity::find_identity_envelope(pool, user)
            .await?
            .ok_or_else(|| AdminError::Unauthorized("Account unavailable".into()))?;
        if identity.status != "active" {
            return Err(AdminError::Forbidden("Active account required".into()));
        }
        let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));
        let attributes = (services).subject_attributes(user).await?;
        let chains = ParentChainIndex::load(
            &repo,
            Arc::new(ChainSources::from_services(&services.config())),
        )
        .await
        .map_err(AdminError::internal)?;
        Ok(Self {
            repo,
            user: user.clone(),
            roles: identity.roles,
            attributes,
            chains,
            services: services.clone(),
        })
    }

    pub async fn allowed(
        &self,
        kind: EntityKind,
        ids: &[String],
    ) -> AdminResult<std::collections::HashSet<String>> {
        allowed_ids(
            &self.repo,
            BulkKeepQuery {
                user_id: &self.user,
                roles: &self.roles,
                kind,
                ids,
                chains: &self.chains,
                attributes: &self.attributes,
                dimensions: self.services.dimensions(),
            },
        )
        .await
        .map_err(AdminError::internal)
    }
}
