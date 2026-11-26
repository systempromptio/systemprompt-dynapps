//! [`NativeBrain`]: the Brain over this instance's own store.
//!
//! Every operation runs as the person who asks and checks the access of what
//! it touches before it reads or writes: `read` for anything it shows, `write`
//! for anything it changes and `manage` for grants. The checks live in
//! `guard`; reading operations are in `read` and writing ones in `write`.
//! `search` and `answer` answer `unavailable` until the retrieval phases.
//!
//! Files (`files` and `serve`) need [`AssetFiles`]: core's file storage and
//! the link signer. Without them the file operations answer `unavailable`
//! and `get` and `export` give no `asset_urls`.

mod files;
mod grants;
mod guard;
mod outline;
mod read;
mod serve;
mod shape;
mod tree;
mod write;

use async_trait::async_trait;
use systemprompt::identifiers::UserId;

use super::args::{
    AnswerArgs, CreateArgs, ExportArgs, GetArgs, GrantInput, GrantTarget, InheritArgs, ListArgs,
    ListSpacesArgs, MetadataArgs, MetadataSchemaArgs, MoveArgs, OutlineArgs, SearchArgs, TreeArgs,
    UpdateArgs,
};
use super::assets::{
    AssetAttached, AssetFile, AssetLink, AssetLinkArgs, AssetList, AssetListArgs, AssetUploadArgs,
    Received, RequestUploadArgs, UploadTicket,
};
use super::fault::BrainFault;
use super::views::{
    Answer, ArticleList, ArticleView, Created, Export, Grants, InheritChanged, MetadataList,
    MetadataSchema, Moved, Outline, SearchResults, SpaceList, Trashed, TreeCreated, Updated,
};
use super::{Brain, Reply};
use crate::assets::AssetFiles;
use crate::assets::link::SignedLink;
use crate::ids::{ArticleId, AssetId, UploadId};
use crate::repository::BrainRepository;

#[derive(Debug, Clone)]
pub struct NativeBrain {
    repo: BrainRepository,
    files: Option<AssetFiles>,
}

impl NativeBrain {
    pub const fn new(repo: BrainRepository) -> Self {
        Self { repo, files: None }
    }

    pub fn with_files(self, files: Option<AssetFiles>) -> Self {
        Self { files, ..self }
    }

    fn files(&self) -> Result<&AssetFiles, BrainFault> {
        self.files.as_ref().ok_or(BrainFault::Unavailable(
            "files are not available on this instance",
        ))
    }
}

#[async_trait]
impl Brain for NativeBrain {
    async fn search(&self, _caller: &UserId, _args: &SearchArgs) -> Reply<SearchResults> {
        Err(BrainFault::Unavailable(
            "search is not available on this instance yet",
        ))
    }

    async fn answer(&self, _caller: &UserId, _args: &AnswerArgs) -> Reply<Answer> {
        Err(BrainFault::Unavailable(
            "answer is not available on this instance yet",
        ))
    }

    async fn get(&self, caller: &UserId, args: &GetArgs) -> Reply<ArticleView> {
        self.fetch_article(caller, args).await
    }

    async fn list_spaces(&self, caller: &UserId, args: &ListSpacesArgs) -> Reply<SpaceList> {
        self.fetch_spaces(caller, args).await
    }

    async fn list(&self, caller: &UserId, args: &ListArgs) -> Reply<ArticleList> {
        self.fetch_list(caller, args).await
    }

    async fn outline(&self, caller: &UserId, args: &OutlineArgs) -> Reply<Outline> {
        self.fetch_outline(caller, args).await
    }

    async fn export(&self, caller: &UserId, args: &ExportArgs) -> Reply<Export> {
        self.fetch_export(caller, args).await
    }

    async fn metadata(&self, caller: &UserId, args: &MetadataArgs) -> Reply<MetadataList> {
        self.fetch_metadata(caller, args).await
    }

    async fn metadata_schema(
        &self,
        caller: &UserId,
        args: &MetadataSchemaArgs,
    ) -> Reply<MetadataSchema> {
        self.fetch_schema(caller, args).await
    }

    async fn create(&self, caller: &UserId, args: &CreateArgs) -> Reply<Created> {
        self.make_article(caller, args).await
    }

    async fn create_tree(&self, caller: &UserId, args: &TreeArgs) -> Reply<TreeCreated> {
        self.make_tree(caller, args).await
    }

    async fn update(&self, caller: &UserId, args: &UpdateArgs) -> Reply<Updated> {
        self.change_article(caller, args).await
    }

    async fn move_article(&self, caller: &UserId, args: &MoveArgs) -> Reply<Moved> {
        self.relocate(caller, args).await
    }

    async fn trash(&self, caller: &UserId, article: ArticleId) -> Reply<Trashed> {
        self.discard(caller, article).await
    }

    async fn list_grants(&self, caller: &UserId, target: GrantTarget) -> Reply<Grants> {
        self.grants_of(caller, target).await
    }

    async fn set_grants(
        &self,
        caller: &UserId,
        target: GrantTarget,
        grants: &[GrantInput],
    ) -> Reply<Grants> {
        self.replace_grants(caller, target, grants).await
    }

    async fn set_inherit(
        &self,
        caller: &UserId,
        article: ArticleId,
        args: InheritArgs,
    ) -> Reply<InheritChanged> {
        self.switch_inherit(caller, article, args).await
    }

    async fn request_upload(
        &self,
        caller: &UserId,
        args: &RequestUploadArgs,
    ) -> Reply<UploadTicket> {
        self.issue_upload(caller, args).await
    }

    async fn receive_upload(
        &self,
        upload: &UploadId,
        link: &SignedLink,
        bytes: &[u8],
    ) -> Reply<Received> {
        self.take_upload(upload, link, bytes).await
    }

    async fn asset_upload(&self, caller: &UserId, args: &AssetUploadArgs) -> Reply<AssetAttached> {
        self.adopt_upload(caller, args).await
    }

    async fn asset_list(&self, caller: &UserId, args: &AssetListArgs) -> Reply<AssetList> {
        self.files_of(caller, args).await
    }

    async fn asset_link(&self, caller: &UserId, args: &AssetLinkArgs) -> Reply<AssetLink> {
        self.link_for(caller, args).await
    }

    async fn open_asset(&self, caller: &UserId, asset: AssetId) -> Reply<AssetFile> {
        self.open_readable(caller, asset).await
    }

    async fn open_signed(&self, asset: AssetId, link: &SignedLink) -> Reply<AssetFile> {
        self.open_linked(asset, link).await
    }

    fn upload_limit(&self) -> u64 {
        self.files
            .as_ref()
            .map_or(0, |files| files.max_upload_bytes)
    }
}
