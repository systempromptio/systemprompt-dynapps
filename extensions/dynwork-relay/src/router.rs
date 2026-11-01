//! The relay's routes, relative to [`BASE_PATH`].
//!
//! Core mounts an extension router at a base path that is `/` or under
//! `/api/`. The relay sits under `/api/` on purpose: core's session
//! middleware treats every other path as a tracked site page and would mint
//! an anonymous session (and record the URI, pairing token included) for
//! each socket and claim.

use axum::Router;
use axum::routing::{delete, get, post};

use crate::state::RelayState;
use crate::{http, ws};

pub const BASE_PATH: &str = "/api/dynwork/relay/v1";

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
