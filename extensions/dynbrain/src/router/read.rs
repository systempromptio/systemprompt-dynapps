//! The reading routes. Each hands its parameters to the [`Brain`] as the
//! person the token names; the path's id overwrites any id in the query.

use axum::Json;
use axum::extract::State;

use super::error::HttpFault;
use super::extract::{Id, Inference, Params, Payload};
use super::state::BrainState;
use crate::auth::Caller;
use crate::brain::args::{
    AnswerArgs, ExportArgs, GetArgs, GrantTarget, ListArgs, ListSpacesArgs, MetadataArgs,
    MetadataSchemaArgs, OutlineArgs, SearchArgs,
};
use crate::brain::found::{Answer, SearchResults};
use crate::brain::views::{
    ArticleList, ArticleView, Export, Grants, MetadataList, MetadataSchema, Outline, SpaceList,
};
use crate::ids::{ArticleId, SpaceId};

pub async fn spaces(
    caller: Caller,
    State(state): State<BrainState>,
    Params(args): Params<ListSpacesArgs>,
) -> Result<Json<SpaceList>, HttpFault> {
    Ok(Json(state.brain.list_spaces(&caller.0, &args).await?))
}

pub async fn articles(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Params(mut args): Params<ListArgs>,
) -> Result<Json<ArticleList>, HttpFault> {
    args.space = SpaceId(id);
    Ok(Json(state.brain.list(&caller.0, &args).await?))
}

pub async fn outline(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Params(mut args): Params<OutlineArgs>,
) -> Result<Json<Outline>, HttpFault> {
    args.space = Some(SpaceId(id));
    Ok(Json(state.brain.outline(&caller.0, &args).await?))
}

pub async fn export(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Params(mut args): Params<ExportArgs>,
) -> Result<Json<Export>, HttpFault> {
    args.space = SpaceId(id);
    Ok(Json(state.brain.export(&caller.0, &args).await?))
}

pub async fn schema(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
) -> Result<Json<MetadataSchema>, HttpFault> {
    let args = MetadataSchemaArgs {
        space: Some(SpaceId(id)),
        ..MetadataSchemaArgs::default()
    };
    Ok(Json(state.brain.metadata_schema(&caller.0, &args).await?))
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
) -> Result<Json<ArticleView>, HttpFault> {
    args.article_id = Some(ArticleId(id));
    args.path = None;
    Ok(Json(state.brain.get(&caller.0, &args).await?))
}

pub async fn metadata(
    caller: Caller,
    State(state): State<BrainState>,
    Id(id): Id,
    Params(mut args): Params<MetadataArgs>,
) -> Result<Json<MetadataList>, HttpFault> {
    args.article_id = Some(ArticleId(id));
    args.path = None;
    args.article_ids = None;
    Ok(Json(state.brain.metadata(&caller.0, &args).await?))
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
) -> Result<Json<SearchResults>, HttpFault> {
    state.fill_embedder(ai);
    Ok(Json(state.brain.search(&caller.0, &args).await?))
}

pub async fn answer(
    caller: Caller,
    State(state): State<BrainState>,
    Inference(ai): Inference,
    Payload(args): Payload<AnswerArgs>,
) -> Result<Json<Answer>, HttpFault> {
    state.fill_embedder(ai);
    Ok(Json(state.brain.answer(&caller.0, &args).await?))
}
