//! The Brain's operations as one trait, the seam between the transports (the
//! REST routes now, the MCP server and the proxy later) and a source.
//!
//! [`Brain`] has the operations of the `docs_*` tools plus the permission
//! operations, which are not MCP tools. Each takes the person it runs as:
//! nothing here reads a token. [`native::NativeBrain`] implements all of it
//! over the repository except `search` and `answer`, which answer
//! `unavailable` until the retrieval phases. Files of an article are a later
//! phase's operations and are not in the trait yet.

pub mod args;
pub mod fault;
pub mod native;
pub mod views;

use std::fmt::Debug;

use async_trait::async_trait;
use systemprompt::identifiers::UserId;

use crate::ids::ArticleId;
use args::{
    AnswerArgs, CreateArgs, ExportArgs, GetArgs, GrantInput, GrantTarget, InheritArgs, ListArgs,
    ListSpacesArgs, MetadataArgs, MetadataSchemaArgs, MoveArgs, OutlineArgs, SearchArgs, TreeArgs,
    UpdateArgs,
};
use fault::BrainFault;
use views::{
    Answer, ArticleList, ArticleView, Created, Export, Grants, InheritChanged, MetadataList,
    MetadataSchema, Moved, Outline, SearchResults, SpaceList, Trashed, TreeCreated, Updated,
};

pub type Reply<T> = Result<T, BrainFault>;

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
}
