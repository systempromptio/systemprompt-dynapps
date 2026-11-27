//! [`super::ProxyBrain`] as a [`Brain`].
//!
//! The transports reach it through `run`, which relays the operation to the
//! source, and `asset_redirect`, which asks the source for a file's link. The
//! typed methods of the eighteen operations are not a way in for a proxy
//! instance, because the source answers with its own JSON, so they answer
//! `unavailable`. The grant operations answer `managed_at_source`, and the
//! routes that take upload bytes or serve signed downloads of this instance
//! find nothing: the source's own URLs carry the bytes.

use async_trait::async_trait;
// JSON: the arguments and the reply of `run`.
use serde_json::{Value, json};
use systemprompt::identifiers::UserId;
use systemprompt::models::execution::context::RequestContext;

use super::ProxyBrain;
use crate::assets::link::SignedLink;
use crate::brain::Reply;
use crate::brain::args::{
    AnswerArgs, CreateArgs, ExportArgs, GetArgs, GrantInput, GrantTarget, InheritArgs, ListArgs,
    ListSpacesArgs, MetadataArgs, MetadataSchemaArgs, MoveArgs, OutlineArgs, SearchArgs, TreeArgs,
    UpdateArgs,
};
use crate::brain::assets::{
    AssetAttached, AssetFile, AssetLink, AssetLinkArgs, AssetList, AssetListArgs, AssetUploadArgs,
    Received, RequestUploadArgs, UploadTicket,
};
use crate::brain::fault::{BrainFault, SourceFault};
use crate::brain::found::{Answer, SearchResults};
use crate::brain::operation::Operation;
use crate::brain::views::{
    ArticleList, ArticleView, Created, Export, Grants, InheritChanged, MetadataList,
    MetadataSchema, Moved, Outline, SpaceList, Trashed, TreeCreated, Updated,
};
use crate::ids::{ArticleId, AssetId, UploadId};

const THROUGH_RUN: BrainFault =
    BrainFault::Unavailable("a proxy instance answers each operation through its source");

#[async_trait]
impl crate::brain::Brain for ProxyBrain {
    async fn search(&self, _caller: &UserId, _args: &SearchArgs) -> Reply<SearchResults> {
        Err(THROUGH_RUN)
    }
    async fn answer(&self, _caller: &UserId, _args: &AnswerArgs) -> Reply<Answer> {
        Err(THROUGH_RUN)
    }
    async fn get(&self, _caller: &UserId, _args: &GetArgs) -> Reply<ArticleView> {
        Err(THROUGH_RUN)
    }
    async fn list_spaces(&self, _caller: &UserId, _args: &ListSpacesArgs) -> Reply<SpaceList> {
        Err(THROUGH_RUN)
    }
    async fn list(&self, _caller: &UserId, _args: &ListArgs) -> Reply<ArticleList> {
        Err(THROUGH_RUN)
    }
    async fn outline(&self, _caller: &UserId, _args: &OutlineArgs) -> Reply<Outline> {
        Err(THROUGH_RUN)
    }
    async fn export(&self, _caller: &UserId, _args: &ExportArgs) -> Reply<Export> {
        Err(THROUGH_RUN)
    }
    async fn metadata(&self, _caller: &UserId, _args: &MetadataArgs) -> Reply<MetadataList> {
        Err(THROUGH_RUN)
    }
    async fn metadata_schema(
        &self,
        _caller: &UserId,
        _args: &MetadataSchemaArgs,
    ) -> Reply<MetadataSchema> {
        Err(THROUGH_RUN)
    }
    async fn create(&self, _caller: &UserId, _args: &CreateArgs) -> Reply<Created> {
        Err(THROUGH_RUN)
    }
    async fn create_tree(&self, _caller: &UserId, _args: &TreeArgs) -> Reply<TreeCreated> {
        Err(THROUGH_RUN)
    }
    async fn update(&self, _caller: &UserId, _args: &UpdateArgs) -> Reply<Updated> {
        Err(THROUGH_RUN)
    }
    async fn move_article(&self, _caller: &UserId, _args: &MoveArgs) -> Reply<Moved> {
        Err(THROUGH_RUN)
    }
    async fn trash(&self, _caller: &UserId, _article: ArticleId) -> Reply<Trashed> {
        Err(THROUGH_RUN)
    }
    async fn list_grants(&self, _caller: &UserId, _target: GrantTarget) -> Reply<Grants> {
        Err(BrainFault::ManagedAtSource)
    }
    async fn set_grants(
        &self,
        _caller: &UserId,
        _target: GrantTarget,
        _grants: &[GrantInput],
    ) -> Reply<Grants> {
        Err(BrainFault::ManagedAtSource)
    }
    async fn set_inherit(
        &self,
        _caller: &UserId,
        _article: ArticleId,
        _args: InheritArgs,
    ) -> Reply<InheritChanged> {
        Err(BrainFault::ManagedAtSource)
    }
    async fn request_upload(
        &self,
        _caller: &UserId,
        _args: &RequestUploadArgs,
    ) -> Reply<UploadTicket> {
        Err(THROUGH_RUN)
    }
    async fn receive_upload(
        &self,
        _upload: &UploadId,
        _link: &SignedLink,
        _bytes: &[u8],
    ) -> Reply<Received> {
        Err(BrainFault::NotFound)
    }
    async fn asset_upload(
        &self,
        _caller: &UserId,
        _args: &AssetUploadArgs,
    ) -> Reply<AssetAttached> {
        Err(THROUGH_RUN)
    }
    async fn asset_list(&self, _caller: &UserId, _args: &AssetListArgs) -> Reply<AssetList> {
        Err(THROUGH_RUN)
    }
    async fn asset_link(&self, _caller: &UserId, _args: &AssetLinkArgs) -> Reply<AssetLink> {
        Err(THROUGH_RUN)
    }
    async fn open_asset(&self, _caller: &UserId, _asset: AssetId) -> Reply<AssetFile> {
        Err(BrainFault::NotFound)
    }
    async fn open_signed(&self, _asset: AssetId, _link: &SignedLink) -> Reply<AssetFile> {
        Err(BrainFault::NotFound)
    }
    fn upload_limit(&self) -> u64 {
        0
    }

    async fn run(
        &self,
        _caller: &UserId,
        context: &RequestContext,
        operation: Operation,
        args: Value,
    ) -> Reply<Value> {
        self.relay(context, operation, args).await
    }

    // Why: a file the person may not read at the source is the same `not
    // found` as a missing one, as for native files.
    async fn asset_redirect(
        &self,
        _caller: &UserId,
        context: &RequestContext,
        asset: AssetId,
    ) -> Reply<Option<String>> {
        let link = self
            .relay(
                context,
                Operation::AssetLink,
                json!({ "attachment_id": asset }),
            )
            .await
            .map_err(|fault| match fault {
                BrainFault::Source(SourceFault::Refused(_) | SourceFault::NotFound(_)) => {
                    tracing::debug!(error = %fault, "dynbrain: the source refused a file link");
                    BrainFault::NotFound
                },
                other => other,
            })?;
        link.get("download_url")
            .and_then(Value::as_str)
            .map(|url| Some(url.to_owned()))
            .ok_or(BrainFault::NotFound)
    }
}
