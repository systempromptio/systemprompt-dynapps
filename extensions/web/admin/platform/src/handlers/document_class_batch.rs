//! `GET /drive/classes/{kind}?ids=…` — class labels for the rows of a Drive
//! listing, so the browser can badge confidential and restricted rows.
//!
//! Only ids the caller can view are answered: documents through core's
//! batch visibility check, folders one access check each. A guessed id the
//! caller cannot see is skipped exactly like one that does not exist.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::files::documents::DocumentService;
use systemprompt::identifiers::{DocumentFolderId, DocumentId, TraceId};
use systemprompt::models::auth::AuthenticatedUser;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;
use uuid::Uuid;

use super::document_class::{ClassDeps, classes, unresolved};
use super::ssr::drive_caller as caller;
use crate::document_class::resolve;
use crate::repositories::document_classes::{KIND_DOCUMENT, KIND_FOLDER};
use crate::services::document_classes::find_chain;

const MAX_DOCUMENTS: usize = 200;
const MAX_FOLDERS: usize = 50;

#[derive(Debug, Deserialize)]
pub(crate) struct Batch {
    ids: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct BatchEntry {
    id: String,
    class_id: String,
    label: String,
}

async fn visible_documents(
    service: &DocumentService,
    who: &AuthenticatedUser,
    raw: &[&str],
    trace: &TraceId,
) -> AdminResult<Vec<String>> {
    let ids: Vec<DocumentId> = raw
        .iter()
        .take(MAX_DOCUMENTS)
        .filter_map(|id| DocumentId::try_new((*id).to_owned()).ok())
        .collect();
    let visible = service
        .visible_documents(who, &ids, trace)
        .await
        .map_err(AdminError::internal)?;
    Ok(visible
        .into_iter()
        .map(|id| id.as_str().to_owned())
        .collect())
}

async fn visible_folders(
    service: &DocumentService,
    who: &AuthenticatedUser,
    raw: &[&str],
    trace: &TraceId,
) -> Vec<String> {
    let mut out = Vec::new();
    for id in raw.iter().take(MAX_FOLDERS) {
        let Ok(folder) = DocumentFolderId::try_new((*id).to_owned()) else {
            continue;
        };
        if service
            .folder_access_of(who, &folder, false, trace)
            .await
            .is_ok()
        {
            out.push((*id).to_owned());
        }
    }
    out
}

pub(crate) async fn list_classes_handler(
    State(pool): State<Arc<PgPool>>,
    deps: ClassDeps,
    Extension(user): Extension<UserContext>,
    Path(kind): Path<String>,
    Query(batch): Query<Batch>,
) -> AdminResult<Json<Vec<BatchEntry>>> {
    let raw: Vec<&str> = batch.ids.split(',').filter(|s| !s.is_empty()).collect();
    let who = caller(&user);
    let trace = TraceId::generate();
    let visible = match kind.as_str() {
        KIND_DOCUMENT => visible_documents(&deps.documents.0, &who, &raw, &trace).await?,
        KIND_FOLDER => visible_folders(&deps.documents.0, &who, &raw, &trace).await,
        _ => return Err(AdminError::NotFound("Not found.".to_owned())),
    };
    let catalog = classes(&deps.declared)?;
    let mut out = Vec::with_capacity(visible.len());
    for id in visible {
        let Ok(uuid) = Uuid::parse_str(&id) else {
            continue;
        };
        if let Some(chain) = find_chain(&pool, &kind, uuid).await? {
            let class = resolve(catalog, &chain).ok_or_else(unresolved)?.class;
            out.push(BatchEntry {
                id,
                class_id: class.id.clone(),
                label: class.label.clone(),
            });
        }
    }
    Ok(Json(out))
}
