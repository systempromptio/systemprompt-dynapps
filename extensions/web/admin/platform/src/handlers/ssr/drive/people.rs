//! `GET /admin/drive/people?ids=a,b`: display names for the user ids a drive
//! page received from the document API, so rows show people, not uuids.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Query, State};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;

use crate::repositories::drive_people::{DrivePerson, list_drive_people};

const MAX_IDS: usize = 200;

#[derive(Debug, Deserialize)]
pub(crate) struct PeopleQuery {
    #[serde(default)]
    ids: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct PeopleResponse {
    people: Vec<DrivePerson>,
}

pub(crate) async fn drive_people(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<PeopleQuery>,
) -> AdminResult<Json<PeopleResponse>> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()));
    }
    let ids: Vec<String> = query
        .ids
        .split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .take(MAX_IDS)
        .map(str::to_owned)
        .collect();
    let people = if ids.is_empty() {
        Vec::new()
    } else {
        list_drive_people(&pool, &ids).await?
    };
    Ok(Json(PeopleResponse { people }))
}
