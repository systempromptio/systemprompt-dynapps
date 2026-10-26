//! The planes core owns: `access_control`, `governance` and
//! `gateway_policies`.
//!
//! Core enforces all three from the database and is their only writer. The
//! console reads their state from core (`config_plane_state`, the reconcile
//! diff, the plane's own export) and writes through core alone: an
//! administrator's apply is [`ConfigApplyService`]'s entity merge, and a
//! marketplace participant's apply is core's scoped apply narrowed to their
//! marketplaces. Groups and gateway routes stay install planes
//! ([`super::plane::SyncPlane`]).
//!
//! The router root builds one [`CorePlanes`] from the extension context and
//! layers it as an `Extension`; a context that cannot provide core's plane set
//! builds none, and every handler that needs it refuses.

use std::sync::Arc;
use std::time::Duration;

use systemprompt::extension::prelude::{
    ApplyMode, ChangeActor, ChangeRequest, ConfigPlaneSet, EntityScope, ExtensionContext,
};
use systemprompt::identifiers::UserId;
use systemprompt::scheduler::repository::ConfigReconcileRepository;
use systemprompt::scheduler::services::{
    AppliedPlane, ApplyOptions, ConfigApplyService, ConfigReconcileService, PlaneDiff, PlaneMerge,
};
use systemprompt::traits::Dependencies;

use super::plane_ids;
use crate::error::{AdminError, AdminResult};

pub const CORE_PLANE_IDS: [&str; 3] = [
    plane_ids::ACCESS_CONTROL,
    plane_ids::GATEWAY_POLICIES,
    plane_ids::GOVERNANCE,
];

const APPLY_WAIT: Duration = Duration::from_secs(30);

/// What the console shows for a core plane besides core's own state.
#[derive(Debug, Clone, Copy)]
pub struct CorePlaneMeta {
    pub id: &'static str,
    pub label: &'static str,
    pub source_file: &'static str,
    pub owner_url: &'static str,
    pub sync_url: &'static str,
}

#[must_use]
pub fn core_plane_meta(id: &str) -> Option<CorePlaneMeta> {
    match id {
        plane_ids::ACCESS_CONTROL => Some(CorePlaneMeta {
            id: plane_ids::ACCESS_CONTROL,
            label: "Access control",
            source_file: "access-control/rules.yaml",
            owner_url: "/admin/access-control",
            sync_url: "/admin/sync?tab=access",
        }),
        plane_ids::GATEWAY_POLICIES => Some(CorePlaneMeta {
            id: plane_ids::GATEWAY_POLICIES,
            label: "Gateway policies",
            source_file: "gateway/policies.yaml",
            owner_url: "/admin/gateway/policies",
            sync_url: "/admin/gateway/policies?tab=sync",
        }),
        plane_ids::GOVERNANCE => Some(CorePlaneMeta {
            id: plane_ids::GOVERNANCE,
            label: "Governance chain",
            source_file: "governance/config.yaml",
            owner_url: "/admin/governance",
            sync_url: "/admin/governance?tab=sync",
        }),
        _ => None,
    }
}

#[must_use]
pub fn is_core_plane(id: &str) -> bool {
    core_plane_meta(id).is_some()
}

/// Core's apply and reconcile services over the context's plane set.
#[derive(Clone)]
pub struct CorePlanes {
    apply: Arc<ConfigApplyService>,
}

impl std::fmt::Debug for CorePlanes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CorePlanes").finish_non_exhaustive()
    }
}

impl CorePlanes {
    #[must_use]
    pub fn over(planes: ConfigPlaneSet, repository: ConfigReconcileRepository) -> Self {
        let reconcile = ConfigReconcileService::new(planes, repository, Dependencies::new());
        Self {
            apply: Arc::new(ConfigApplyService::new(reconcile)),
        }
    }

    pub fn from_context(
        ctx: &dyn ExtensionContext,
        repository: ConfigReconcileRepository,
    ) -> Option<Self> {
        match ctx.config_planes() {
            Ok(planes) => Some(Self::over(planes, repository)),
            Err(error) => {
                tracing::error!(%error, "core's config planes are unavailable; /admin/sync cannot read or apply them");
                None
            },
        }
    }

    pub async fn diff(&self, id: &str) -> AdminResult<PlaneDiff> {
        require_core(id)?;
        self.apply
            .reconcile()
            .diff(id)
            .await
            .map_err(AdminError::internal)
    }

    pub async fn export(&self, id: &str) -> AdminResult<Option<String>> {
        require_core(id)?;
        self.apply
            .reconcile()
            .export(id)
            .await
            .map_err(AdminError::internal)
    }

    pub async fn apply(&self, id: &str, actor: &UserId, reason: &str) -> AdminResult<PlaneMerge> {
        require_core(id)?;
        let planes = [id.to_owned()];
        let report = self
            .apply
            .apply(&ApplyOptions {
                planes: &planes,
                actor: ChangeActor::User(actor.clone()),
                reason,
                dry_run: false,
                wait: APPLY_WAIT,
            })
            .await
            .map_err(AdminError::internal)?;
        report
            .planes
            .into_iter()
            .find(|p| p.plane == id)
            .ok_or_else(|| AdminError::internal(format!("core applied no '{id}' plane")))
    }

    pub async fn apply_entities(
        &self,
        id: &str,
        entities: &[String],
        actor: &UserId,
        reason: &str,
    ) -> AdminResult<AppliedPlane> {
        require_core(id)?;
        self.apply
            .reconcile()
            .apply(
                id,
                ApplyMode::Overwrite,
                &ChangeRequest {
                    actor: ChangeActor::User(actor.clone()),
                    reason,
                },
                EntityScope::Entities(entities),
            )
            .await
            .map_err(AdminError::internal)
    }
}

fn require_core(id: &str) -> AdminResult<()> {
    if is_core_plane(id) {
        Ok(())
    } else {
        Err(AdminError::NotFound(format!("no core plane '{id}'")))
    }
}
