//! Publish only the exact reviewed content, reconciling interrupted external
//! writes.
use crate::model::{Gate, Phase, Publication, digest};
use crate::signing;
use crate::store::{self, StoredWorkflow};
use anyhow::{Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::PgPool;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_knowledge_bank::{AtlassianMcp, KnowledgeBankConfig};

pub fn canonical_markdown(body: &str) -> String {
    let options = pulldown_cmark::Options::ENABLE_TABLES
        | pulldown_cmark::Options::ENABLE_STRIKETHROUGH
        | pulldown_cmark::Options::ENABLE_TASKLISTS;
    let mut html = String::new();
    pulldown_cmark::html::push_html(
        &mut html,
        pulldown_cmark::Parser::new_ext(body.trim(), options),
    );
    html
}

#[derive(Deserialize)]
struct Destination {
    space_id: String,
    parent_id: String,
}

async fn connect(ctx: &RequestContext, bearer: &str) -> Result<AtlassianMcp> {
    let config = KnowledgeBankConfig::load(&KnowledgeBankConfig::resolve_path())?;
    Ok(AtlassianMcp::connect(bearer, ctx, &config.atlassian).await?)
}

fn body_for(stored: &StoredWorkflow, path: &str, content: &str) -> Result<String> {
    let receipt = stored.workflow.require_approved(Gate::Publication)?;
    Ok(format!(
        "[DEMO] Fictional migration project. workflow-generated-output\n\n{content}\n\n## Approval receipt\nArtifact: {path}\n\n```json\n{}\n```\n",
        serde_json::to_string_pretty(receipt)?
    ))
}

async fn verify_receipt(pool: &PgPool, stored: &StoredWorkflow) -> Result<()> {
    let receipt = stored.workflow.require_approved(Gate::Publication)?;
    let public_key: String =
        sqlx::query_scalar("SELECT public_key FROM requirements_workflow_keys WHERE key_id=$1")
            .bind(&receipt.key_id)
            .fetch_one(pool)
            .await?;
    signing::verify(receipt, &public_key)?;
    let payload: Value = serde_json::from_str(&receipt.payload)?;
    ensure!(
        payload["workflow_id"] == stored.workflow.id.0.to_string()
            && payload["gate"] == "publication"
            && payload["decision"] == "approved"
            && payload["bundle_hash"] == stored.workflow.bundle_hash(Gate::Publication)?,
        "Approval receipt does not bind this publication bundle"
    );
    Ok(())
}

pub async fn publish(
    _server: &crate::server::WorkflowServer,
    pool: &PgPool,
    ctx: &RequestContext,
    bearer: &str,
    stored: &mut StoredWorkflow,
) -> Result<Value> {
    verify_receipt(pool, stored).await?;
    let destination: Destination = serde_json::from_slice(
        &tokio::fs::read("services/requirements-workflow/destination.json").await?,
    )?;
    let client = connect(ctx, bearer).await?;
    let tools = client.list_tools().await?;
    ensure!(
        tools.iter().any(|t| t.name == "createConfluenceContent"),
        "Connected Atlassian does not expose createConfluenceContent; inspect its advertised write contract before publishing"
    );
    let bundle = stored.workflow.bundle_hash(Gate::Publication)?;
    let files: Vec<_> = stored
        .workflow
        .artifacts
        .iter()
        .filter(|(p, _)| matches!(p, Phase::Discovery | Phase::Fsd))
        .flat_map(|(_, a)| a.files.clone())
        .collect();
    for (path, content) in files {
        let title = format!("[DEMO] {} {} {}", stored.workflow.name, path, &bundle[..12]);
        let body = body_for(stored, &path, &content)?;
        let hash = digest(&body)?;
        let existing=client.call("searchConfluence",json!({"cloudId":client.cloud_id(),"cql":format!("type=page AND space=EAICOMM AND title=\"{title}\""),"limit":100})).await?;
        let matches: Vec<_> = existing["results"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|p| {
                p.pointer("/content/title")
                    .or_else(|| p.get("title"))
                    .and_then(Value::as_str)
                    == Some(&title)
            })
            .collect();
        ensure!(
            matches.len() <= 1,
            "Multiple publication pages match; reconcile duplicate pages before continuing"
        );
        let page_id = if let Some(page) = matches.first() {
            page.pointer("/content/id")
                .or_else(|| page.get("id"))
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("Page has no id"))?
                .to_owned()
        } else {
            let reserved=sqlx::query("INSERT INTO requirements_publication_attempts(workflow_id,title,content_hash,status) VALUES($1,$2,$3,'started') ON CONFLICT DO NOTHING")
                .bind(stored.workflow.id.0.to_string()).bind(&title).bind(&hash).execute(pool).await?;
            ensure!(
                reserved.rows_affected() == 1,
                "Previous publication has an uncertain outcome. Reconcile it in Confluence before retrying; no duplicate create was sent."
            );
            let created=client.call("createConfluenceContent",json!({"cloudId":client.cloud_id(),"parent":{"spaceId":destination.space_id,"parentContentId":destination.parent_id},"contentType":"page","title":title,"body":{"format":"markdown","value":body}})).await?;
            created["id"]
                .as_str()
                .ok_or_else(|| {
                    anyhow::anyhow!("Created page id missing; reconcile the publication attempt")
                })?
                .to_owned()
        };
        let page=client.call("getConfluenceContent",json!({"cloudId":client.cloud_id(),"content_id":page_id,"detail":"full","content_format":"markdown","include_metadata":true})).await?;
        let actual = page
            .pointer("/body/value")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Cannot verify published page body"))?;
        ensure!(
            canonical_markdown(actual) == canonical_markdown(&body),
            "Published page differs from the approved body; publication is not verified"
        );
        sqlx::query("UPDATE requirements_publication_attempts SET status='confirmed',page_id=$3 WHERE workflow_id=$1 AND title=$2 AND content_hash=$4")
            .bind(stored.workflow.id.0.to_string()).bind(&title).bind(&page_id).bind(&hash).execute(pool).await?;
        stored.workflow.publications.insert(
            path,
            Publication {
                page_id: page_id.clone(),
                uri: format!("{}/wiki/spaces/EAICOMM/pages/{page_id}", client.site_url()),
                body_hash: digest(&actual.trim())?,
                bundle_hash: bundle.clone(),
            },
        );
        stored.revision = store::save(
            pool,
            ctx.user_id(),
            stored.revision,
            stored,
            "publication_confirmed",
        )
        .await?;
    }
    Ok(serde_json::to_value(stored)?)
}

pub async fn verify(
    _server: &crate::server::WorkflowServer,
    pool: &PgPool,
    ctx: &RequestContext,
    bearer: &str,
    stored: &StoredWorkflow,
) -> Result<Value> {
    verify_receipt(pool, stored).await?;
    ensure!(
        !stored.workflow.publications.is_empty(),
        "Nothing has been published"
    );
    let client = connect(ctx, bearer).await?;
    let bundle = stored.workflow.bundle_hash(Gate::Publication)?;
    for publication in stored.workflow.publications.values() {
        ensure!(
            publication.bundle_hash == bundle,
            "Publication approval is stale"
        );
        let page=client.call("getConfluenceContent",json!({"cloudId":client.cloud_id(),"content_id":publication.page_id,"detail":"full","content_format":"markdown"})).await?;
        let body = page
            .pointer("/body/value")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing page content"))?;
        ensure!(
            digest(&body.trim())? == publication.body_hash,
            "Published content was modified"
        );
    }
    Ok(json!({"verified":true,"publications":stored.workflow.publications}))
}
