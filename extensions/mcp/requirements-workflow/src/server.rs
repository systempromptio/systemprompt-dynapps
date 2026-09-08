//! Authenticated MCP entry points. Human decisions have no MCP tool.
use crate::evidence::snapshot;
use crate::model::{Gate, Phase, Workflow, WorkflowId};
use crate::{store, validation};
use anyhow::{Result, ensure};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, Implementation, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::collections::BTreeMap;
use std::sync::Arc;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, McpServerId};
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::{
    McpArtifactRepository, McpToolExecutor, McpToolHandler, build_tool_list_result,
    client_profile_from_peer,
};
use systemprompt::models::artifacts::{CliArtifact, TextArtifact};
use systemprompt::models::execution::context::RequestContext as SysContext;
use systemprompt::security::authz::SharedAuthzHook;
use systemprompt_mcp_shared::record_mcp_access;

const TOOLS: [&str; 14] = [
    "refresh_requirements_evidence",
    "start_requirements_workflow",
    "get_requirements_workflow",
    "submit_discovery",
    "submit_design",
    "submit_proposal",
    "submit_grooming",
    "submit_specs",
    "submit_fsd",
    "validate_requirements_phase",
    "request_requirements_review",
    "generate_fsd_scaffold",
    "publish_requirements",
    "verify_requirements_publication",
];

#[derive(Debug, Clone)]
pub struct WorkflowServer {
    db: DbPool,
    pool: Arc<PgPool>,
    knowledge: Arc<PgPool>,
    service_id: McpServerId,
    hook: SharedAuthzHook,
    executor: McpToolExecutor,
}

impl WorkflowServer {
    pub fn new(
        db: DbPool,
        pool: Arc<PgPool>,
        knowledge: Arc<PgPool>,
        hook: SharedAuthzHook,
    ) -> Result<Self> {
        let executor = McpToolExecutor::new(
            Arc::new(ToolUsageRepository::new(&db)?),
            Arc::new(McpArtifactRepository::new(&db)?),
            "requirements-workflow",
        );
        Ok(Self {
            db,
            pool,
            knowledge,
            service_id: McpServerId::new("requirements-workflow"),
            hook,
            executor,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolInput {
    pub workflow_id: Option<String>,
    pub expected_revision: Option<i64>,
    pub name: Option<String>,
    pub project: Option<String>,
    pub delivery_scope: Option<String>,
    pub phase: Option<String>,
    pub gate: Option<String>,
    pub files: Option<BTreeMap<String, String>>,
}

fn parse_phase(name: &str) -> Result<Phase> {
    Phase::ALL
        .into_iter()
        .find(|p| p.name() == name)
        .ok_or_else(|| anyhow::anyhow!("Unknown phase"))
}

impl ServerHandler for WorkflowServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("Astound Requirements Workflow",env!("CARGO_PKG_VERSION")))
            .with_instructions("Use the four requirements skills. Sync Jira and Confluence before starting. Each artifact must pass validation before the next phase. Human review occurs in the browser; no tool can approve. Always use the returned revision. Evidence and review receipts are immutable history.")
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> std::result::Result<ListToolsResult, McpError> {
        let schema = serde_json::to_value(schemars::schema_for!(ToolInput))
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let tools=TOOLS.into_iter().map(|name|{let mut tool=Tool::default();tool.name=name.into();tool.description=Some(format!("{}; strict server-side phase and approval gates. Read workflow status for current revision.",name.replace('_'," ")).into());tool.input_schema=Arc::new(schema.as_object().cloned().unwrap_or_default());tool}).collect();
        Ok(build_tool_list_result(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        ctx: RequestContext<RoleServer>,
    ) -> std::result::Result<CallToolResponse, McpError> {
        let name = TOOLS
            .into_iter()
            .find(|n| *n == request.name.as_ref())
            .ok_or_else(|| McpError::invalid_params("Unknown workflow tool", None))?;
        let result = enforce_rbac_from_registry(&ctx, self.service_id.as_str(), &self.hook).await?;
        let auth = result.expect_authenticated("requirements-workflow requires authentication")?;
        record_mcp_access(
            &self.db,
            auth.context.user_id(),
            "requirements-workflow",
            name,
            "authenticated",
        )
        .await;
        let handler = Handler {
            server: self,
            name,
            bearer: auth.token(),
        };
        self.executor
            .execute(
                &handler,
                &request,
                &auth.context,
                &client_profile_from_peer(&ctx),
            )
            .await
            .map(Into::into)
    }
}

struct Handler<'a> {
    server: &'a WorkflowServer,
    name: &'static str,
    bearer: &'a str,
}
impl McpToolHandler for Handler<'_> {
    type Input = ToolInput;
    type Output = CliArtifact;
    fn tool_name(&self) -> &'static str {
        self.name
    }
    fn description(&self) -> &'static str {
        "Advance an authenticated, versioned requirements workflow"
    }
    async fn handle(
        &self,
        input: ToolInput,
        ctx: &SysContext,
        _id: &McpExecutionId,
    ) -> std::result::Result<(CliArtifact, String), McpError> {
        let result = self
            .execute(input, ctx)
            .await
            .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
        let body = serde_json::to_string_pretty(&result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok((
            CliArtifact::text(TextArtifact::new(&body).with_title("Requirements workflow")),
            self.name.to_owned(),
        ))
    }
}

impl Handler<'_> {
    async fn execute(&self, input: ToolInput, ctx: &SysContext) -> Result<Value> {
        let pool = &self.server.pool;
        let user = ctx.user_id();
        if self.name == "start_requirements_workflow" {
            ensure!(
                input.delivery_scope.as_deref() == Some("functional"),
                "Select functional scope explicitly. Split integration services into separate changes before discovery."
            );
            let project = input.project.as_deref().unwrap_or("storefront");
            store::require_project(pool, user, project).await?;
            let evidence = snapshot(&self.server.knowledge, user).await?;
            let workflow = Workflow::new(
                input
                    .name
                    .ok_or_else(|| anyhow::anyhow!("name is required"))?,
                evidence,
            )?;
            return Ok(serde_json::to_value(
                store::create(pool, user, project, workflow).await?,
            )?);
        }
        let id = WorkflowId(
            input
                .workflow_id
                .ok_or_else(|| anyhow::anyhow!("workflow_id is required"))?
                .parse()?,
        );
        let mut stored = store::get(pool, user, id).await?;
        if self.name == "get_requirements_workflow" {
            let jobs = crate::jobs::list(pool, &stored).await?;
            let mut value = serde_json::to_value(stored)?;
            value["validation_jobs"] = serde_json::to_value(jobs)?;
            return Ok(value);
        }
        ensure!(
            input.expected_revision == Some(stored.revision),
            "Reload workflow; expected_revision does not match"
        );
        let current = snapshot(&self.server.knowledge, user).await?;
        let current_hash = crate::model::digest(&current)?;
        if self.name == "refresh_requirements_evidence" {
            if current_hash != stored.workflow.evidence_hash {
                stored.workflow.evidence = current;
                stored.workflow.evidence_hash = current_hash;
                stored.workflow.artifacts.clear();
                stored.workflow.grooming_review = None;
                stored.workflow.publication_review = None;
            }
            stored.revision = store::save(pool, user, stored.revision, &stored, self.name).await?;
            return Ok(serde_json::to_value(stored)?);
        }
        ensure!(
            current_hash == stored.workflow.evidence_hash,
            "Source evidence changed. Refresh the snapshot; dependent artifacts and approvals must be regenerated."
        );
        if let Some(name) = self.name.strip_prefix("submit_") {
            stored.workflow.submit(
                parse_phase(name)?,
                input
                    .files
                    .ok_or_else(|| anyhow::anyhow!("files are required"))?,
            )?;
        } else {
            match self.name {
                "validate_requirements_phase" => {
                    let phase = parse_phase(
                        input
                            .phase
                            .as_deref()
                            .ok_or_else(|| anyhow::anyhow!("phase is required"))?,
                    )?;
                    return crate::jobs::start((**pool).clone(), user.clone(), stored, phase).await;
                },
                "request_requirements_review" => {
                    let gate = match input.gate.as_deref() {
                        Some("grooming") => Gate::Grooming,
                        Some("publication") => Gate::Publication,
                        _ => anyhow::bail!("gate must be grooming or publication"),
                    };
                    stored.workflow.request_review(gate)?;
                },
                "generate_fsd_scaffold" => {
                    return Ok(serde_json::to_value(
                        validation::scaffold(&stored.workflow).await?,
                    )?);
                },
                "publish_requirements" => {
                    return crate::publication::publish(
                        self.server,
                        pool,
                        ctx,
                        self.bearer,
                        &mut stored,
                    )
                    .await;
                },
                "verify_requirements_publication" => {
                    return crate::publication::verify(
                        self.server,
                        pool,
                        ctx,
                        self.bearer,
                        &stored,
                    )
                    .await;
                },
                _ => anyhow::bail!("Unknown operation"),
            }
        }
        stored.revision = store::save(pool, user, stored.revision, &stored, self.name).await?;
        Ok(json!({"workflow":stored,"review_url":format!("/admin/requirements/{}",id.0)}))
    }
}
