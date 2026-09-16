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

    #[must_use]
    pub fn sources_refreshed(user_id: &UserId, changed: bool, restarting: bool) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::UserManagement,
            action: ActivityAction::Imported,
            entity: Some(plane_ref("sources", "services sources")),
            description: if changed {
                "Refreshed services sources: composition changed".to_owned()
            } else {
                "Refreshed services sources: unchanged".to_owned()
            },
            // JSON: JSONB column — whether the composition moved and a restart
            // was requested
            metadata: serde_json::json!({ "changed": changed, "restarting": restarting }),
        }
    }
}
