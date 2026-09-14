//! Activity constructors for session-scoped notifications.

use systemprompt::identifiers::{SessionId, UserId};

use super::super::constructors::truncate;
use super::super::enums::{ActivityAction, ActivityCategory};
use super::super::types::NewActivity;

impl NewActivity {
    #[must_use]
    pub fn notification(
        user_id: &UserId,
        session_id: &SessionId,
        ntype: Option<&str>,
        message: Option<&str>,
    ) -> Self {
        let description = match (ntype, message) {
            (Some("permission_prompt"), Some(msg)) => {
                format!("Permission prompt: {}", truncate(msg, 80))
            },
            (Some(t), Some(msg)) => format!("{t}: {}", truncate(msg, 80)),
            (Some(t), None) => t.to_owned(),
            (None, Some(msg)) => truncate(msg, 80),
            (None, None) => "Notification received".to_owned(),
        };
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::Notification,
            action: ActivityAction::Submitted,
            entity: None,
            description,
            metadata: serde_json::Value::Object(serde_json::Map::from_iter([(
                "session_id".to_owned(),
                serde_json::Value::String(session_id.as_str().to_owned()),
            )])),
        }
    }
}
