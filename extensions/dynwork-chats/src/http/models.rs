//! `GET /models`.
//!
//! The models this instance can chat with, `{ models: [...] }`, each as
//! `{ id, name, providerID, provider: { id, name }, reasoning, tools }`: the
//! phone reads `providerID`, the desktop's Chats view reads `provider`. `id`
//! and `providerID` are what a thread's `model` and a prompt's `model` take.
//! The list comes from the instance's catalog (see [`crate::catalog`]); an
//! instance with no AI service answers `503 ai_unavailable`.

use std::sync::Arc;

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::Serialize;
use systemprompt::ai::AiService;

use crate::auth::Caller;
use crate::catalog::ModelInfo;
use crate::error::ChatsError;
use crate::http::response::no_store;
use crate::state::ChatsState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WireProvider {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WireModelInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "providerID")]
    pub provider_id: String,
    pub provider: WireProvider,
    pub reasoning: bool,
    pub tools: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WireModels {
    pub models: Vec<WireModelInfo>,
}

impl From<&ModelInfo> for WireModelInfo {
    fn from(model: &ModelInfo) -> Self {
        Self {
            id: model.id.clone(),
            name: model.name.clone(),
            provider_id: model.provider_id.clone(),
            provider: WireProvider {
                id: model.provider_id.clone(),
                name: model.provider_name.clone(),
            },
            reasoning: model.reasoning,
            tools: model.tools,
        }
    }
}

pub async fn list(
    _caller: Caller,
    State(state): State<ChatsState>,
    ai: Option<Extension<Option<Arc<AiService>>>>,
) -> Result<Response, ChatsError> {
    let ai = ai.and_then(|Extension(ai)| ai);
    let models = state
        .models
        .list(ai.as_deref())
        .ok_or(ChatsError::AiUnavailable)?;
    let body = WireModels {
        models: models.iter().map(WireModelInfo::from).collect(),
    };
    Ok(no_store(Json(body).into_response()))
}
