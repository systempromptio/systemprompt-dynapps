//! Browser-only review of immutable requirements bundles.
use crate::error::{AdminError, AdminResult};
use crate::types::UserContext;
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Extension, Form};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::config::{ProfileBootstrap, SecretsBootstrap};
use systemprompt_requirements_workflow::model::{Gate, WorkflowId};
use systemprompt_requirements_workflow::signing::ReceiptSigner;
use systemprompt_requirements_workflow::store;
use uuid::Uuid;

fn failure(e: impl std::fmt::Display) -> AdminError {
    AdminError::Conflict(e.to_string())
}
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn browser_session<'a>(headers: &HeaderMap, user: &'a UserContext) -> AdminResult<&'a str> {
    if headers.contains_key("authorization") || !headers.contains_key("cookie") {
        return Err(AdminError::Forbidden(
            "Use the signed-in browser review screen".to_owned(),
        ));
    }
    user.session_id
        .as_ref()
        .map(|s| s.as_str())
        .ok_or_else(|| AdminError::Forbidden("An interactive session is required".to_owned()))
}

pub(crate) async fn review(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(raw): Path<String>,
    headers: HeaderMap,
) -> AdminResult<Html<String>> {
    let session = browser_session(&headers, &user)?;
    let id = WorkflowId(raw.parse().map_err(failure)?);
    let stored = store::get(&pool, &user.user_id, id)
        .await
        .map_err(failure)?;
    let nonce = Uuid::new_v4();
    sqlx::query("INSERT INTO requirements_workflow_review_nonces(nonce,workflow_id,revision,user_id,session_id) VALUES($1,$2,$3,$4,$5)")
        .bind(nonce.to_string()).bind(raw.clone()).bind(stored.revision).bind(user.user_id.as_str()).bind(session).execute(&*pool).await.map_err(failure)?;
    let mut body = format!(
        "<!doctype html><html lang=en><meta charset=utf-8><meta name=viewport content='width=device-width,initial-scale=1'><title>Requirements review</title><link rel=stylesheet href='/files/css/requirements-review.css'><main><h1>{}</h1><p>Review revision {}. Your decision binds this exact evidence and artifact bundle.</p>",
        escape(&stored.workflow.name),
        stored.revision
    );
    body.push_str(&format!("<p><a href='/admin/requirements/{raw}/download'>Download evidence, artifacts and validation receipts (JSON)</a></p>"));
    let previous: Option<serde_json::Value> = sqlx::query_scalar("SELECT state FROM requirements_workflow_events WHERE workflow_id=$1 AND revision<$2 ORDER BY revision DESC LIMIT 1")
        .bind(&raw).bind(stored.revision).fetch_optional(&*pool).await.map_err(failure)?;
    if let Some(previous) = previous {
        let previous: systemprompt_requirements_workflow::model::Workflow =
            serde_json::from_value(previous).map_err(failure)?;
        body.push_str("<details><summary>Changes since the previous revision</summary><ul>");
        for phase in systemprompt_requirements_workflow::model::Phase::ALL {
            let before = previous.artifacts.get(&phase);
            let after = stored.workflow.artifacts.get(&phase);
            if before.map(|a| &a.hash) != after.map(|a| &a.hash) {
                body.push_str(&format!(
                    "<li>{}: {}</li>",
                    phase.name(),
                    if after.is_some() {
                        "content changed"
                    } else {
                        "invalidated and removed"
                    }
                ));
                if let Some(before) = before {
                    for (path, content) in &before.files {
                        body.push_str(&format!(
                            "<li>Previous {}<pre>{}</pre></li>",
                            escape(path),
                            escape(content)
                        ));
                    }
                }
            }
        }
        body.push_str("</ul></details>");
    }
    for (phase, artifact) in &stored.workflow.artifacts {
        body.push_str(&format!("<section><h2>{}</h2>", phase.name()));
        for (path, content) in &artifact.files {
            body.push_str(&format!(
                "<h3>{}</h3><pre>{}</pre>",
                escape(path),
                escape(content)
            ));
        }
        if let Some(validation) = &artifact.validation {
            body.push_str(&format!(
                "<details><summary>Validation: {}</summary><pre>{}</pre></details>",
                if validation.passed {
                    "passed"
                } else {
                    "failed"
                },
                escape(&validation.report)
            ));
        }
        body.push_str("</section>");
    }
    body.push_str("<details><summary>Source evidence</summary>");
    for source in &stored.workflow.evidence {
        body.push_str(&format!(
            "<h3>{}</h3><p>{}</p><pre>{}</pre>",
            escape(&source.title),
            escape(&source.uri),
            escape(&source.body)
        ));
    }
    body.push_str("</details>");
    for (gate, review) in [
        ("grooming", &stored.workflow.grooming_review),
        ("publication", &stored.workflow.publication_review),
    ] {
        if let Some(review) = review {
            if review.receipt.is_none() && review.rejection.is_none() {
                body.push_str(&format!("<form method=post action='/admin/requirements/{raw}/decision'><h2>{gate} approval</h2><input type=hidden name=nonce value='{nonce}'><input type=hidden name=revision value='{}'><input type=hidden name=gate value='{gate}'><label>Review note <textarea name=note maxlength=2000 required></textarea></label><button name=decision value=approve>Approve this exact bundle</button><button name=decision value=reject>Request changes</button></form>",stored.revision));
            } else {
                body.push_str(&format!(
                    "<p>{gate}: {}</p>",
                    if review.receipt.is_some() {
                        "approved"
                    } else {
                        "changes requested"
                    }
                ));
            }
        }
    }
    body.push_str("</main></html>");
    Ok(Html(body))
}

pub(crate) async fn download(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(raw): Path<String>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    browser_session(&headers, &user)?;
    let stored = store::get(
        &pool,
        &user.user_id,
        WorkflowId(raw.parse().map_err(failure)?),
    )
    .await
    .map_err(failure)?;
    let key_ids: Vec<String> = [
        &stored.workflow.grooming_review,
        &stored.workflow.publication_review,
    ]
    .into_iter()
    .filter_map(|r| r.as_ref()?.receipt.as_ref().map(|r| r.key_id.clone()))
    .collect();
    let keys: Vec<(String, String)> = sqlx::query_as(
        "SELECT key_id,public_key FROM requirements_workflow_keys WHERE key_id=ANY($1)",
    )
    .bind(&key_ids)
    .fetch_all(&*pool)
    .await
    .map_err(failure)?;
    let keys: std::collections::BTreeMap<_, _> = keys.into_iter().collect();
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

pub(crate) async fn decide(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(raw): Path<String>,
    headers: HeaderMap,
    Form(form): Form<DecisionForm>,
) -> AdminResult<Redirect> {
    let session = browser_session(&headers, &user)?;
    let profile = ProfileBootstrap::get().map_err(failure)?;
    let expected = url::Url::parse(&profile.server.api_external_url)
        .map_err(failure)?
        .origin()
        .ascii_serialization();
    if headers.get("origin").and_then(|v| v.to_str().ok()) != Some(expected.as_str()) {
        return Err(AdminError::Forbidden(
            "Same-origin browser review is required".to_owned(),
        ));
    }
    if !matches!(form.decision.as_str(), "approve" | "reject")
        || form.note.trim().is_empty()
        || form.note.len() > 2000
    {
        return Err(failure("A decision and review note are required"));
    }
    let id = WorkflowId(raw.parse().map_err(failure)?);
    let mut stored = store::get(&pool, &user.user_id, id)
        .await
        .map_err(failure)?;
    if stored.revision != form.revision {
        return Err(failure("The bundle changed; review the latest version"));
    }
    let project = stored.project_id.clone();
    let evidence_hash = stored.workflow.evidence_hash.clone();
    let rubric = stored.workflow.rubric_commit.clone();
    let artifact_hashes: std::collections::BTreeMap<_, _> = stored
        .workflow
        .artifacts
        .iter()
        .map(|(phase, artifact)| (phase.name(), artifact.hash.clone()))
        .collect();
    let review = stored.workflow.review_mut(form.gate).map_err(failure)?;
    let mut tx = pool.begin().await.map_err(failure)?;
    let consumed=sqlx::query("DELETE FROM requirements_workflow_review_nonces WHERE nonce=$1 AND workflow_id=$2 AND revision=$3 AND user_id=$4 AND session_id=$5 AND expires_at>NOW()")
        .bind(form.nonce.to_string()).bind(&raw).bind(form.revision).bind(user.user_id.as_str()).bind(session).execute(&mut *tx).await.map_err(failure)?;
    if consumed.rows_affected() != 1 {
        return Err(failure("Review form expired or was already used"));
    }
    if form.decision == "approve" {
        let secrets = SecretsBootstrap::require().map_err(failure)?;
        let seed = secrets
            .get("requirements_workflow_signing_seed")
            .ok_or_else(|| failure("Workflow signing key has not been provisioned"))?;
        let signer = ReceiptSigner::from_seed_base64(seed).map_err(failure)?;
        let receipt=signer.sign(&json!({"schema":"astound.requirements.approval.v1","workflow_id":id,"project":project,"gate":form.gate,"bundle_hash":review.bundle_hash,"evidence_hash":evidence_hash,"rubric_commit":rubric,"artifact_hashes":artifact_hashes,"reviewer_id":user.user_id,"reviewer_name":user.username,"session_id":session,"decision":"approved","decided_at":chrono::Utc::now(),"note":form.note})).map_err(failure)?;
        sqlx::query("INSERT INTO requirements_workflow_keys(key_id,public_key) VALUES($1,$2) ON CONFLICT DO NOTHING").bind(&receipt.key_id).bind(signer.public_key()).execute(&mut *tx).await.map_err(failure)?;
        review.receipt = Some(receipt);
    } else {
        review.rejection = Some(form.note);
    }
    let state = serde_json::to_value(&stored.workflow).map_err(failure)?;
    let changed=sqlx::query("UPDATE requirements_workflows SET state=$1,revision=revision+1,updated_at=NOW() WHERE id=$2 AND revision=$3")
        .bind(&state).bind(&raw).bind(form.revision).execute(&mut *tx).await.map_err(failure)?;
    if changed.rows_affected() != 1 {
        return Err(failure("Concurrent edit; reload and review"));
    }
    sqlx::query("INSERT INTO requirements_workflow_events(workflow_id,revision,actor_id,event,state) VALUES($1,$2,$3,$4,$5)")
        .bind(&raw).bind(form.revision+1).bind(user.user_id.as_str()).bind(format!("human_{}",form.decision)).bind(state).execute(&mut *tx).await.map_err(failure)?;
    tx.commit().await.map_err(failure)?;
    Ok(Redirect::to(&format!("/admin/requirements/{raw}")))
}
