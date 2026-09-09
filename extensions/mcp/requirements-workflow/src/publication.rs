//! Publish only the exact reviewed content, reconciling interrupted external
//! writes.
use crate::model::{Destination, Gate, Phase, Publication, digest};
use crate::signing;
use crate::store::{self, StoredWorkflow};
use anyhow::{Result, ensure};
// JSON: Atlassian MCP tool arguments and replies cross the wire as untyped
// documents whose shape the connected server defines.
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

async fn connect(ctx: &RequestContext, bearer: &str) -> Result<AtlassianMcp> {
    let config = KnowledgeBankConfig::load(&KnowledgeBankConfig::resolve_path())?;
    Ok(AtlassianMcp::connect(bearer, ctx, &config.atlassian).await?)
}

fn body_for(stored: &StoredWorkflow, path: &str, content: &str) -> Result<String> {
    let receipt = stored.workflow.require_approved(Gate::Publication)?;
    Ok(format!(
        "workflow-generated-output\n\n{content}\n\n## Approval receipt\nArtifact: {path}\n\n```json\n{}\n```\n",
        serde_json::to_string_pretty(receipt)?
    ))
}

async fn verify_receipt(pool: &PgPool, stored: &StoredWorkflow) -> Result<()> {
    let receipt = stored.workflow.require_approved(Gate::Publication)?;
    let public_key = sqlx::query_scalar!(
        "SELECT public_key FROM requirements_workflow_keys WHERE key_id=$1",
        receipt.key_id
    )
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

struct Draft {
    title: String,
    body: String,
    hash: String,
}

async fn find_page_id(
    client: &AtlassianMcp,
    destination: &Destination,
    title: &str,
) -> Result<Option<String>> {
    let existing = client
        .call(
            "searchConfluence",
            json!({"cloudId":client.cloud_id(),"cql":format!("type=page AND space={} AND title={}", serde_json::to_string(&destination.space_id)?, serde_json::to_string(title)?),"limit":100}),
        )
        .await?;
    let matches: Vec<_> = existing["results"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|p| {
            p.pointer("/content/title")
                .or_else(|| p.get("title"))
                .and_then(Value::as_str)
                == Some(title)
        })
        .collect();
    ensure!(
        matches.len() <= 1,
        "Multiple publication pages match; reconcile duplicate pages before continuing"
    );
    matches
        .first()
        .map(|page| {
            page.pointer("/content/id")
                .or_else(|| page.get("id"))
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| anyhow::anyhow!("Page has no id"))
        })
        .transpose()
}

async fn create_page(
    pool: &PgPool,
    client: &AtlassianMcp,
    destination: &Destination,
    workflow_id: &str,
    draft: &Draft,
) -> Result<String> {
    let reserved = sqlx::query!(
        "INSERT INTO requirements_publication_attempts(workflow_id,title,content_hash,status) VALUES($1,$2,$3,'started') ON CONFLICT DO NOTHING",
        workflow_id,
        draft.title,
        draft.hash
    )
    .execute(pool)
    .await?;
    ensure!(
        reserved.rows_affected() == 1,
        "Previous publication has an uncertain outcome. Reconcile it in Confluence before retrying; no duplicate create was sent."
    );
    let created = client
        .call(
            "createConfluenceContent",
            json!({"cloudId":client.cloud_id(),"parent":{"spaceId":destination.space_id,"parentContentId":destination.parent_id},"contentType":"page","title":draft.title,"body":{"format":"markdown","value":draft.body}}),
        )
        .await?;
    created["id"].as_str().map(str::to_owned).ok_or_else(|| {
        anyhow::anyhow!("Created page id missing; reconcile the publication attempt")
    })
}

async fn read_page_body(client: &AtlassianMcp, page_id: &str) -> Result<String> {
    let page = client
        .call(
            "getConfluenceContent",
            json!({"cloudId":client.cloud_id(),"content_id":page_id,"detail":"full","content_format":"markdown","include_metadata":true}),
        )
        .await?;
    page.pointer("/body/value")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("Cannot verify published page body"))
}

pub async fn publish(
    pool: &PgPool,
    ctx: &RequestContext,
    bearer: &str,
    stored: &mut StoredWorkflow,
) -> Result<Value> {
    verify_receipt(pool, stored).await?;
    let destination = stored
        .workflow
        .destination
        .clone()
        .ok_or_else(|| anyhow::anyhow!("Approved destination required"))?;
    destination.validate()?;
    let client = connect(ctx, bearer).await?;
    ensure!(
        client.cloud_id() == destination.cloud_id,
        "Connected Confluence site differs from the approved destination"
    );
    let tools = client.list_tools().await?;
    ensure!(
        tools.iter().any(|t| t.name == "createConfluenceContent"),
        "Connected Atlassian does not expose createConfluenceContent; inspect its advertised write contract before publishing"
    );
    let bundle = stored.workflow.bundle_hash(Gate::Publication)?;
    let workflow_id = stored.workflow.id.0.to_string();
    let files: Vec<_> = stored
        .workflow
        .artifacts
        .iter()
        .filter(|(p, _)| matches!(p, Phase::Specification))
        .flat_map(|(_, a)| a.files.clone())
        .collect();
    for (path, content) in files {
        let body = body_for(stored, &path, &content)?;
        let draft = Draft {
            title: format!("{} [{}]", destination.title, &bundle[..12]),
            hash: digest(&body)?,
            body,
        };
        let page_id = match find_page_id(&client, &destination, &draft.title).await? {
            Some(id) => id,
            None => create_page(pool, &client, &destination, &workflow_id, &draft).await?,
        };
        let actual = read_page_body(&client, &page_id).await?;
        ensure!(
            canonical_markdown(&actual) == canonical_markdown(&draft.body),
            "Published page differs from the approved body; publication is not verified"
        );
        sqlx::query!(
            "UPDATE requirements_publication_attempts SET status='confirmed',page_id=$3 WHERE workflow_id=$1 AND title=$2 AND content_hash=$4",
            workflow_id,
            draft.title,
            page_id,
            draft.hash
        )
        .execute(pool)
        .await?;
        stored.workflow.publications.insert(
            path,
            Publication {
                uri: format!("{}/wiki/pages/{page_id}", client.site_url()),
                page_id,
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
    let destination = stored
        .workflow
        .destination
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Approved destination required"))?;
    ensure!(
        client.cloud_id() == destination.cloud_id,
        "Connected Confluence site differs from the approved destination"
    );
    let bundle = stored.workflow.bundle_hash(Gate::Publication)?;
    for publication in stored.workflow.publications.values() {
        ensure!(
            publication.bundle_hash == bundle,
            "Publication approval is stale"
        );
        let body = read_page_body(&client, &publication.page_id).await?;
        ensure!(
            digest(&body.trim())? == publication.body_hash,
            "Published content was modified"
        );
    }
    Ok(json!({"verified":true,"publications":stored.workflow.publications}))
}
