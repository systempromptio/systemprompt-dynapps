//! The reading routes. Each hands its parameters to the [`crate::brain::Brain`]
//! as the person the token names; the path's id overwrites any id in the
//! query.

use axum::Json;
use axum::extract::State;
// JSON: an operation's reply, as the Brain answered it.
use serde_json::Value;

use super::error::HttpFault;
use super::extract::{Id, Inference, Params, Payload};
use super::relay::relay;
use super::state::BrainState;
use crate::auth::Caller;
use crate::brain::args::{
    AnswerArgs, ExportArgs, GetArgs, GrantTarget, ListArgs, ListSpacesArgs, MetadataArgs,
    MetadataSchemaArgs, OutlineArgs, SearchArgs,
};
use crate::brain::operation::Operation;
use crate::brain::views::Grants;
use crate::ids::{ArticleId, SpaceId};

pub async fn spaces(
    caller: Caller,
    State(state): State<BrainState>,
    Params(args): Params<ListSpacesArgs>,
) -> Result<Json<Value>, HttpFault> {
    relay(&state, &caller, Operation::ListSpaces, &args).await
}

pub async fn articles(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Params(mut args): Params<ListArgs>,
) -> Result<Json<Value>, HttpFault> {
    args.space = SpaceId(id);
    relay(&state, &caller, Operation::List, &args).await
}

pub async fn outline(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Params(mut args): Params<OutlineArgs>,
) -> Result<Json<Value>, HttpFault> {
    args.space = Some(SpaceId(id));
    relay(&state, &caller, Operation::Outline, &args).await
}

pub async fn export(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Params(mut args): Params<ExportArgs>,
) -> Result<Json<Value>, HttpFault> {
    args.space = SpaceId(id);
    relay(&state, &caller, Operation::Export, &args).await
}

pub async fn schema(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
) -> Result<Json<Value>, HttpFault> {
    let args = MetadataSchemaArgs {
        space: Some(SpaceId(id)),
        ..MetadataSchemaArgs::default()
    };
    relay(&state, &caller, Operation::MetadataSchema, &args).await
}

pub async fn space_grants(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
) -> Result<Json<Grants>, HttpFault> {
    let target = GrantTarget::Space(SpaceId(id));
    Ok(Json(state.brain.list_grants(&caller.0, target).await?))
}

pub async fn article(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Params(mut args): Params<GetArgs>,
) -> Result<Json<Value>, HttpFault> {
    args.article_id = Some(ArticleId(id));
    args.path = None;
    relay(&state, &caller, Operation::Get, &args).await
}

pub async fn metadata(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Params(mut args): Params<MetadataArgs>,
) -> Result<Json<Value>, HttpFault> {
    args.article_id = Some(ArticleId(id));
    args.path = None;
    args.article_ids = None;
    relay(&state, &caller, Operation::Metadata, &args).await
}

pub async fn article_grants(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
) -> Result<Json<Grants>, HttpFault> {
    let target = GrantTarget::Article(ArticleId(id));
    Ok(Json(state.brain.list_grants(&caller.0, target).await?))
}

pub async fn search(
    caller: Caller,
    State(state): State<BrainState>,
    Inference(ai): Inference,
    Payload(args): Payload<SearchArgs>,
) -> Result<Json<Value>, HttpFault> {
    state.fill_embedder(ai);
    relay(&state, &caller, Operation::Search, &args).await
}

pub async fn answer(
    caller: Caller,
    State(state): State<BrainState>,
    Inference(ai): Inference,
    Payload(args): Payload<AnswerArgs>,
) -> Result<Json<Value>, HttpFault> {
    state.fill_embedder(ai);
    relay(&state, &caller, Operation::Answer, &args).await
}
