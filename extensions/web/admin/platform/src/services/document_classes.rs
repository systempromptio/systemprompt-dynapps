//! Document class lookups and writes: assembling the class chain a target
//! resolves through from its lineage rows, and recording a class change.
//!
//! Folder ids arrive root first and ancestors leave nearest first; a target
//! with no live row has no chain.

use sqlx::PgPool;
use uuid::Uuid;

use crate::document_class::ClassChain;
use crate::repositories::document_classes::{
    ClassRow, KIND_DOCUMENT, KIND_FOLDER, Lineage, NewClass, find_document_lineage,
    find_folder_lineage, list_classes, set_class,
};

#[must_use]
pub(crate) fn ancestors_nearest_first(
    folder_ids: &[Uuid],
    classed: &[ClassRow],
) -> Vec<(String, Option<String>)> {
    folder_ids
        .iter()
        .rev()
        .map(|id| {
            let class = classed
                .iter()
                .find(|c| c.target_id == *id)
                .map(|c| c.class_id.clone());
            (id.to_string(), class)
        })
        .collect()
}

async fn chain_of(pool: &PgPool, lineage: Lineage) -> Result<ClassChain, sqlx::Error> {
    let classed = list_classes(pool, KIND_FOLDER, &lineage.folder_ids).await?;
    Ok(ClassChain {
        own: lineage.own,
        ancestors: ancestors_nearest_first(&lineage.folder_ids, &classed),
        created_by_agent: lineage.created_by_agent,
    })
}

pub(crate) async fn find_chain(
    pool: &PgPool,
    kind: &str,
    id: Uuid,
) -> Result<Option<ClassChain>, sqlx::Error> {
    let lineage = match kind {
        KIND_DOCUMENT => find_document_lineage(pool, id).await?,
        KIND_FOLDER => find_folder_lineage(pool, id).await?,
        _ => None,
    };
    match lineage {
        Some(lineage) => Ok(Some(chain_of(pool, lineage).await?)),
        None => Ok(None),
    }
}

pub(crate) async fn list_folder_classes(
    pool: &PgPool,
    folder_ids: &[Uuid],
) -> Result<Vec<ClassRow>, sqlx::Error> {
    list_classes(pool, KIND_FOLDER, folder_ids).await
}

pub(crate) async fn find_document_class(
    pool: &PgPool,
    document_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    Ok(list_classes(pool, KIND_DOCUMENT, &[document_id])
        .await?
        .into_iter()
        .next()
        .map(|row| row.class_id))
}

pub(crate) async fn record_class(pool: &PgPool, new: &NewClass<'_>) -> Result<(), sqlx::Error> {
    set_class(pool, new).await
}
