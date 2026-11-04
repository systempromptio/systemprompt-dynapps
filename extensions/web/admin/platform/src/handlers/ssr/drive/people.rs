//! `GET /drive/people?ids=a,b`: display names for the user ids a drive page
//! received from the document API, so rows show people, not uuids.
//!
//! Any signed-in person may ask, as they may search the same directory
//! through core's `/api/v1/documents/subjects` when sharing: a name and an
//! email are what that search already shows every signed-in user. The route
//! resolves at most 200 given ids a call and lists nobody unasked, so it
//! names people, it does not enumerate them.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use systemprompt_web_admin_core::error::AdminResult;

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
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<PeopleQuery>,
) -> AdminResult<Json<PeopleResponse>> {
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
