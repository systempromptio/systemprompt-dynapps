//! The extractors of the routes.
//!
//! Each turns a malformed path, query or body into the same
//! `{ "error": { "key", "message" } }` answer as every other failure, instead
//! of axum's plain-text rejection. [`Inference`] reads the AI service core
//! layers onto extension routes; it never fails.

use std::fmt;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{FromRequest, FromRequestParts, Path, Query, Request};
use axum::http::request::Parts;
use serde::de::DeserializeOwned;
use systemprompt::ai::AiService;

use super::error::HttpFault;
use super::state::BrainState;

#[derive(Debug, Clone, Copy)]
pub struct Id(pub i64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token(pub String);

#[derive(Debug, Clone)]
pub struct Params<T>(pub T);

#[derive(Debug, Clone)]
pub struct Payload<T>(pub T);

/// The instance's AI service, which core hands to every extension route.
#[derive(Clone)]
pub struct Inference(pub Option<Arc<AiService>>);

impl fmt::Debug for Inference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Inference").field(&self.0.is_some()).finish()
    }
}

impl FromRequestParts<BrainState> for Inference {
    type Rejection = HttpFault;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &BrainState,
    ) -> impl Future<Output = Result<Self, HttpFault>> + Send {
        let ai = parts
            .extensions
            .get::<Option<Arc<AiService>>>()
            .cloned()
            .flatten();
        std::future::ready(Ok(Self(ai)))
    }
}

impl FromRequestParts<BrainState> for Id {
    type Rejection = HttpFault;

    async fn from_request_parts(parts: &mut Parts, state: &BrainState) -> Result<Self, HttpFault> {
        let Path(id) = Path::<i64>::from_request_parts(parts, state).await?;
        Ok(Self(id))
    }
}

impl FromRequestParts<BrainState> for Token {
    type Rejection = HttpFault;

    async fn from_request_parts(parts: &mut Parts, state: &BrainState) -> Result<Self, HttpFault> {
        let Path(token) = Path::<String>::from_request_parts(parts, state).await?;
        Ok(Self(token))
    }
}

impl<T: DeserializeOwned + Send> FromRequestParts<BrainState> for Params<T> {
    type Rejection = HttpFault;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &BrainState,
    ) -> impl Future<Output = Result<Self, HttpFault>> + Send {
        std::future::ready(
            Query::<T>::try_from_uri(&parts.uri)
                .map(|Query(value)| Self(value))
                .map_err(HttpFault::from),
        )
    }
}

impl<T: DeserializeOwned + Send> FromRequest<BrainState> for Payload<T> {
    type Rejection = HttpFault;

    async fn from_request(request: Request, state: &BrainState) -> Result<Self, HttpFault> {
        let bytes = Bytes::from_request(request, state).await?;
        let body: &[u8] = if bytes.iter().all(u8::is_ascii_whitespace) {
            b"{}"
        } else {
            &bytes
        };
        Ok(Self(serde_json::from_slice(body)?))
    }
}
