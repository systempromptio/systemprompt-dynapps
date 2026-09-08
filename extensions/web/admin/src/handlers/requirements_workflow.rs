//! Browser-only review of immutable requirements bundles.
use crate::error::{AdminError, AdminResult};
use crate::handlers::requirements_review_html::review_body;
use crate::repositories::requirements_workflow as repo;
use crate::types::UserContext;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Extension, Form};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use std::collections::BTreeMap;
use std::sync::Arc;
use systemprompt::config::{ProfileBootstrap, SecretsBootstrap};
use systemprompt::identifiers::SessionId;
use systemprompt_requirements_model::model::{Gate, Workflow, WorkflowId};
use systemprompt_requirements_model::signing::ReceiptSigner;
use uuid::Uuid;

fn failure(e: impl std::fmt::Display) -> AdminError {
    AdminError::Conflict(e.to_string())
}
fn browser_session<'a>(headers: &HeaderMap, user: &'a UserContext) -> AdminResult<&'a SessionId> {
    if headers.contains_key("authorization") || !headers.contains_key("cookie") {
        return Err(AdminError::Forbidden(
            "Use the signed-in browser review screen".to_owned(),
        ));
    }
    user.session_id
        .as_ref()
        .ok_or_else(|| AdminError::Forbidden("An interactive session is required".to_owned()))
}

fn parse_id(raw: &str) -> AdminResult<WorkflowId> {
    Ok(WorkflowId(raw.parse().map_err(failure)?))
}

pub(crate) async fn review(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(raw): Path<String>,
    headers: HeaderMap,
) -> AdminResult<Html<String>> {
    let session = browser_session(&headers, &user)?;
    let stored = repo::get_workflow(&pool, &user.user_id, parse_id(&raw)?).await?;
    let nonce = Uuid::new_v4();
    repo::insert_review_nonce(
        &pool,
        &repo::ReviewNonce {
            nonce: &nonce.to_string(),
            workflow_id: &raw,
            revision: stored.revision,
            user: &user.user_id,
            session_id: session,
        },
    )
    .await?;
    let previous = repo::find_previous_state(&pool, &raw, stored.revision)
        .await?
        .map(serde_json::from_value::<Workflow>)
        .transpose()
        .map_err(failure)?;
    Ok(Html(review_body(&raw, nonce, &stored, previous.as_ref())))
}

pub(crate) async fn download(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(raw): Path<String>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    browser_session(&headers, &user)?;
    let stored = repo::get_workflow(&pool, &user.user_id, parse_id(&raw)?).await?;
    let key_ids: Vec<String> = [
        &stored.workflow.grooming_review,
        &stored.workflow.publication_review,
    ]
    .into_iter()
    .filter_map(|r| r.as_ref()?.receipt.as_ref().map(|r| r.key_id.clone()))
    .collect();
    let keys: BTreeMap<String, String> = repo::list_verification_keys(&pool, &key_ids)
        .await?
        .into_iter()
        .map(|k| (k.key_id, k.public_key))
        .collect();
    let body = serde_json::to_string_pretty(&json!({"workflow":stored,"verification_keys":keys}))
        .map_err(failure)?;
    Ok((
        [
            ("content-type", "application/json"),
            (
                "content-disposition",
                "attachment; filename=requirements-bundle.json",
            ),
            ("cache-control", "no-store"),
        ],
        body,
    )
        .into_response())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DecisionForm {
    nonce: Uuid,
    revision: i64,
    gate: Gate,
    decision: String,
    note: String,
}

fn require_same_origin(headers: &HeaderMap) -> AdminResult<()> {
    let profile = ProfileBootstrap::get().map_err(failure)?;
    let expected = url::Url::parse(&profile.server.api_external_url)
        .map_err(failure)?
        .origin()
        .ascii_serialization();
    if headers.get("origin").and_then(|v| v.to_str().ok()) == Some(expected.as_str()) {
        return Ok(());
    }
    Err(AdminError::Forbidden(
        "Same-origin browser review is required".to_owned(),
    ))
}

pub(crate) async fn decide(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(raw): Path<String>,
    headers: HeaderMap,
    Form(form): Form<DecisionForm>,
) -> AdminResult<Redirect> {
    let session = browser_session(&headers, &user)?;
    require_same_origin(&headers)?;
    if !matches!(form.decision.as_str(), "approve" | "reject")
        || form.note.trim().is_empty()
        || form.note.len() > 2000
    {
        return Err(failure("A decision and review note are required"));
    }
    let id = parse_id(&raw)?;
    let mut stored = repo::get_workflow(&pool, &user.user_id, id).await?;
    if stored.revision != form.revision {
        return Err(failure("The bundle changed; review the latest version"));
    }
    let project = stored.project_id.clone();
    let evidence_hash = stored.workflow.evidence_hash.clone();
    let rubric = stored.workflow.rubric_commit.clone();
    let artifact_hashes: BTreeMap<_, _> = stored
        .workflow
        .artifacts
        .iter()
        .map(|(phase, artifact)| (phase.name(), artifact.hash.clone()))
        .collect();
    let review = stored.workflow.review_mut(form.gate).map_err(failure)?;
    let mut signing_key = None;
    if form.decision == "approve" {
        let secrets = SecretsBootstrap::require().map_err(failure)?;
        let seed = secrets
            .get("requirements_workflow_signing_seed")
            .ok_or_else(|| failure("Workflow signing key has not been provisioned"))?;
        let signer = ReceiptSigner::from_seed_base64(seed).map_err(failure)?;
        let receipt=signer.sign(&json!({"schema":"astound.requirements.approval.v1","workflow_id":id,"project":project,"gate":form.gate,"bundle_hash":review.bundle_hash,"evidence_hash":evidence_hash,"rubric_commit":rubric,"artifact_hashes":artifact_hashes,"reviewer_id":user.user_id,"reviewer_name":user.username,"session_id":session.as_str(),"decision":"approved","decided_at":chrono::Utc::now(),"note":form.note})).map_err(failure)?;
        signing_key = Some(repo::VerificationKey {
            key_id: receipt.key_id.clone(),
            public_key: signer.public_key(),
        });
        review.receipt = Some(receipt);
    } else {
        review.rejection = Some(form.note);
    }
    let state = serde_json::to_value(&stored.workflow).map_err(failure)?;
    let event = format!("human_{}", form.decision);
    repo::insert_decision(
        &pool,
        &repo::Decision {
            workflow_id: &raw,
            nonce: &form.nonce.to_string(),
            revision: form.revision,
            user: &user.user_id,
            session_id: session,
            event: &event,
            signing_key,
            state,
        },
    )
    .await?;
    Ok(Redirect::to(&format!("/admin/requirements/{raw}")))
}
