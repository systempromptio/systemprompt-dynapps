//! The extension's routes: the relay's, relative to [`BASE_PATH`], and the
//! mobile sign-in's, relative to [`MOBILE_BASE_PATH`], both mounted under
//! [`EXTENSION_BASE_PATH`] by [`extension_router`].
//!
//! Core mounts an extension router at a base path that is `/` or under
//! `/api/`. The relay sits under `/api/` on purpose: core's session
//! middleware treats every other path as a tracked site page and would mint
//! an anonymous session (and record the URI, pairing token included) for
//! each socket and claim.

use axum::Router;
use axum::routing::{delete, get, post};

pub use crate::mobile_auth::MOBILE_BASE_PATH;
use crate::mobile_auth::mobile_router;
use crate::state::RelayState;
use crate::{http, ws};

pub const EXTENSION_BASE_PATH: &str = "/api/dynwork";
pub const BASE_PATH: &str = "/api/dynwork/relay/v1";

pub fn extension_router(state: RelayState) -> Router {
    Router::new()
        .nest("/relay/v1", relay_router(state.clone()))
        .nest("/mobile/v1", mobile_router(state))
}

pub fn relay_router(state: RelayState) -> Router {
    Router::new()
        .route("/desktop", get(ws::desktop_socket))
        .route("/device", get(ws::device_socket))
        .route("/pairings", post(http::pairings::create))
        .route("/pairings/claim", post(http::pairings::claim))
        .route("/devices", get(http::devices::list))
        .route("/devices/{id}", delete(http::devices::revoke))
        .with_state(state)
}
