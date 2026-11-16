//! Teams presence: the signed-in user's, another user's, and setting the
//! user's own. Graph takes a presence set by an app as a session of that
//! app, so `sessionId` is the Entra app's client id the endpoint carries
//! (the Belgian server sent its `entra_client_id` the same way).

use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use super::graph_tool::graph_tools;
use super::reply::GraphItem;
use super::teams::select;
use super::teams_write::noted;
use crate::graph::config::APP_ID_SECRET;
use crate::graph::sanitize::validate_id;
use crate::graph::{GraphClient, GraphError, GraphId, GraphQuery};

pub const PRESENCE_FIELDS: &str = "id,availability,activity,statusMessage";

const GET_MY_PRESENCE: &str = "Get the signed-in user's presence status.";
const GET_USER_PRESENCE: &str = "Get another user's presence status.";
const SET_MY_PRESENCE: &str = "Set the signed-in user's presence status for a while \
    (default one hour).";

graph_tools! {
    "d365_get_my_presence" => get_my_presence, MyPresenceInput, GraphItem, GET_MY_PRESENCE;
    "d365_get_user_presence" => get_user_presence, UserPresenceInput, GraphItem,
        GET_USER_PRESENCE;
    "d365_set_my_presence" => set_my_presence, SetPresenceInput, GraphItem, SET_MY_PRESENCE;
}

fn default_expiration() -> String {
    "PT1H".to_owned()
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct MyPresenceInput {}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UserPresenceInput {
    #[schemars(description = "The Azure AD user ID.")]
    pub user_id: GraphId,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SetPresenceInput {
    #[schemars(
        description = "Availability: Available, Busy, DoNotDisturb, BeRightBack, Away or Offline."
    )]
    pub availability: String,
    #[schemars(
        description = "Activity: Available, InACall, InAConferenceCall, Away, Presenting, etc."
    )]
    pub activity: String,
    #[schemars(
        description = "ISO 8601 duration the status stays active (default \"PT1H\" = 1 hour)."
    )]
    #[serde(default = "default_expiration")]
    pub expiration_duration: String,
}

pub async fn get_my_presence(
    graph: &GraphClient,
    _input: MyPresenceInput,
) -> Result<GraphItem, GraphError> {
    let data = graph
        .get_json("/me/presence", &select(PRESENCE_FIELDS))
        .await?;
    Ok(GraphItem::new(data))
}

pub async fn get_user_presence(
    graph: &GraphClient,
    input: UserPresenceInput,
) -> Result<GraphItem, GraphError> {
    let user = validate_id(&input.user_id, "user_id")?;
    let data = graph
        .get_json(&format!("/users/{user}/presence"), &GraphQuery::new())
        .await?;
    Ok(GraphItem::new(data))
}

pub async fn set_my_presence(
    graph: &GraphClient,
    input: SetPresenceInput,
) -> Result<GraphItem, GraphError> {
    let session = graph.endpoint().app_id().ok_or_else(|| {
        GraphError::Config(format!(
            "setting presence needs the Entra app's client id (secret `{APP_ID_SECRET}`)"
        ))
    })?;
    let message = format!("Presence set to {}/{}", input.availability, input.activity);
    let presence = json!({
        "sessionId": session,
        "availability": input.availability,
        "activity": input.activity,
        "expirationDuration": input.expiration_duration,
    });
    graph
        .post_json("/me/presence/setPresence", presence)
        .await?;
    Ok(noted(message))
}
