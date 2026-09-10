//! Authenticated report handlers dispatched through the existing MCP executor.

use super::atlassian::{Atlassian, identifier, rows};
use super::shape::table;
use super::{CatalogInput, ReportInput, ReportOutput};
use crate::cli::CliLocation;
use rmcp::ErrorData;
use serde_json::json;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::execution::context::RequestContext;

#[derive(Clone, Copy)]
pub struct ReportHandler<'a> {
    pub cli: &'a CliLocation,
    pub token: &'a str,
}

impl McpToolHandler for ReportHandler<'_> {
    type Input = ReportInput;
    type Output = ReportOutput;
    fn tool_name(&self) -> &'static str {
        "admin_report"
    }
    fn description(&self) -> &'static str {
        "Read a deterministic admin dashboard: the daily brief with a traffic light per project, one project's delivery detail, or AI usage and cost. Returns structured facts, source coverage and an interactive artifact. Never performs administrative changes."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: ReportInput,
        context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(ReportOutput, String), ErrorData> {
        // Why: the two Atlassian-backed reports connect as the caller's own
        // linked account; the cost report reads the CLI. Splitting on the input
        // rather than on a flag keeps that choice impossible to get wrong.
        let output = match input {
            ReportInput::Projects { .. } => {
                let client = Atlassian::connect(self.token, context).await?;
                super::projects::report(&client, &input).await?
            },
            ReportInput::Brief { .. } => {
                let client = Atlassian::connect(self.token, context).await?;
                super::brief::report(&client, &input).await?
            },
            ReportInput::Costs { .. } => super::platform::run(&input, self.cli, self.token).await?,
        };
        let summary = format!(
            "{} — {}. {}",
            output.title,
            output.checked_at,
            if output.complete {
                "Sources loaded"
            } else {
                "Partial data; inspect source warnings"
            }
        );
        Ok((output, summary))
    }
}

#[derive(Clone, Copy)]
pub struct CatalogHandler<'a> {
    pub token: &'a str,
}

impl McpToolHandler for CatalogHandler<'_> {
    type Input = CatalogInput;
    type Output = ReportOutput;
    fn tool_name(&self) -> &'static str {
        "admin_project_catalog"
    }
    fn description(&self) -> &'static str {
        "List your accessible Atlassian sites, or projects and Confluence spaces within a selected cloud_id. Read-only; uses your linked account."
    }
    fn read_only(&self) -> bool {
        true
    }
    async fn handle(
        &self,
        input: CatalogInput,
        context: &RequestContext,
        _execution: &McpExecutionId,
    ) -> Result<(ReportOutput, String), ErrorData> {
        if let Some(cloud) = &input.cloud_id {
            identifier(cloud)?;
        }
        let client = Atlassian::connect(self.token, context).await?;
        let mut output = ReportOutput::new(
            "catalog",
            "Atlassian Project Catalog",
            "Current accessible resources".into(),
        );
        if let Some(cloud) = input.cloud_id {
            for (title, names, keys) in [
                (
                    "Projects",
                    vec!["listJiraProjects", "getVisibleJiraProjects"],
                    vec!["values", "projects", "items"],
                ),
                (
                    "Spaces",
                    vec!["listConfluenceSpaces", "getConfluenceSpaces"],
                    vec!["results", "spaces", "items"],
                ),
            ] {
                match client
                    .read(
                        &names,
                        json!({"cloudId":cloud,"maxResults":100,"limit":100}),
                    )
                    .await
                    .and_then(|value| rows(&value, &keys))
                {
                    Ok(values) => {
                        output.source(
                            title,
                            values.len() < 100,
                            Some(
                                "First 100 accessible entries; enter an explicit ID if omitted."
                                    .into(),
                            ),
                        );
                        output.tables.push(table(
                            title,
                            values
                                .into_iter()
                                .map(|v| {
                                    json!({
                                        "id":v["id"],"key":v["key"],"name":v["name"]
                                    })
                                })
                                .collect(),
                        ));
                    },
                    Err(e) => output.source(title, false, Some(e.message.to_string())),
                }
            }
        } else {
            let data = client
                .read(&["getAccessibleAtlassianResources"], json!({}))
                .await?;
            output
                .tables
                .push(table("Sites", rows(&data, &["resources", "items"])?));
            output.source("Atlassian sites", true, None);
        }
        Ok((output, "Accessible Atlassian resources".into()))
    }
}

impl std::fmt::Debug for ReportHandler<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReportHandler")
            .field("cli", self.cli)
            .finish_non_exhaustive()
    }
}
impl std::fmt::Debug for CatalogHandler<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CatalogHandler")
            .finish_non_exhaustive()
    }
}
