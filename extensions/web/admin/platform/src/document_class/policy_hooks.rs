//! The move and copy hooks of [`DocumentClassPolicy`]: they apply
//! [`super::movement`]'s decisions by pinning a class row onto the target.
//!
//! A move whose lookup fails is refused, so a class is never lost to a
//! transient error; a copy whose lookup fails leaves the copy to resolve on
//! its own and logs it.

use std::sync::Arc;

use systemprompt::files::documents::agents::{DocumentContext, MoveActor, MoveRefused};
use systemprompt::identifiers::{DocumentFolderId, DocumentId};
use uuid::Uuid;

use super::catalog::CatalogError;
use super::movement::{MoveOutcome, on_copy, on_move};
use super::policy::DocumentClassPolicy;
use crate::repositories::document_classes::{KIND_DOCUMENT, KIND_FOLDER, NewClass};
use crate::services::document_classes::{
    ancestors_nearest_first, find_chain, list_folder_classes, record_class,
};

const PINNED_ON_MOVE: &str = "pinned when moved into a looser folder";
const PINNED_ON_COPY: &str = "kept from the source when copied";

#[derive(Debug, thiserror::Error)]
pub(crate) enum ClassHookError {
    #[error(transparent)]
    Catalog(#[from] Arc<CatalogError>),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("the copy id is not a uuid: {0}")]
    CopyId(#[from] uuid::Error),
    #[error("the moved target has no id")]
    NoTarget,
    #[error("the copy does not exist")]
    NoCopy,
}

impl From<ClassHookError> for MoveRefused {
    fn from(error: ClassHookError) -> Self {
        let reason = match error {
            ClassHookError::NoTarget => error.to_string(),
            other => format!("the document class could not be checked: {other}"),
        };
        Self { reason }
    }
}

fn uuids(ids: impl Iterator<Item = String>) -> Vec<Uuid> {
    ids.filter_map(|id| Uuid::parse_str(&id).ok()).collect()
}

fn target(doc: &DocumentContext) -> Option<(&'static str, Uuid)> {
    doc.document_id.as_ref().map_or_else(
        || {
            doc.folders
                .first()
                .and_then(|f| Uuid::parse_str(f.as_str()).ok())
                .map(|u| (KIND_FOLDER, u))
        },
        |id| {
            Uuid::parse_str(id.as_str())
                .ok()
                .map(|u| (KIND_DOCUMENT, u))
        },
    )
}

impl DocumentClassPolicy {
    pub(super) async fn move_hook(
        &self,
        doc: &DocumentContext,
        to: &[DocumentFolderId],
        mover: &MoveActor,
    ) -> Result<(), ClassHookError> {
        let catalog = self.classes.get()?;
        let current = self.chain(doc).await?;
        let root_first = uuids(to.iter().rev().map(|f| f.as_str().to_owned()));
        let classed = list_folder_classes(&self.pool, &root_first).await?;
        let admin_reason = mover.reason.as_deref().filter(|_| mover.is_admin);
        let destination = ancestors_nearest_first(&root_first, &classed);
        let MoveOutcome::Pin(class_id) = on_move(catalog, &current, destination, admin_reason)
        else {
            return Ok(());
        };
        let (kind, id) = target(doc).ok_or(ClassHookError::NoTarget)?;
        record_class(
            &self.pool,
            &NewClass {
                target_kind: kind,
                target_id: id,
                class_id: &class_id,
                set_by: mover.user.as_str(),
                reason: Some(PINNED_ON_MOVE),
                loosened: false,
            },
        )
        .await?;
        Ok(())
    }

    pub(super) async fn copy_hook(&self, source: &DocumentContext, copy: &DocumentId) {
        if let Err(error) = self.pin_copy(source, copy).await {
            tracing::error!(%error, copy = copy.as_str(), "copy class could not be pinned");
        }
    }

    async fn pin_copy(
        &self,
        source: &DocumentContext,
        copy: &DocumentId,
    ) -> Result<(), ClassHookError> {
        let catalog = self.classes.get()?;
        let copy_id = Uuid::parse_str(copy.as_str())?;
        let from = self.chain(source).await?;
        let landed = find_chain(&self.pool, KIND_DOCUMENT, copy_id)
            .await?
            .ok_or(ClassHookError::NoCopy)?;
        let Some(class_id) = on_copy(catalog, &from, &landed) else {
            return Ok(());
        };
        record_class(
            &self.pool,
            &NewClass {
                target_kind: KIND_DOCUMENT,
                target_id: copy_id,
                class_id: &class_id,
                set_by: source.on_behalf_of.as_str(),
                reason: Some(PINNED_ON_COPY),
                loosened: false,
            },
        )
        .await?;
        Ok(())
    }
}
