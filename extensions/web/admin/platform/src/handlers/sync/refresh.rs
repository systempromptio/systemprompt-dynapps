//! `POST /sync/sources/refresh` — re-resolve the pinned sources through
//! core's own refresh pipeline, in-process.
//!
//! Core owns the fetch-verify-compose pipeline and the in-place import that
//! follows it (authz reconcile + inventory refresh); an extension has
//! neither. Core hands every extension router a `ServicesRefresh` handle to
//! that pipeline, so this handler runs it directly for a caller it has
//! authorised by its own rule — an administrator, or a marketplace
//! participant syncing their kits — with no `restart` (a marketplace-only kit
//! is served the moment core recomposes) and no admin token minted. Core's
//! typed response is relayed whole: `refused` names each source held back on
//! its previous bundle and `provenance.error` says why a whole composition
//! fell back. Both come with `changed: false` as often as not, so neither an
//! import nor its activity row reads as unchanged while one is set.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use systemprompt::api::routes::admin::services::ServicesRefresh;

use systemprompt_web_admin_core::activity;
use systemprompt_web_admin_core::activity::{NewActivity, SourcesRefresh};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;

pub(crate) async fn refresh_sources_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Extension(refresh): Extension<ServicesRefresh>,
) -> AdminResult<Response> {
    let result = Box::pin(refresh.run(&user_ctx.user_id))
        .await
        .map_err(|e| {
            let err = e.into_inner();
            match err.code.status_code() {
                StatusCode::CONFLICT => AdminError::Conflict(err.message),
                _ => AdminError::Upstream(format!("services refresh failed: {}", err.message)),
            }
        })?;
    activity::record(
        &pool,
        NewActivity::sources_refreshed(
            &user_ctx.user_id,
            SourcesRefresh {
                changed: result.changed,
                reconciled: result.reconciled,
                fallback: result.provenance.error.as_deref(),
                refused: &result.refused,
            },
        ),
    )
    .await;
    Ok(Json(result).into_response())
}
