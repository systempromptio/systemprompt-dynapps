//! Activity constructors for `/admin/sync`: each apply, export and source
//! refresh leaves a row saying who, which plane, which mode, and what moved.

use systemprompt::identifiers::UserId;

use super::super::enums::{ActivityAction, ActivityCategory, ActivityEntity};
use super::super::types::{ActivityEntityRef, NewActivity};
use crate::repositories::sync::plane::{ApplyOutcome, SyncMode};

fn plane_ref(plane: &str, label: &str) -> ActivityEntityRef {
    ActivityEntityRef {
        kind: ActivityEntity::Sync,
        id: Some(plane.to_owned()),
        name: Some(label.to_owned()),
    }
}

impl NewActivity {
    #[must_use]
    pub fn sync_applied(
        user_id: &UserId,
        plane: &str,
        label: &str,
        mode: SyncMode,
        outcome: &ApplyOutcome,
    ) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::UserManagement,
            action: ActivityAction::Imported,
            entity: Some(plane_ref(plane, label)),
            description: format!(
                "Synced {label} from code ({}): +{} ~{} -{}",
                mode.human(),
                outcome.inserted,
                outcome.updated,
                outcome.deleted
            ),
            // JSON: JSONB column — the plane and counts, so the event page can
            // show exactly what moved without re-deriving it
            metadata: serde_json::json!({
                "plane": plane,
                "mode": mode.label(),
                "inserted": outcome.inserted,
                "updated": outcome.updated,
                "deleted": outcome.deleted,
                "entities_inserted": outcome.entities_inserted,
                "entities_updated": outcome.entities_updated,
            }),
        }
    }

    #[must_use]
    pub fn sync_exported(user_id: &UserId, plane: &str, label: &str, row_count: usize) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::UserManagement,
            action: ActivityAction::Uploaded,
            entity: Some(plane_ref(plane, label)),
            description: format!("Exported {label} ({row_count} rows) as its declaration file"),
            // JSON: JSONB column — the plane and export size
            metadata: serde_json::json!({ "plane": plane, "row_count": row_count }),
        }
    }

    // Why: a console edit to `ai_gateway_policies` is a write to the
    // gateway_policies plane by another door; it is filed under the plane so
    // the sync page's history and this row read as one story.
    #[must_use]
    pub fn gateway_policy_saved(user_id: &UserId, policy: &str, deleted: bool) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::UserManagement,
            action: if deleted {
                ActivityAction::Deleted
            } else {
                ActivityAction::Updated
            },
            entity: Some(plane_ref("gateway_policies", "Gateway policies")),
            description: if deleted {
                format!("Deleted gateway policy '{policy}' from the console")
            } else {
                format!("Saved gateway policy '{policy}' from the console; live within a minute")
            },
            // JSON: JSONB column — which policy row moved
            metadata: serde_json::json!({ "plane": "gateway_policies", "policy": policy }),
        }
    }

    #[must_use]
    pub fn sources_refreshed(user_id: &UserId, changed: bool, reconciled: bool) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::UserManagement,
            action: ActivityAction::Imported,
            entity: Some(plane_ref("sources", "services sources")),
            description: if changed {
                "Imported services sources: composition changed and reconciled in place".to_owned()
            } else {
                "Refreshed services sources: unchanged".to_owned()
            },
            // JSON: JSONB column — whether the composition moved and whether
            // the new composition was projected into the authz tables
            metadata: serde_json::json!({ "changed": changed, "reconciled": reconciled }),
        }
    }
}
