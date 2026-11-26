//! From a tool call to the operation of the Brain and back to its reply.

use dynbrain::brain::Brain;
use dynbrain::brain::args::{
    AnswerArgs, CreateArgs, ExportArgs, GetArgs, ListArgs, ListSpacesArgs, MetadataArgs,
    MetadataSchemaArgs, MoveArgs, OutlineArgs, TreeArgs, UpdateArgs,
};
use dynbrain::brain::assets::{AssetLinkArgs, AssetListArgs, AssetUploadArgs, RequestUploadArgs};
use dynbrain::brain::fault::BrainFault;
use dynbrain::ids::ArticleId;
use rmcp::ErrorData as McpError;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::UserId;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_shared::reply;

use super::Json;
use super::fault::{tool_error, undeclared_filter};

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(default)]
struct Trash {
    article_id: ArticleId,
}

// Why: the person is the one the authenticated request names, never one an
// argument names; a call that carries none is refused before any operation.
pub async fn run(
    brain: &dyn Brain,
    context: &RequestContext,
    name: &str,
    arguments: Json,
) -> Result<Json, McpError> {
    let caller = &context
        .user
        .as_ref()
        .ok_or_else(|| reply::forbidden("a signed-in person is required"))?
        .id;
    match name {
        "search" => match brain.search(caller, &parse(arguments)?).await {
            Err(fault) => undeclared_filter(fault, "results"),
            found => encode(found),
        },
        "answer" => encode(brain.answer(caller, &parse::<AnswerArgs>(arguments)?).await),
        "get" => encode(brain.get(caller, &parse::<GetArgs>(arguments)?).await),
        "list_spaces" => encode(
            brain
                .list_spaces(caller, &parse::<ListSpacesArgs>(arguments)?)
                .await,
        ),
        "list" => match brain.list(caller, &parse::<ListArgs>(arguments)?).await {
            Err(fault) => undeclared_filter(fault, "documents"),
            listed => encode(listed),
        },
        "outline" => encode(brain.outline(caller, &parse::<OutlineArgs>(arguments)?).await),
        "export" => encode(brain.export(caller, &parse::<ExportArgs>(arguments)?).await),
        "metadata" => encode(
            brain
                .metadata(caller, &parse::<MetadataArgs>(arguments)?)
                .await,
        ),
        "metadata_schema" => encode(
            brain
                .metadata_schema(caller, &parse::<MetadataSchemaArgs>(arguments)?)
                .await,
        ),
        "create" => encode(brain.create(caller, &parse::<CreateArgs>(arguments)?).await),
        "create_tree" => encode(
            brain
                .create_tree(caller, &parse::<TreeArgs>(arguments)?)
                .await,
        ),
        "update" => encode(brain.update(caller, &parse::<UpdateArgs>(arguments)?).await),
        "move" => encode(
            brain
                .move_article(caller, &parse::<MoveArgs>(arguments)?)
                .await,
        ),
        "trash" => encode(
            brain
                .trash(caller, parse::<Trash>(arguments)?.article_id)
                .await,
        ),
        other => run_files(brain, caller, other, arguments).await,
    }
}

async fn run_files(
    brain: &dyn Brain,
    caller: &UserId,
    name: &str,
    arguments: Json,
) -> Result<Json, McpError> {
    match name {
        "request_upload" => encode(
            brain
                .request_upload(caller, &parse::<RequestUploadArgs>(arguments)?)
                .await,
        ),
        "asset_list" => encode(
            brain
                .asset_list(caller, &parse::<AssetListArgs>(arguments)?)
                .await,
        ),
        "asset_link" => encode(
            brain
                .asset_link(caller, &parse::<AssetLinkArgs>(arguments)?)
                .await,
        ),
        "asset_upload" => asset_upload(brain, caller, arguments).await,
        other => Err(reply::invalid(format!("Unknown tool: {other}"))),
    }
}

// Why: the file contract is two calls to one tool; the first one, with no
// `upload_id`, is the request for the URL the bytes are sent to.
async fn asset_upload(
    brain: &dyn Brain,
    caller: &UserId,
    arguments: Json,
) -> Result<Json, McpError> {
    let args = parse::<AssetUploadArgs>(arguments)?;
    if args.upload_id.is_some() {
        return encode(brain.asset_upload(caller, &args).await);
    }
    let ticket = RequestUploadArgs {
        article_id: args.article_id,
        path: args.path,
        filename: args.filename,
    };
    encode(brain.request_upload(caller, &ticket).await)
}

fn parse<A: DeserializeOwned>(arguments: Json) -> Result<A, McpError> {
    match serde_json::from_value(arguments) {
        Ok(args) => Ok(args),
        Err(error) => Err(reply::invalid(format!("Invalid tool input: {error}"))),
    }
}

fn encode<T: Serialize>(outcome: Result<T, BrainFault>) -> Result<Json, McpError> {
    let view = outcome.map_err(|fault| tool_error(&fault))?;
    serde_json::to_value(view).map_err(|error| {
        tracing::error!(%error, "a Brain reply could not be encoded");
        reply::failure("the reply could not be encoded")
    })
}
