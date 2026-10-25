//! `POST /users` — operator-created accounts and their credential bootstrap.
//!
//! Split from `users.rs` at the 300-line ceiling. A user created here gets no
//! credential of its own: Dynapps SSO (Microsoft Entra ID) is the only door,
//! so the row waits to be adopted by the first assertion carrying the matching
//! email and a mapped Entra ID group. The groups and projects picked in the
//! drawer are written as manual grants alongside the row, so the account is
//! scoped before that first sign-in rather than after it.

use std::sync::Arc;
use systemprompt::traits::BackgroundTasks;

use axum::Json;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::repositories;
use systemprompt_web_admin_core::activity;
use systemprompt_web_admin_core::activity::{ActivityEntity, NewActivity};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::{CreateUserRequest, UserContext};

#[derive(Debug, serde::Serialize)]
pub(crate) struct CreatedUserResponse {
    pub user: systemprompt_web_admin_core::types::UserSummary,
    pub invite_note: String,
}

pub(crate) async fn create_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Extension(tasks): Extension<BackgroundTasks>,
    Json(body): Json<CreateUserRequest>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    let user = repositories::users::mutations::create_user(&pool, &body).await?;
    grant_memberships(&pool, &body, &user_ctx.user_id).await?;

    let invite_note = sign_in_note();

    let p = Arc::clone(&pool);
    let uid = user_ctx.user_id.clone();
    let new_user_id = user.user_id.clone();
    let name = user
        .display_name
        .clone()
        .unwrap_or_else(|| user.user_id.as_str().to_owned());
    tasks.spawn("user_created_activity", async move {
        activity::record(
            &p,
            NewActivity::entity_created(&uid, ActivityEntity::User, new_user_id.as_str(), &name),
        )
        .await;
    });
    Ok((
        StatusCode::CREATED,
        Json(CreatedUserResponse { user, invite_note }),
    )
        .into_response())
}

async fn grant_memberships(
    pool: &PgPool,
    body: &CreateUserRequest,
    granted_by: &systemprompt::identifiers::UserId,
) -> AdminResult<()> {
    for group_id in &body.group_ids {
        let res = repositories::groups::members::insert_group_member(
            pool,
            group_id,
            &body.user_id,
            granted_by,
            None,
        )
        .await;
        already_held_is_fine(res)?;
    }
    for project_id in &body.project_ids {
        let res = repositories::projects::members::insert_project_member(
            pool,
            project_id,
            &body.user_id,
            granted_by,
            None,
        )
        .await;
        already_held_is_fine(res)?;
    }
    Ok(())
}

fn already_held_is_fine(res: AdminResult<()>) -> AdminResult<()> {
    match res {
        Err(AdminError::Conflict(_)) => Ok(()),
        other => other,
    }
}

fn sign_in_note() -> String {
    "User created. They sign in with Dynapps SSO (Microsoft Entra ID) — there is no link to \
     send. The groups and projects chosen here are manual grants; any their Entra ID group \
     maps to are added at their first sign-in."
        .to_owned()
}
