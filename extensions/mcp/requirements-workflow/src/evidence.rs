//! Freeze selected caller-scoped sources; re-read Confluence through governed
//! MCP.

use crate::model::{Evidence, SourceSelection, digest};
use anyhow::{Result, ensure};
use serde_json::{Value, json};
use sqlx::PgPool;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_knowledge_bank::{AtlassianMcp, KnowledgeBankConfig, export};

pub(crate) async fn snapshot(
    pool: &PgPool,
    ctx: &RequestContext,
    bearer: &str,
    selected: &[SourceSelection],
) -> Result<Vec<Evidence>> {
    ensure!(
        !selected.is_empty() && selected.len() <= 40,
        "Select 1–40 relevant sources from Knowledge Bank"
    );
    for (i, source) in selected.iter().enumerate() {
        ensure!(
            matches!(source.source_type.as_str(), "jira" | "confluence")
                && !source.source_ref.is_empty(),
            "Select a Jira or Confluence source identity returned by Knowledge Bank"
        );
        ensure!(
            !selected[..i].contains(source),
            "Duplicate source selection"
        );
    }
    let client = if selected.iter().any(|s| s.source_type == "confluence") {
        let config = KnowledgeBankConfig::load(&KnowledgeBankConfig::resolve_path())?;
        Some(AtlassianMcp::connect(bearer, ctx, &config.atlassian).await?)
    } else {
        None
    };
    let mut after = String::new();
    let mut evidence = Vec::new();
    loop {
        let page = export::export_sources(pool, ctx.user_id(), &after, 100).await?;
        if page.is_empty() {
            break;
        }
        for doc in page {
            after = format!("{}:{}", doc.source_type, doc.source_ref);
            if !selected
                .iter()
                .any(|s| s.source_type == doc.source_type && s.source_ref == doc.source_ref)
            {
                continue;
            }
            let mut source = Evidence {
                source_ref: doc.source_ref,
                source_type: doc.source_type,
                title: doc.title,
                uri: doc.uri,
                body: doc.body,
                indexed_hash: doc.content_hash.clone(),
                retrieval_method: "knowledge_bank_snapshot".to_owned(),
                hash: doc.content_hash,
                version: doc.version,
            };
            if source.source_type == "confluence" {
                let client = client
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Confluence connection required"))?;
                refresh_confluence(&mut source, client).await?;
            }
            ensure!(
                !source.body.contains("workflow-generated-output"),
                "Generated workflow output cannot be used as source evidence"
            );
            evidence.push(source);
        }
    }
    ensure!(
        evidence.len() == selected.len(),
        "Selected evidence is missing, stale or inaccessible. Sync the selected sources and review access before retrying."
    );
    ensure!(
        evidence.iter().map(|e| e.body.len()).sum::<usize>() <= 120_000,
        "Selected evidence exceeds the 120 KB context budget. Narrow the scope explicitly; no source was silently truncated."
    );
    evidence.sort_by(|a, b| (&a.source_type, &a.source_ref).cmp(&(&b.source_type, &b.source_ref)));
    Ok(evidence)
}

async fn refresh_confluence(source: &mut Evidence, client: &AtlassianMcp) -> Result<()> {
    let page = client
        .call(
            "getConfluenceContent",
            json!({
                "cloudId":client.cloud_id(), "content_id":source.source_ref,
                "detail":"full", "content_format":"markdown", "include_metadata":true
            }),
        )
        .await?;
    page.pointer("/body/value")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("Source page has no readable Markdown body"))?
        .clone_into(&mut source.body);
    page.get("title")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("Source page title is missing"))?
        .clone_into(&mut source.title);
    source.version = page
        .pointer("/metadata/version")
        .or_else(|| page.get("version"))
        .filter(|v| !v.is_null())
        .map(Value::to_string)
        .ok_or_else(|| anyhow::anyhow!("Source page version is missing"))?;
    source.uri = format!("{}/wiki/pages/{}", client.site_url(), source.source_ref);
    source.hash = digest(&source.body)?;
    "knowledge_bank_selection_and_atlassian_mcp_read".clone_into(&mut source.retrieval_method);
    Ok(())
}
