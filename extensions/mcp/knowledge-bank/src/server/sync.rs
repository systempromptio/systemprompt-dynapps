//! `sync_knowledge`: start the caller's run in the background. Open to every
//! signed-in user. Files categories need nothing from the caller; Confluence
//! and Jira are read through the Atlassian account the caller connected on
//! `/admin/profile`, reached via this instance's own MCP gateway with the
//! caller's bearer. The connection is attempted *before* the run starts:
//! an unscoped sync that cannot reach Atlassian degrades to the files
//! categories and says why, a sync that names an Atlassian category
//! explicitly is refused with the same reason.

use super::format::{AtlassianReach, format_sync_started};
use super::tool::to_mcp_error;
use crate::error::KnowledgeBankError;
use crate::ingest::sources::AtlassianMcp;
use crate::ingest::{Ingestor, RunOptions};
use crate::principal::Principal;
use crate::tools::{SYNC_TOOL, SyncKnowledgeInput};
use rmcp::ErrorData as McpError;
use std::sync::Arc;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::CliArtifact;
use systemprompt::models::execution::context::RequestContext as SysRequestContext;

pub(super) struct SyncKnowledgeHandler<'a> {
    pub(super) ingestor: &'a Arc<Ingestor>,
    pub(super) bearer: &'a str,
}

impl McpToolHandler for SyncKnowledgeHandler<'_> {
    type Input = SyncKnowledgeInput;
    type Output = CliArtifact;

    fn tool_name(&self) -> &'static str {
        SYNC_TOOL
    }

    fn description(&self) -> &'static str {
        "Index what your connected Atlassian account can read."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &SysRequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let principal = Principal::from_request_context(ctx);
        start_sync(self.ingestor, self.bearer, ctx, &principal, input)
            .await
            .map_err(|e| to_mcp_error(&e))
    }
}

pub(super) async fn start_sync(
    ingestor: &Arc<Ingestor>,
    bearer: &str,
    ctx: &SysRequestContext,
    principal: &Principal,
    input: SyncKnowledgeInput,
) -> Result<(CliArtifact, String), KnowledgeBankError> {
    let config = ingestor.config();
    let mut sources = input.source_types.clone().unwrap_or_default();
    for id in &sources {
        if !config.sources.contains_key(id) {
            return Err(KnowledgeBankError::UnknownSource(id.clone()));
        }
    }
    if let Some(result) = queue_drive(ingestor, principal, &input, &mut sources).await? {
        return Ok(result);
    }
    if ingestor.is_running_for(&principal.user_id) {
        return Err(KnowledgeBankError::Backend(
            "your knowledge sync is already in progress".to_owned(),
        ));
    }
    let (atlassian, reach) = if config.requires_atlassian(&sources) {
        match AtlassianMcp::connect(bearer, ctx, &config.atlassian).await {
            Ok(connected) => (Some(connected), AtlassianReach::Connected),
            Err(e) if sources.is_empty() && !config.files_source_ids().is_empty() => {
                let reason = e.to_string();
                tracing::info!(user_id = %principal.user_id, reason = %reason, "Atlassian unreachable; syncing files categories only");
                ingestor
                    .record_skipped(&principal.user_id, &config.atlassian_source_ids(), &reason)
                    .await;
                sources = config.files_source_ids();
                (None, AtlassianReach::Skipped(reason))
            },
            Err(e) => return Err(e),
        }
    } else {
        (None, AtlassianReach::NotNeeded)
    };
    let options = RunOptions {
        rebuild: input.rebuild,
        sources: sources.clone(),
    };
    let runner = Arc::clone(ingestor);
    let caller = principal.clone();
    tracing::info!(user_id = %principal.user_id, rebuild = input.rebuild, atlassian = ?reach, "knowledge sync requested");
    let run_id = ingestor
        .start_run_record(&principal.user_id, &sources)
        .await?;
    let recorded_run = run_id.clone();
    tokio::spawn(async move {
        let result = runner.run(&caller, atlassian, &options).await;
        let (successful, detail) = match &result {
            Ok(report) => (!report.has_errors(), format!("{report:?}")),
            Err(error) => (false, error.to_string()),
        };
        if let Err(error) = runner
            .finish_run_record(&recorded_run, successful, &detail)
            .await
        {
            tracing::error!(error = %error, run_id = %recorded_run, "could not persist sync outcome");
        }
        match result {
            Ok(report) => tracing::info!(
                user_id = %caller.user_id,
                duration_ms = report.duration_ms,
                errors = report.has_errors(),
                "requested knowledge sync finished"
            ),
            Err(e) => {
                tracing::error!(user_id = %caller.user_id, error = %e, "requested knowledge sync failed");
            },
        }
    });
    let (artifact, summary) = format_sync_started(&sources, input.rebuild, &reach);
    Ok((
        artifact,
        format!("{summary}; run_id={run_id}; poll knowledge_sync_status"),
    ))
}

async fn queue_drive(
    ingestor: &Ingestor,
    principal: &Principal,
    input: &SyncKnowledgeInput,
    sources: &mut Vec<String>,
) -> Result<Option<(CliArtifact, String)>, KnowledgeBankError> {
    use crate::drive::store;
    let config = ingestor.config();
    let drive_sources: Vec<String> = config
        .sources
        .iter()
        .filter(|(id, source)| {
            source.kind == crate::config::SourceKind::GoogleDrive
                && (sources.is_empty() || sources.contains(id))
        })
        .map(|(id, _)| id.clone())
        .collect();
    if !drive_sources.is_empty() {
        if config.google_drive.enabled {
            for id in &drive_sources {
                let drive = config.google_drive.sources.get(id).ok_or_else(|| {
                    KnowledgeBankError::Config(format!("Missing Drive settings for {id}"))
                })?;
                store::enqueue(
                    ingestor.pool(),
                    id,
                    &drive.drive_id,
                    principal.user_id.as_str(),
                    store::SyncRequest {
                        force: input.force_resync,
                        rebuild: input.rebuild,
                    },
                )
                .await?;
            }
        } else if !sources.is_empty() {
            return Err(KnowledgeBankError::NotConfigured("Google Drive sync is disabled; configure service credentials and enable google_drive".to_owned()));
        }
        if !sources.is_empty() && sources.iter().all(|id| drive_sources.contains(id)) {
            let summary = format!(
                "Queued Drive sync for {}; poll knowledge_sync_status and knowledge_drive_inventory",
                drive_sources.join(", ")
            );
            return Ok(Some((
                CliArtifact::text(systemprompt::models::artifacts::TextArtifact::new(&summary)),
                summary,
            )));
        }
        sources.retain(|id| !drive_sources.contains(id));
    }
    Ok(None)
}
