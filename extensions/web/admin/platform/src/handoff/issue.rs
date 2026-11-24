//! `POST /api/v1/drive/handoff {path}`: a one-shot code for the person's
//! `ui` session, answered as the URL that redeems it.
//!
//! The one credential is `Authorization: Bearer <ui session>`, checked as the
//! extension checks it everywhere it takes one (`artifacts::manifest_read`):
//! core's signature and expiry for the `ui` audience, the `ui_session` claim
//! and the shape core gives it. The session row must also still be live (not
//! revoked, not expired, its person active), checked in the same statement
//! that stores the code. A cookie is never read. `path` must be a `/drive/`
//! path on this site.

use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::header::{AUTHORIZATION, CACHE_CONTROL};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt::models::auth::{JwtAudience, JwtClaims};
use systemprompt::oauth::validate_jwt_token;

use super::{Handoff, REDEEM_PATH, drive_path, hash_code};
use crate::repositories::drive_handoffs::{self, NewDriveHandoff};

const BEARER: &str = "Bearer ";
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

struct UiSession {
    user: UserId,
    session: SessionId,
    signed_in: DateTime<Utc>,
}

pub(super) async fn issue(
    State(handoff): State<Handoff>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, HandoffRefused> {
    let parts = &handoff.0;
    let person = ui_session(&parts.boot, &headers)?;
    let wanted: IssueBody =
        serde_json::from_slice(&body).map_err(|_unparsed| HandoffRefused::BadPath)?;
    let site = parts.boot.config().api_external_url.trim_end_matches('/');
    let path = drive_path(site, &wanted.path).ok_or(HandoffRefused::BadPath)?;
    let code = URL_SAFE_NO_PAD.encode(rand::rng().random::<[u8; 32]>());
    let stored = drive_handoffs::insert_drive_handoff(
        &parts.pool,
        &NewDriveHandoff {
            code_hash: &hash_code(&code),
            user: &person.user,
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

fn ui_session(
    boot: &systemprompt_web_admin_core::extension_boot::ExtensionBoot,
    headers: &HeaderMap,
) -> Result<UiSession, HandoffRefused> {
    let bearer = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix(BEARER))
        .ok_or(HandoffRefused::Reauth)?;
    let claims = validate_jwt_token(
        boot.token_authority().as_ref(),
        bearer.trim(),
        &[JwtAudience::Ui],
    )
    .map_err(|error| {
        tracing::debug!(%error, "a handoff bearer did not validate");
        HandoffRefused::Reauth
    })?;
    person_of(claims).ok_or(HandoffRefused::Reauth)
}

fn person_of(claims: JwtClaims) -> Option<UiSession> {
    if !claims.custom.ui_session || !claims.ui_session_well_formed() {
        return None;
    }
    let signed_in = DateTime::<Utc>::from_timestamp(claims.custom.auth_time, 0)
        .filter(|_| claims.custom.auth_time > 0)?
        .min(Utc::now());
    Some(UiSession {
        user: UserId::new(claims.registered.sub),
        session: claims.custom.session_id?,
        signed_in,
    })
}
