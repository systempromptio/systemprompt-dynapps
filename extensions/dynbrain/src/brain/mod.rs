//! The Brain's operations as one trait, the seam between the transports (the
//! REST routes now, the MCP server and the proxy later) and a source.
//!
//! [`Brain`] has the operations of the `docs_*` tools plus the permission
//! operations, which are not MCP tools. Each takes the person it runs as:
//! nothing here reads a token. [`native::NativeBrain`] implements all of it
//! over the repository; its `search` and `answer` run the hybrid retrieval
//! of [`crate::search`], and `answer` returns the assembled context with its
//! citations, never a text written by a model.
//!
//! The file operations are the `docs_*` four (`request_upload`,
//! `asset_upload`, `asset_list`, `asset_link`) plus what the routes need to
//! take the bytes of an upload and to serve a file, to its reader or to the
//! holder of a signed link. No operation moves file bytes through a tool
//! call or through `get`.

pub mod args;
pub mod assets;
pub mod fault;
pub mod found;
pub mod native;
pub mod views;

use std::fmt::Debug;

use async_trait::async_trait;
use systemprompt::identifiers::UserId;

use crate::assets::link::SignedLink;
use crate::ids::{ArticleId, AssetId, UploadId};
use args::{
    AnswerArgs, CreateArgs, ExportArgs, GetArgs, GrantInput, GrantTarget, InheritArgs, ListArgs,
    ListSpacesArgs, MetadataArgs, MetadataSchemaArgs, MoveArgs, OutlineArgs, SearchArgs, TreeArgs,
    UpdateArgs,
};
use assets::{
    AssetAttached, AssetFile, AssetLink, AssetLinkArgs, AssetList, AssetListArgs, AssetUploadArgs,
    Received, RequestUploadArgs, UploadTicket,
};
use fault::BrainFault;
use found::{Answer, SearchResults};
use views::{
    ArticleList, ArticleView, Created, Export, Grants, InheritChanged, MetadataList,
    MetadataSchema, Moved, Outline, SpaceList, Trashed, TreeCreated, Updated,
};

pub type Reply<T> = Result<T, BrainFault>;

/// The Brain's operations. Held as `Arc<dyn Brain>` by the transports, hence
/// `#[async_trait]`.
#[async_trait]
pub trait Brain: Send + Sync + Debug {
    async fn search(&self, caller: &UserId, args: &SearchArgs) -> Reply<SearchResults>;
    async fn answer(&self, caller: &UserId, args: &AnswerArgs) -> Reply<Answer>;
    async fn get(&self, caller: &UserId, args: &GetArgs) -> Reply<ArticleView>;
    async fn list_spaces(&self, caller: &UserId, args: &ListSpacesArgs) -> Reply<SpaceList>;
    async fn list(&self, caller: &UserId, args: &ListArgs) -> Reply<ArticleList>;
    async fn outline(&self, caller: &UserId, args: &OutlineArgs) -> Reply<Outline>;
    async fn export(&self, caller: &UserId, args: &ExportArgs) -> Reply<Export>;
    async fn metadata(&self, caller: &UserId, args: &MetadataArgs) -> Reply<MetadataList>;
    async fn metadata_schema(
        &self,
        caller: &UserId,
        args: &MetadataSchemaArgs,
    ) -> Reply<MetadataSchema>;
    async fn create(&self, caller: &UserId, args: &CreateArgs) -> Reply<Created>;
    async fn create_tree(&self, caller: &UserId, args: &TreeArgs) -> Reply<TreeCreated>;
    async fn update(&self, caller: &UserId, args: &UpdateArgs) -> Reply<Updated>;
    async fn move_article(&self, caller: &UserId, args: &MoveArgs) -> Reply<Moved>;
    async fn trash(&self, caller: &UserId, article: ArticleId) -> Reply<Trashed>;
    async fn list_grants(&self, caller: &UserId, target: GrantTarget) -> Reply<Grants>;
    async fn set_grants(
        &self,
        caller: &UserId,
        target: GrantTarget,
        grants: &[GrantInput],
    ) -> Reply<Grants>;
    async fn set_inherit(
        &self,
        caller: &UserId,
        article: ArticleId,
        args: InheritArgs,
    ) -> Reply<InheritChanged>;
    async fn request_upload(
        &self,
        caller: &UserId,
        args: &RequestUploadArgs,
    ) -> Reply<UploadTicket>;
    async fn receive_upload(
        &self,
        upload: &UploadId,
        link: &SignedLink,
        bytes: &[u8],
    ) -> Reply<Received>;
    async fn asset_upload(&self, caller: &UserId, args: &AssetUploadArgs) -> Reply<AssetAttached>;
    async fn asset_list(&self, caller: &UserId, args: &AssetListArgs) -> Reply<AssetList>;
    async fn asset_link(&self, caller: &UserId, args: &AssetLinkArgs) -> Reply<AssetLink>;
    async fn open_asset(&self, caller: &UserId, asset: AssetId) -> Reply<AssetFile>;
    async fn open_signed(&self, asset: AssetId, link: &SignedLink) -> Reply<AssetFile>;
    fn upload_limit(&self) -> u64;
}
