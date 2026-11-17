//! What a request says besides its path: the `If-Match` revision, a page
//! size and the body of a thread write.

use axum::http::{HeaderMap, header};
use serde::Deserialize;

use crate::error::ChatsError;
use crate::http::wire::WireModel;
use crate::store::{ModelRef, ThreadPatch};

pub const MAX_TITLE_CHARS: usize = 512;
pub const MAX_MODEL_PART_BYTES: usize = 128;

/// The body of `PUT /threads/{id}`. Every field is optional; unknown fields
/// are ignored.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ThreadInput {
    pub title: Option<String>,
    pub model: Option<WireModel>,
    pub archived: Option<bool>,
}

impl ThreadInput {
    pub fn parse(body: &[u8]) -> Result<Self, ChatsError> {
        if body.iter().all(u8::is_ascii_whitespace) {
            return Ok(Self::default());
        }
        let input: Self = serde_json::from_slice(body)?;
        if input
            .title
            .as_ref()
            .is_some_and(|title| title.chars().count() > MAX_TITLE_CHARS)
        {
            return Err(ChatsError::BadRequest(
                "the title is longer than 512 characters",
            ));
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
        Ok(input)
    }

    pub fn into_patch(self) -> ThreadPatch {
        ThreadPatch {
            title: self.title,
            model: self.model.map(ModelRef::from),
            archived: self.archived,
        }
    }
}

fn valid_model(model: &WireModel) -> bool {
    [&model.provider_id, &model.id]
        .iter()
        .all(|part| !part.trim().is_empty() && part.len() <= MAX_MODEL_PART_BYTES)
}

pub fn if_match(headers: &HeaderMap) -> Result<Option<i64>, ChatsError> {
    let Some(value) = headers.get(header::IF_MATCH) else {
        return Ok(None);
    };
    let text = value
        .to_str()
        .ok()
        .map(str::trim)
        .map(|tag| tag.strip_prefix("W/").unwrap_or(tag))
        .map(|tag| {
            tag.strip_prefix('"')
                .and_then(|inner| inner.strip_suffix('"'))
                .unwrap_or(tag)
        });
    text.and_then(|digits| digits.parse::<i64>().ok())
        .filter(|revision| *revision >= 1)
        .map(Some)
        .ok_or(ChatsError::BadRequest("If-Match must be a revision number"))
}

pub fn page_size(requested: Option<u32>, default: u32, max: u32) -> Result<u32, ChatsError> {
    match requested {
        None => Ok(default),
        Some(0) => Err(ChatsError::BadRequest("limit must be at least 1")),
        Some(limit) => Ok(limit.min(max)),
    }
}
