//! The writing routes. A write is refused before anything changes when the
//! token's person lacks the access it needs.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
// JSON: an operation's reply, as the Brain answered it.
use serde_json::Value;

use super::error::HttpFault;
use super::extract::{Id, Payload};
use super::relay::relay;
use super::state::BrainState;
use crate::auth::Caller;
use crate::brain::args::{
    CreateArgs, GrantTarget, InheritArgs, MoveArgs, SetGrantsArgs, TreeArgs, UpdateArgs,
};
use crate::brain::operation::Operation;
use crate::brain::views::{Grants, InheritChanged};
use crate::ids::{ArticleId, SpaceId};

pub async fn create(
    caller: Caller,
    State(state): State<BrainState>,
    Payload(args): Payload<CreateArgs>,
) -> Result<(StatusCode, Json<Value>), HttpFault> {
    let created = relay(&state, &caller, Operation::Create, &args).await?;
    Ok((StatusCode::CREATED, created))
}

pub async fn create_tree(
    caller: Caller,
    State(state): State<BrainState>,
    Payload(args): Payload<TreeArgs>,
) -> Result<(StatusCode, Json<Value>), HttpFault> {
    let created = relay(&state, &caller, Operation::CreateTree, &args).await?;
    Ok((StatusCode::CREATED, created))
}

pub async fn update(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Payload(mut args): Payload<UpdateArgs>,
) -> Result<Json<Value>, HttpFault> {
    args.article_id = ArticleId(id);
    relay(&state, &caller, Operation::Update, &args).await
}

pub async fn move_article(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Payload(mut args): Payload<MoveArgs>,
) -> Result<Json<Value>, HttpFault> {
    args.article_id = ArticleId(id);
    relay(&state, &caller, Operation::Move, &args).await
}

pub async fn trash(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
) -> Result<Json<Value>, HttpFault> {
    let args = serde_json::json!({ "article_id": ArticleId(id) });
    relay(&state, &caller, Operation::Trash, &args).await
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
