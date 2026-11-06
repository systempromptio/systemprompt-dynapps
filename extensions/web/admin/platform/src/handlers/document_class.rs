//! `/drive/classes/…` — read and set the document class on a folder or
//! document, and the effective agent caps it grants.
//!
//! Reading needs view access to the target; setting needs ownership. A
//! change may only tighten (see `document_class::resolve`); an admin may
//! loosen with a reason, which is recorded on the row. Access is asked of
//! core's document service as the caller, so a target the caller cannot see
//! is a 404 here exactly as it is in Drive.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::files::documents::DocumentsError;
use systemprompt::files::documents::agents::{AgentCap, current_actor};
use systemprompt::identifiers::{DocumentFolderId, DocumentId, TraceId};
use systemprompt::models::documents::AccessLevel;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::UserContext;
use uuid::Uuid;

use super::ssr::drive_caller as caller;
use crate::document_class::{
    ChangeKind, ChangeRequest, ClassCatalog, ClassChain, ClassSource, DeclaredClasses,
    check_change, resolve,
};
use crate::repositories::document_classes::{KIND_DOCUMENT, KIND_FOLDER, NewClass};
use crate::routes::drive_user::DriveDocuments;
use crate::services::document_classes::{find_chain, record_class};

#[derive(Clone)]
pub(crate) struct ClassDeps {
    pub(crate) documents: DriveDocuments,
    pub(crate) declared: DeclaredClasses,
}

impl std::fmt::Debug for ClassDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClassDeps").finish_non_exhaustive()
    }
}

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ClassDeps {
    type Rejection = axum::extract::rejection::ExtensionRejection;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Extension(documents) =
            Extension::<DriveDocuments>::from_request_parts(parts, state).await?;
        let Extension(declared) =
            Extension::<DeclaredClasses>::from_request_parts(parts, state).await?;
        Ok(Self {
            documents,
            declared,
        })
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ClassOption {
    id: String,
    label: String,
    reason: String,
    attended: AgentCap,
    unattended: AgentCap,
}

#[derive(Debug, Serialize)]
pub(crate) struct ClassView {
    class_id: String,
    label: String,
    reason: String,
    inherited_from: Option<String>,
    is_default: bool,
    attended: AgentCap,
    unattended: AgentCap,
    can_change: bool,
    can_loosen: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SetClass {
    class_id: String,
    reason: Option<String>,
}

pub(super) fn classes(declared: &DeclaredClasses) -> AdminResult<&ClassCatalog> {
    declared.get().map_err(AdminError::internal)
}

pub(super) fn unresolved() -> AdminError {
    AdminError::internal("the class catalog declares no default class")
}

pub(crate) async fn list_class_options(
    Extension(declared): Extension<DeclaredClasses>,
) -> AdminResult<Json<Vec<ClassOption>>> {
    Ok(Json(
        classes(&declared)?
            .classes
            .iter()
            .map(|c| ClassOption {
                id: c.id.clone(),
                label: c.label.clone(),
                reason: c.reason.clone(),
                attended: c.attended,
                unattended: c.unattended,
            })
            .collect(),
    ))
}

fn view(
    catalog: &ClassCatalog,
    chain: &ClassChain,
    level: AccessLevel,
    user: &UserContext,
) -> AdminResult<ClassView> {
    let resolved = resolve(catalog, chain).ok_or_else(unresolved)?;
    let class = resolved.class;
    let mut caps = (class.attended, class.unattended);
    let mut reason = class.reason.clone();
    let (inherited_from, is_default) = match resolved.source {
        ClassSource::Own => (None, false),
        ClassSource::Folder(id) => (Some(id), false),
        ClassSource::Default => (None, true),
        ClassSource::Unknown(id) => {
            caps = (AgentCap::None, AgentCap::None);
            reason = format!(
                "a class row names `{id}`, which is not declared; agents are blocked until it is reset"
            );
            (None, false)
        },
    };
    Ok(ClassView {
        class_id: class.id.clone(),
        label: class.label.clone(),
        reason,
        inherited_from,
        is_default,
        attended: caps.0,
        unattended: caps.1,
        can_change: level >= AccessLevel::Own || user.is_admin,
        can_loosen: user.is_admin,
    })
}

pub(super) fn hidden(error: DocumentsError) -> AdminError {
    match error {
        DocumentsError::Forbidden { .. }
        | DocumentsError::NotFound { .. }
        | DocumentsError::Trashed { .. } => AdminError::NotFound("Not found.".to_owned()),
        other => AdminError::internal(other),
    }
}

fn parse_uuid(raw: &str) -> AdminResult<Uuid> {
    Uuid::parse_str(raw).map_err(|_bad| AdminError::NotFound("Not found.".to_owned()))
}

async fn access_and_chain(
    pool: &PgPool,
    documents: &DriveDocuments,
    user: &UserContext,
    kind: &str,
    raw_id: &str,
) -> AdminResult<(Uuid, AccessLevel, ClassChain)> {
    let id = parse_uuid(raw_id)?;
    let who = caller(user);
    let trace = TraceId::generate();
    let not_found = || AdminError::NotFound("Not found.".to_owned());
    let (level, chain) = match kind {
        KIND_DOCUMENT => {
            let doc = DocumentId::try_new(raw_id.to_owned()).map_err(|_bad| not_found())?;
            let level = documents
                .0
                .get(&who, &doc, &trace)
                .await
                .map_err(hidden)?
                .my_access;
            (level, find_chain(pool, KIND_DOCUMENT, id).await?)
        },
        KIND_FOLDER => {
            let folder =
                DocumentFolderId::try_new(raw_id.to_owned()).map_err(|_bad| not_found())?;
            let access = documents
                .0
                .folder_access_of(&who, &folder, false, &trace)
                .await
                .map_err(hidden)?;
            (access.my_access, find_chain(pool, KIND_FOLDER, id).await?)
        },
        _ => return Err(not_found()),
    };
    Ok((id, level, chain.ok_or_else(not_found)?))
}

pub(crate) async fn get_class_handler(
    State(pool): State<Arc<PgPool>>,
    deps: ClassDeps,
    Extension(user): Extension<UserContext>,
    Path((kind, raw_id)): Path<(String, String)>,
) -> AdminResult<Json<ClassView>> {
    let (_id, level, chain) =
        access_and_chain(&pool, &deps.documents, &user, &kind, &raw_id).await?;
    Ok(Json(view(classes(&deps.declared)?, &chain, level, &user)?))
}

pub(crate) async fn set_class_handler(
    State(pool): State<Arc<PgPool>>,
    deps: ClassDeps,
    Extension(user): Extension<UserContext>,
    Path((kind, raw_id)): Path<(String, String)>,
    Json(body): Json<SetClass>,
) -> AdminResult<Json<ClassView>> {
    if current_actor().is_agent() {
        return Err(AdminError::Forbidden(
            "Only a person in a browser session can change a document's class.".to_owned(),
        ));
    }
    let (id, level, chain) =
        access_and_chain(&pool, &deps.documents, &user, &kind, &raw_id).await?;
    if level < AccessLevel::Own && !user.is_admin {
        return Err(AdminError::Forbidden(
            "Only the owner can change the class.".to_owned(),
        ));
    }
    let reason = body
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|r| !r.is_empty());
    let catalog = classes(&deps.declared)?;
    let change = check_change(
        catalog,
        &chain,
        &ChangeRequest {
            requested: &body.class_id,
            is_admin: user.is_admin,
            reason,
        },
    )
    .map_err(|refusal| AdminError::invalid("document class", refusal))?;
    record_class(
        &pool,
        &NewClass {
            target_kind: &kind,
            target_id: id,
            class_id: &body.class_id,
            set_by: user.user_id.as_str(),
            reason,
            loosened: change == ChangeKind::AdminLoosen,
        },
    )
    .await?;
    let updated = ClassChain {
        own: Some(body.class_id),
        ..chain
    };
    Ok(Json(view(catalog, &updated, level, &user)?))
}
