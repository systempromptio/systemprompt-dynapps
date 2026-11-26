//! The eighteen operations a transport runs by name, with their arguments
//! and reply as JSON.
//!
//! The REST routes and the `brain` MCP server both call [`super::Brain::run`]
//! with one of these. The native Brain answers through [`dispatch`], which
//! reads the arguments into the typed ones and serializes the typed reply;
//! a proxy Brain passes them to the source's tool and returns its answer.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
// JSON: the arguments and the reply of an operation are the JSON a tool call
// carries.
use serde_json::Value;
use systemprompt::identifiers::UserId;

use super::fault::BrainFault;
use super::{Brain, Reply};
use crate::ids::ArticleId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Search,
    Answer,
    Get,
    ListSpaces,
    List,
    Outline,
    Export,
    Metadata,
    MetadataSchema,
    Create,
    CreateTree,
    Update,
    Move,
    Trash,
    RequestUpload,
    AssetUpload,
    AssetList,
    AssetLink,
}

impl Operation {
    pub const ALL: [Self; 18] = [
        Self::Search,
        Self::Answer,
        Self::Get,
        Self::ListSpaces,
        Self::List,
        Self::Outline,
        Self::Export,
        Self::Metadata,
        Self::MetadataSchema,
        Self::Create,
        Self::CreateTree,
        Self::Update,
        Self::Move,
        Self::Trash,
        Self::RequestUpload,
        Self::AssetUpload,
        Self::AssetList,
        Self::AssetLink,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Search => "search",
            Self::Answer => "answer",
            Self::Get => "get",
            Self::ListSpaces => "list_spaces",
            Self::List => "list",
            Self::Outline => "outline",
            Self::Export => "export",
            Self::Metadata => "metadata",
            Self::MetadataSchema => "metadata_schema",
            Self::Create => "create",
            Self::CreateTree => "create_tree",
            Self::Update => "update",
            Self::Move => "move",
            Self::Trash => "trash",
            Self::RequestUpload => "request_upload",
            Self::AssetUpload => "asset_upload",
            Self::AssetList => "asset_list",
            Self::AssetLink => "asset_link",
        }
    }

    pub fn from_tool(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|operation| operation.name() == name)
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(default)]
struct TrashArgs {
    article_id: ArticleId,
}

// JSON: arguments in, the typed reply serialized out.
pub async fn dispatch<B: Brain + ?Sized>(
    brain: &B,
    caller: &UserId,
    operation: Operation,
    args: Value,
) -> Reply<Value> {
    match operation {
        Operation::Search => encode(brain.search(caller, &parse(args)?).await),
        Operation::Answer => encode(brain.answer(caller, &parse(args)?).await),
        Operation::Get => encode(brain.get(caller, &parse(args)?).await),
        Operation::ListSpaces => encode(brain.list_spaces(caller, &parse(args)?).await),
        Operation::List => encode(brain.list(caller, &parse(args)?).await),
        Operation::Outline => encode(brain.outline(caller, &parse(args)?).await),
        Operation::Export => encode(brain.export(caller, &parse(args)?).await),
        Operation::Metadata => encode(brain.metadata(caller, &parse(args)?).await),
        Operation::MetadataSchema => encode(brain.metadata_schema(caller, &parse(args)?).await),
        Operation::Create => encode(brain.create(caller, &parse(args)?).await),
        Operation::CreateTree => encode(brain.create_tree(caller, &parse(args)?).await),
        Operation::Update => encode(brain.update(caller, &parse(args)?).await),
        Operation::Move => encode(brain.move_article(caller, &parse(args)?).await),
        Operation::Trash => {
            let target: TrashArgs = parse(args)?;
            encode(brain.trash(caller, target.article_id).await)
        },
        Operation::RequestUpload => encode(brain.request_upload(caller, &parse(args)?).await),
        Operation::AssetUpload => encode(brain.asset_upload(caller, &parse(args)?).await),
        Operation::AssetList => encode(brain.asset_list(caller, &parse(args)?).await),
        Operation::AssetLink => encode(brain.asset_link(caller, &parse(args)?).await),
    }
}

fn parse<A: DeserializeOwned>(args: Value) -> Reply<A> {
    serde_json::from_value(args).map_err(BrainFault::Arguments)
}

fn encode<T: Serialize>(reply: Reply<T>) -> Reply<Value> {
    serde_json::to_value(reply?).map_err(BrainFault::Encode)
}
