//! HTTP handlers for plugin listing and installation.

use axum::Json;
use axum::extract::{Extension, Query, State};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use std::sync::Arc;

use crate::repositories;
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::request as shared;
use systemprompt_web_admin_core::types::{UserContext, UserQuery};

use systemprompt_web_admin_core::types::responses::PluginsListResponse;

pub(crate) async fn list_plugins_handler(
    Extension(services): Extension<
        systemprompt_web_admin_core::extension_services::ExtensionServices,
    >,
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Query(_query): Query<UserQuery>,
) -> AdminResult<Response> {
    let services_path = shared::get_services_path()?;
    let plugins = repositories::marketplace::plugins::list_plugins_for_user(
        &pool,
        &services,
        &services_path,
        &user_ctx.user_id,
    )
    .await?;
    Ok(Json(PluginsListResponse { plugins }).into_response())
}
