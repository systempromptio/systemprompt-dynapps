//! Activity constructor for a secrets reload: who asked, which generation is
//! now live, and the names (never values) of the keys that changed.

use systemprompt::identifiers::UserId;

use super::super::enums::{ActivityAction, ActivityCategory, ActivityEntity};
use super::super::types::{ActivityEntityRef, NewActivity};

impl NewActivity {
    #[must_use]
    pub fn secrets_reloaded(user_id: &UserId, generation: u64, changed_keys: &[String]) -> Self {
        let changed = if changed_keys.is_empty() {
            "no keys changed".to_owned()
        } else {
            format!("changed: {}", changed_keys.join(", "))
        };
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::UserManagement,
            action: ActivityAction::Updated,
            entity: Some(ActivityEntityRef {
                kind: ActivityEntity::Sync,
                id: Some("secrets".to_owned()),
                name: Some("Secrets".to_owned()),
            }),
            description: format!("Reloaded secrets to generation {generation} ({changed})"),
            // JSON: JSONB column — the generation and changed key names, so the
            // event page shows what moved without any value
            metadata: serde_json::json!({
                "generation": generation,
                "changed_keys": changed_keys,
            }),
        }
    }
}
