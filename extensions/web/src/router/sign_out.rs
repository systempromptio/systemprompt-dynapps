//! `DELETE /api/public/auth/session` — sign out.
//!
//! Ends the session row the request's token names, then answers with the
//! content crate's cookie-clearing response. The revocation lives in the admin
//! crate (`admin::sign_out`); this is only where the two halves meet, because
//! the content crate sits below admin and may not call it.

use std::sync::Arc;

use axum::Json;
use axum::http::HeaderMap;
use systemprompt::database::PgPool;

use crate::admin::sign_out::revoke_cookie_session;
use crate::api::OkResponse;
use crate::api::auth::clear_session;

pub(super) async fn sign_out(
    pool: Arc<PgPool>,
    headers: HeaderMap,
) -> (HeaderMap, Json<OkResponse>) {
    revoke_cookie_session(&pool, &headers).await;
    clear_session().await
}
