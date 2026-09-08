//! Freeze complete knowledge-bank source evidence for requirements review.

use crate::model::Evidence;
use anyhow::{Result, ensure};
use sqlx::PgPool;
use systemprompt_mcp_knowledge_bank::export;

pub(crate) async fn snapshot(
    pool: &PgPool,
    user: &systemprompt::identifiers::UserId,
) -> Result<Vec<Evidence>> {
    let run = export::latest_run(pool, user)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Run a complete Jira and Confluence sync first"))?;
    ensure!(
        run["status"].as_str() == Some("complete"),
        "The latest sync is not complete"
    );
    let sources = run["sources"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Missing sync source inventory"))?;
    ensure!(
        sources.is_empty()
            || ["jira", "confluence"]
                .iter()
                .all(|name| sources.iter().any(|s| s.as_str() == Some(name))),
        "Sync both Jira and Confluence before freezing evidence"
    );
    let states = export::list_sync_state(pool, user).await?;
    for source in ["jira", "confluence"] {
        let state = states
            .iter()
            .find(|s| s.source_type == source)
            .ok_or_else(|| anyhow::anyhow!("Sync {source} first"))?;
        ensure!(
            state.last_run.is_some() && state.error.is_none() && state.documents_failed == 0,
            "{source} sync is incomplete"
        );
    }
    let mut after = String::new();
    let mut evidence = Vec::new();
    loop {
        let page = export::export_sources(pool, user, &after, 100).await?;
        if page.is_empty() {
            break;
        }
        for doc in page {
            after = format!("{}:{}", doc.source_type, doc.source_ref);
            if (doc.title.starts_with("[DEMO]") || doc.title.contains("] [DEMO]"))
                && doc.body.contains("demo-spa-migration")
                && !doc.body.contains("workflow-generated-output")
            {
                evidence.push(Evidence {
                    source_ref: doc.source_ref,
                    source_type: doc.source_type,
                    title: doc.title,
                    uri: doc.uri,
                    body: doc.body,
                    hash: doc.content_hash,
                    version: doc.version,
                });
            }
        }
    }
    ensure!(
        evidence.iter().filter(|d| d.source_type == "jira").count() == 60
            && evidence
                .iter()
                .filter(|d| d.source_type == "confluence")
                .count()
                == 20,
        "Expected 60 demo issues and 20 source pages. Complete the seed manifest and rebuild Knowledge Bank before starting."
    );
    evidence.sort_by(|a, b| (&a.source_type, &a.source_ref).cmp(&(&b.source_type, &b.source_ref)));
    Ok(evidence)
}
