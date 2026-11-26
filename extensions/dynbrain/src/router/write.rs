//! The writing routes. A write is refused before anything changes when the
//! token's person lacks the access it needs.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use super::error::HttpFault;
use super::extract::{Id, Payload};
use super::state::BrainState;
use crate::auth::Caller;
use crate::brain::args::{
    CreateArgs, GrantTarget, InheritArgs, MoveArgs, SetGrantsArgs, TreeArgs, UpdateArgs,
};
use crate::brain::views::{Created, Grants, InheritChanged, Moved, Trashed, TreeCreated, Updated};
use crate::ids::{ArticleId, SpaceId};

pub async fn create(
    caller: Caller,
    State(state): State<BrainState>,
    Payload(args): Payload<CreateArgs>,
) -> Result<(StatusCode, Json<Created>), HttpFault> {
    let created = state.brain.create(&caller.0, &args).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn create_tree(
    caller: Caller,
    State(state): State<BrainState>,
    Payload(args): Payload<TreeArgs>,
) -> Result<(StatusCode, Json<TreeCreated>), HttpFault> {
    let created = state.brain.create_tree(&caller.0, &args).await?;
    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn update(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Payload(mut args): Payload<UpdateArgs>,
) -> Result<Json<Updated>, HttpFault> {
    args.article_id = ArticleId(id);
    Ok(Json(state.brain.update(&caller.0, &args).await?))
}

pub async fn move_article(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Payload(mut args): Payload<MoveArgs>,
) -> Result<Json<Moved>, HttpFault> {
    args.article_id = ArticleId(id);
    Ok(Json(state.brain.move_article(&caller.0, &args).await?))
}

pub async fn trash(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
) -> Result<Json<Trashed>, HttpFault> {
    Ok(Json(state.brain.trash(&caller.0, ArticleId(id)).await?))
}

pub async fn set_space_grants(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Payload(args): Payload<SetGrantsArgs>,
) -> Result<Json<Grants>, HttpFault> {
    let target = GrantTarget::Space(SpaceId(id));
    Ok(Json(
        state
            .brain
            .set_grants(&caller.0, target, &args.grants)
            .await?,
    ))
}

pub async fn set_article_grants(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Payload(args): Payload<SetGrantsArgs>,
) -> Result<Json<Grants>, HttpFault> {
    let target = GrantTarget::Article(ArticleId(id));
    Ok(Json(
        state
            .brain
            .set_grants(&caller.0, target, &args.grants)
            .await?,
    ))
}

pub async fn set_inherit(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Payload(args): Payload<InheritArgs>,
) -> Result<Json<InheritChanged>, HttpFault> {
    let changed = state
        .brain
        .set_inherit(&caller.0, ArticleId(id), args)
        .await?;
    Ok(Json(changed))
}
