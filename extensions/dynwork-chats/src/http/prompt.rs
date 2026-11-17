//! `POST /threads/{id}/prompt` and `POST /threads/{id}/interrupt`.
//!
//! A prompt is `{ messageID, parts: [{ type: "text", text }], model? }` (a
//! malformed `messageID` is a malformed body, `400 bad_request`). It
//! answers `202 { assistantMessageID }` once the reply has started; the reply
//! itself runs on the server and is read from `/log`. Without a model service
//! on the instance it is `503 ai_unavailable`; while another reply runs it is
//! `409 busy`. `interrupt` stops the running reply, waits for its last events
//! and answers `204`, also when nothing was running.

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::rejection::{BytesRejection, PathRejection};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use systemprompt::ai::AiService;

use crate::auth::Caller;
use crate::error::ChatsError;
use crate::http::params::valid_model;
use crate::http::response::no_store;
use crate::http::wire::WireModel;
use crate::ids::{MessageId, ThreadId};
use crate::reply::model::GatewayModel;
use crate::reply::start::{self, Prompt};
use crate::state::ChatsState;
use crate::store::ModelRef;

/// One part of a prompt; stage 1 takes text parts only.
#[derive(Debug, Clone, Deserialize)]
pub struct PartInput {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: Option<String>,
}

/// The body of a prompt.
#[derive(Debug, Clone, Deserialize)]
pub struct PromptInput {
    #[serde(rename = "messageID")]
    pub message_id: MessageId,
    pub parts: Vec<PartInput>,
    pub model: Option<WireModel>,
}

impl PromptInput {
    pub fn parse(body: &[u8]) -> Result<Prompt, ChatsError> {
        let input: Self = serde_json::from_slice(body)?;
        if input.parts.iter().any(|part| part.kind != "text") {
            return Err(ChatsError::BadRequest("a prompt takes text parts only"));
        }
        let text = input
            .parts
            .iter()
            .filter_map(|part| part.text.as_deref())
            .collect::<Vec<_>>()
            .join("\n");
        if text.trim().is_empty() {
            return Err(ChatsError::BadRequest("a prompt needs some text"));
        }
        if input
            .model
            .as_ref()
            .is_some_and(|model| !valid_model(model))
        {
            return Err(ChatsError::BadRequest(
                "a model needs a providerID and an id of 1 to 128 bytes",
            ));
        }
        Ok(Prompt {
            message: input.message_id,
            text,
            model: input.model.map(ModelRef::from),
        })
    }
}

/// The answer to an accepted prompt.
#[derive(Debug, Clone, Serialize)]
pub struct Accepted {
    #[serde(rename = "assistantMessageID")]
    pub assistant_message_id: String,
}

pub async fn prompt(
    caller: Caller,
    State(state): State<ChatsState>,
    ai: Option<Extension<Option<Arc<AiService>>>>,
    path: Result<Path<String>, PathRejection>,
    body: Result<Bytes, BytesRejection>,
) -> Result<Response, ChatsError> {
    let Path(id) = path?;
    let thread: ThreadId = id.parse()?;
    let prompt = PromptInput::parse(&body?)?;
    let gateway = ai.and_then(|Extension(ai)| ai).map(GatewayModel::shared);
    let engine = state
        .models
        .pick(gateway)
        .ok_or(ChatsError::AiUnavailable)?;
    let reply = start::start(&state, &caller.user, &thread, prompt, engine).await?;
    let body = Accepted {
        assistant_message_id: reply.as_str().to_owned(),
    };
    Ok(no_store((StatusCode::ACCEPTED, Json(body)).into_response()))
}

pub async fn interrupt(
    caller: Caller,
    State(state): State<ChatsState>,
    path: Result<Path<String>, PathRejection>,
) -> Result<Response, ChatsError> {
    let Path(id) = path?;
    let thread: ThreadId = id.parse()?;
    start::interrupt(&state, &caller.user, &thread).await?;
    Ok(no_store(StatusCode::NO_CONTENT.into_response()))
}
