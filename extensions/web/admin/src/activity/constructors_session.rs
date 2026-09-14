//! Activity constructors for sign-in and agent-response events.

use systemprompt::identifiers::{SessionId, UserId};

use super::constructors::truncate;
use super::enums::{ActivityAction, ActivityCategory};
use super::types::NewActivity;

// JSON: JSONB column — the empty `user_activity.metadata` object for events
// that carry no fields of their own
fn empty_meta() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

impl NewActivity {
    #[must_use]
    pub fn login(user_id: &UserId, display_name: &str) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::Login,
            action: ActivityAction::LoggedIn,
            entity: None,
            description: format!("{display_name} logged in"),
            metadata: empty_meta(),
        }
    }

    #[must_use]
    pub fn agent_response(
        user_id: &UserId,
        session_id: &SessionId,
        message_preview: Option<&str>,
    ) -> Self {
        let description = message_preview.map_or_else(
            || "Claude finished responding".to_owned(),
            |msg| format!("Claude responded: \"{}\"", truncate(msg, 80)),
        );
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::AgentResponse,
            action: ActivityAction::Submitted,
            entity: None,
            description,
            // JSON: user_activity.metadata JSONB stores the known session identifier.
            metadata: serde_json::Value::Object(serde_json::Map::from_iter([(
                "session_id".to_owned(),
                serde_json::Value::String(session_id.as_str().to_owned()),
            )])),
        }
    }
}
