//! Ending the browser session a sign-out request carries.
//!
//! Clearing the cookies alone leaves the `user_sessions` row live until its
//! expiry, so a copied `access_token` keeps working after the user signs out.
//! [`revoke_cookie_session`] ends that row, scoped to the token's own user so a
//! lifted session id cannot end somebody else's. It is best effort: a missing,
//! expired or foreign token revokes nothing, and a failed write is logged,
//! because sign-out must still clear the browser either way.

use axum::http::HeaderMap;
use sqlx::PgPool;
use systemprompt_web_admin_core::auth::session::extract_user_from_cookie;

use crate::repositories::users::sessions::revoke_signin_session;

pub async fn revoke_cookie_session(pool: &PgPool, headers: &HeaderMap) -> bool {
    let Ok(session) = extract_user_from_cookie(headers) else {
        return false;
    };
    let Some(session_id) = session.session_id else {
        return false;
    };
    revoke_signin_session(pool, &session.user_id, &session_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "sign-out could not revoke the session row");
            false
        })
}
