//! Owner-authenticated source and revision authoring; no implicit activation.

use super::evaluation_experiments::require_write_origin;
use crate::error::AdminResult;
use crate::routes::managed_state::ManagedState;
use crate::types::UserContext;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use std::sync::Arc;

use axum::{Extension, Json};
use serde::Deserialize;
use systemprompt::identifiers::{
    ManagedResourceId, ManagedSourceId, ResourceRevisionId, SourceSnapshotId,
};
use systemprompt::marketplace::managed::{
    NewResource, NewRevision, RevisionFiles, RevisionManifest, SnapshotProvenance, SourceSpec,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateSource {
    name: String,
    specification: SourceSpec,
}

pub(crate) async fn create_source(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<CreateSource>,
) -> AdminResult<(StatusCode, Json<ManagedSourceId>)> {
    require_write_origin(&headers)?;
    let id = state
        .repository
        .register_source(&user.user_id, &input.name, &input.specification)
        .await?;
    Ok((StatusCode::CREATED, Json(id)))
}

pub(crate) async fn source(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ManagedSourceId>,
) -> AdminResult<Json<SourceSpec>> {
    Ok(Json(state.repository.get_source(&user.user_id, &id).await?))
}

pub(crate) async fn capture_snapshot(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ManagedSourceId>,
    headers: HeaderMap,
    Json(input): Json<SnapshotProvenance>,
) -> AdminResult<(StatusCode, Json<SourceSnapshotId>)> {
    require_write_origin(&headers)?;
    Ok((
        StatusCode::CREATED,
        Json(
            state
                .repository
                .capture_snapshot(&user.user_id, &id, &input)
                .await?,
        ),
    ))
}

pub(crate) async fn bind_resource(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<NewResource>,
) -> AdminResult<(StatusCode, Json<ManagedResourceId>)> {
    require_write_origin(&headers)?;
    Ok((
        StatusCode::CREATED,
        Json(
            state
                .repository
                .bind_resource(&user.user_id, &input)
                .await?,
        ),
    ))
}

pub(crate) async fn create_revision(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<NewRevision>,
) -> AdminResult<(StatusCode, Json<ResourceRevisionId>)> {
    require_write_origin(&headers)?;
    Ok((
        StatusCode::CREATED,
        Json(
            state
                .repository
                .create_revision(&user.user_id, &input)
                .await?,
        ),
    ))
}

pub(crate) async fn revision(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ResourceRevisionId>,
) -> AdminResult<Json<RevisionManifest>> {
    Ok(Json(
        state.repository.get_revision(&user.user_id, &id).await?,
    ))
}

pub(crate) async fn files(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ResourceRevisionId>,
) -> AdminResult<([(&'static str, &'static str); 2], Json<RevisionFiles>)> {
    Ok((
        [
            ("cache-control", "no-store"),
            ("x-content-type-options", "nosniff"),
        ],
        Json(
            state
                .repository
                .get_revision_files(&user.user_id, &id)
                .await?,
        ),
    ))
}

pub(crate) async fn create_candidate(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ResourceRevisionId>,
    headers: HeaderMap,
    Json(input): Json<systemprompt::marketplace::managed::TextCandidate>,
) -> AdminResult<(StatusCode, Json<ResourceRevisionId>)> {
    require_write_origin(&headers)?;
    Ok((
        StatusCode::CREATED,
        Json(
            state
                .repository
                .create_text_candidate(&user.user_id, &id, &input)
                .await?,
        ),
    ))
}

pub(crate) async fn comparison(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path((baseline, candidate)): Path<(ResourceRevisionId, ResourceRevisionId)>,
) -> AdminResult<Json<systemprompt::marketplace::managed::RevisionComparison>> {
    Ok(Json(
        state
            .repository
            .compare_revisions(&user.user_id, &baseline, &candidate)
            .await?,
    ))
}
