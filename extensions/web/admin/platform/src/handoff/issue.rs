//! `POST /api/v1/drive/handoff {path}`: a one-shot code for the person's
//! `ui` session, answered as the URL that redeems it.
//!
//! The one credential is `Authorization: Bearer <ui session>`, checked as the
//! extension checks it everywhere it takes one (`routes::person_session`):
//! core's signature and expiry for the `ui` audience, the `ui_session` claim
//! and the shape core gives it, and a sign-in time. The session row must also
//! still be live (not revoked, not expired, its person active), checked in the
//! same statement that stores the code. A cookie is never read. `path` must be
//! a `/drive/` path on this site.

use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::header::CACHE_CONTROL;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{Handoff, REDEEM_PATH, drive_path, hash_code};
use crate::repositories::drive_handoffs::{self, NewDriveHandoff};
use crate::routes::person_session::person_session;

const CODE_TTL_SECS: i64 = 60;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct IssueBody {
    path: String,
}

#[derive(Debug, Serialize)]
struct Issued {
    url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HandoffRefused {
    Reauth,
    BadPath,
    Internal,
}

#[derive(Debug, Serialize)]
struct HandoffRefusedBody {
    error: &'static str,
    error_key: &'static str,
}

impl IntoResponse for HandoffRefused {
    fn into_response(self) -> Response {
        let (status, error_key, error) = match self {
            Self::Reauth => (
                StatusCode::UNAUTHORIZED,
                "reauth",
                "a handoff is issued only for the person's live `ui` session",
            ),
            Self::BadPath => (
                StatusCode::BAD_REQUEST,
                "bad_request",
                "the body is `{path}`, a `/drive/` path on this site",
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "the handoff could not be stored",
            ),
        };
        (status, Json(HandoffRefusedBody { error, error_key })).into_response()
    }
}

pub(super) async fn issue(
    State(handoff): State<Handoff>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, HandoffRefused> {
    let parts = &handoff.0;
    let person = person_session(&parts.boot, &headers).ok_or(HandoffRefused::Reauth)?;
    let wanted: IssueBody =
        serde_json::from_slice(&body).map_err(|_unparsed| HandoffRefused::BadPath)?;
    let site = parts.boot.config().api_external_url.trim_end_matches('/');
    let path = drive_path(site, &wanted.path).ok_or(HandoffRefused::BadPath)?;
    let code = URL_SAFE_NO_PAD.encode(rand::rng().random::<[u8; 32]>());
    let stored = drive_handoffs::insert_drive_handoff(
        &parts.pool,
        &NewDriveHandoff {
            code_hash: &hash_code(&code),
            user: &person.user.id,
            session: &person.session,
            path: &path,
            authenticated_at: person.signed_in,
            expires_at: Utc::now() + Duration::seconds(CODE_TTL_SECS),
        },
    )
    .await
    .map_err(|error| {
        tracing::error!(%error, "a session handoff code was not stored");
        HandoffRefused::Internal
    })?;
    if !stored {
        return Err(HandoffRefused::Reauth);
    }
    let url = format!(
        "{site}{REDEEM_PATH}?code={code}&path={}",
        urlencoding::encode(&path)
    );
    Ok(([(CACHE_CONTROL, "no-store")], Json(Issued { url })).into_response())
}
