//! HTTP handlers for user administration.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use systemprompt::identifiers::UserId;

use crate::repositories;
use systemprompt_web_admin_core::activity;
use systemprompt_web_admin_core::activity::{ActivityEntity, NewActivity};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::{UpdateUserRequest, UserContext, UsersQuery};

use systemprompt_web_admin_core::types::responses::{EventsListResponse, UsersListResponse};

pub(crate) async fn list_users_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Query(query): Query<UsersQuery>,
) -> AdminResult<Response> {
    let request = systemprompt_web_admin_core::repositories::scope::ScopeRequest::from_query(
        &user_ctx,
        query.group.as_deref(),
        query.project.as_deref(),
    );
    let scope = systemprompt_web_admin_core::repositories::scope::membership::get_subject_scope(
        &pool, &request,
    )
    .await?;
    let users =
        systemprompt_web_admin_core::repositories::principal::listing::list_users(&pool, &scope)
            .await?;
    Ok(Json(UsersListResponse { users }).into_response())
}

pub(crate) async fn user_detail_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id_raw): Path<String>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id_raw);
    let detail = repositories::users::queries::find_user_detail(&pool, &user_id)
        .await?
        .ok_or_else(|| AdminError::NotFound("User not found".to_owned()))?;
    Ok(Json(detail).into_response())
}

pub(crate) async fn user_usage_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id_raw): Path<String>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id_raw);
    let events = repositories::users::queries::list_user_usage(&pool, &user_id).await?;
    Ok(Json(EventsListResponse { events }).into_response())
}

pub(crate) async fn update_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id_raw): Path<String>,
    Json(body): Json<UpdateUserRequest>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id_raw);
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    let user = repositories::users::mutations::update_user(&pool, &user_id, &body)
        .await?
        .ok_or_else(|| AdminError::NotFound("User not found".to_owned()))?;
    let p = Arc::clone(&pool);
    let uid = user_ctx.user_id.clone();
    let target_user_id = user.user_id.clone();
    let name = user
        .display_name
        .clone()
        .unwrap_or_else(|| user.user_id.as_str().to_owned());
    tokio::spawn(async move {
        activity::record(
            &p,
            NewActivity::entity_updated(&uid, ActivityEntity::User, target_user_id.as_str(), &name),
        )
        .await;
    });
    Ok(Json(user).into_response())
}

pub(crate) async fn delete_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id_raw): Path<String>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id_raw);
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    if !repositories::users::mutations::delete_user(&pool, &user_id).await? {
        return Err(AdminError::NotFound("User not found".to_owned()));
    }
    let p = Arc::clone(&pool);
    let uid = user_ctx.user_id.clone();
    let target = user_id.clone();
    tokio::spawn(async move {
        activity::record(
            &p,
            NewActivity::entity_deleted(
                &uid,
                ActivityEntity::User,
                target.as_str(),
                target.as_str(),
            ),
        )
        .await;
    });
    Ok(StatusCode::NO_CONTENT.into_response())
}
