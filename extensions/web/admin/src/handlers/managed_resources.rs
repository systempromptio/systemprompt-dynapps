//! Owner-authenticated source and revision authoring; no implicit activation.

use super::evaluation_experiments::require_write_origin;
use crate::error::AdminResult;
use crate::routes::managed_state::ManagedState;
use crate::types::UserContext;
use axum::extract::{Path, Query};
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

pub(crate) async fn bundle(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ResourceRevisionId>,
) -> AdminResult<axum::response::Response> {
    use axum::response::IntoResponse;
    use systemprompt::marketplace::managed::AssetDigest;
    let bundle = state
        .repository
        .get_revision_bundle(&user.user_id, &id)
        .await?;
    let bytes = bundle.canonical_bytes()?;
    let digest = AssetDigest::of(&bytes);
    Ok((
        [
            ("content-type", "application/json".to_owned()),
            (
                "content-disposition",
                "attachment; filename=revision-bundle.json".to_owned(),
            ),
            ("cache-control", "no-store".to_owned()),
            ("x-content-type-options", "nosniff".to_owned()),
            ("x-content-sha256", digest.as_str().to_owned()),
        ],
        bytes,
    )
        .into_response())
}

pub(crate) async fn publish(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<systemprompt::marketplace::managed::PublicationRequest>,
) -> AdminResult<(
    StatusCode,
    Json<systemprompt::marketplace::managed::PublicationDecision>,
)> {
    require_write_origin(&headers)?;
    let decision = state
        .repository
        .review_and_publish(&user.user_id, &user.user_id, &input)
        .await?;
    Ok((StatusCode::CREATED, Json(decision)))
}

pub(crate) async fn resolution(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path((kind, key)): Path<(systemprompt::marketplace::managed::ResourceKind, String)>,
) -> AdminResult<Json<systemprompt::marketplace::managed::ManagedResolution>> {
    Ok(Json(
        state
            .repository
            .resolve_managed(&user.user_id, kind, &key)
            .await?,
    ))
}

#[derive(Debug, Deserialize)]
pub(crate) struct PublicationBundleQuery {
    digest: String,
}

pub(crate) async fn publication_bundle(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path((resource, generation)): Path<(ManagedResourceId, i64)>,
    Query(query): Query<PublicationBundleQuery>,
) -> AdminResult<axum::response::Response> {
    use axum::response::IntoResponse;
    use systemprompt::marketplace::managed::AssetDigest;
    let digest = AssetDigest::try_from(query.digest)?;
    let bundle = state
        .repository
        .get_publication_bundle(&user.user_id, &resource, generation, &digest)
        .await?;
    let bytes = bundle.canonical_bytes()?;
    Ok((
        [
            ("content-type", "application/json".to_owned()),
            (
                "content-disposition",
                format!("attachment; filename=managed-bundle-{generation}.json"),
            ),
            ("cache-control", "private, no-store".to_owned()),
            ("x-content-type-options", "nosniff".to_owned()),
            ("x-content-sha256", digest.as_str().to_owned()),
            ("x-publication-generation", generation.to_string()),
        ],
        bytes,
    )
        .into_response())
}
