//! The `odoo` MCP server: struct construction and the rmcp `ServerHandler`
//! surface (info, tool listing, call dispatch, artifact-viewer resources).
//!
//! Per-call logic — credential resolution and routing — lives in the [`tool`]
//! submodule; the handlers themselves in [`crm`], [`report`], [`partner`],
//! [`partner_write`], [`discovery`], [`sales`], [`notes`], [`attachments`],
//! [`activity`], [`calendar`], [`tasks`], [`channels`], [`overview`] and the
//! destructive and erasure tools in [`guarded`].

pub mod activity;
pub mod attachments;
pub mod briefing;
pub mod calendar;
pub mod call;
pub mod channels;
pub mod crm;
mod crm_shape;
pub mod discovery;
mod discovery_shape;
pub mod gate;
pub mod guarded;
pub mod notes;
mod notes_shape;
pub mod overview;
mod overview_shape;
pub mod partner;
pub mod partner_write;
pub mod report;
pub mod sales;
mod sales_shape;
mod serve;
pub mod tasks;
#[doc(hidden)]
pub mod tool;
mod work_shape;

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, Implementation, InitializeRequestParams,
    InitializeResult, ListResourceTemplatesResult, ListResourcesResult, ListToolsResult,
    PaginatedRequestParams, ProtocolVersion, ReadResourceRequestParams, ReadResourceResponse,
    ServerCapabilities, ServerConfig,
};
use rmcp::service::{MaybeSendFuture, RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use std::future::Future;
use std::sync::Arc;
use systemprompt::ai::AiRequestRepository;
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::{
    ArtifactIngest, ArtifactViewerConfig, McpArtifactRepository, McpToolExecutor,
    artifact_shell_template, build_artifact_viewer_resource, build_extension_capabilities,
    build_resource_template_list_result, build_tool_list_result, client_profile_from_peer,
    parse_artifact_resource_uri, read_artifact_resource, read_artifact_viewer_resource,
};
use systemprompt::security::authz::SharedAuthzHook;
use systemprompt::traits::DynToolCallIntentClaims;
use systemprompt_mcp_shared::auth::authenticate_tool_request;

use crate::client::OdooClient;
use crate::credential::CredentialSource;
use crate::error::OdooError;
use crate::guard::demo_scope_enabled;
use crate::tools::{self, SERVER_NAME};
use systemprompt::users::UserRepository;
use systemprompt_web_admin_platform::service_accounts::ServiceAccounts;

const INSTRUCTIONS: &str = "Odoo CRM. Every call here runs as the Odoo login named by your own \
                            sealed Odoo credential (on this instance, a service account's \
                            `odoo_api_key`), so what you can see and change is exactly what Odoo \
                            permits that login. Prefer business_overview_data for a daily \
                            picture and crm_lead_report for pipeline numbers. The deletes and \
                            partner_anonymise are irreversible: each needs a `reason`, and while \
                            demo scope is on they only touch records tagged `demo-*`. Call \
                            gdpr_erase_inventory before anonymising a contact.";

#[derive(Clone, Debug)]
pub struct OdooServer {
    service_id: McpServerId,
    db_pool: DbPool,
    executor: McpToolExecutor,
    authz_hook: SharedAuthzHook,
    client: Arc<OdooClient>,
    source: CredentialSource,
}

impl OdooServer {
    pub fn new(
        db_pool: DbPool,
        service_id: McpServerId,
        authz_hook: SharedAuthzHook,
        artifact_ingest: Arc<ArtifactIngest>,
    ) -> Result<Self, OdooError> {
        let tool_usage_repo = Arc::new(ToolUsageRepository::new(&db_pool));
        let intent_claims: DynToolCallIntentClaims = Arc::new(AiRequestRepository::new(&db_pool));
        let executor = McpToolExecutor::new(
            tool_usage_repo,
            intent_claims,
            artifact_ingest,
            service_id.clone(),
        );
        let client = Arc::new(OdooClient::new()?);
        let source = CredentialSource {
            accounts: ServiceAccounts::new(UserRepository::new(&db_pool)),
        };

        tracing::info!(
            demo_scope = demo_scope_enabled(),
            "Odoo MCP server configured"
        );

        Ok(Self {
            service_id,
            db_pool,
            executor,
            authz_hook,
            client,
            source,
        })
    }
}

impl ServerHandler for OdooServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_extensions_with(build_extension_capabilities())
                .build(),
        )
        .with_protocol_version(ProtocolVersion::V_2026_07_28)
        .with_server_info(
            Implementation::new(
                format!("Odoo ({})", self.service_id),
                env!("CARGO_PKG_VERSION"),
            )
            .with_title("Odoo CRM"),
        )
        .with_instructions(INSTRUCTIONS.to_owned())
    }

    fn initialize(
        &self,
        _request: InitializeRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<InitializeResult, McpError>> + MaybeSendFuture + '_ {
        tracing::info!("odoo MCP server initialized");
        std::future::ready(Ok(self.get_info()))
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + MaybeSendFuture + '_ {
        std::future::ready(Ok(build_tool_list_result(tools::list_tools())))
    }

    fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourceTemplatesResult, McpError>> + MaybeSendFuture + '_
    {
        std::future::ready(Ok(build_resource_template_list_result()))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let tool_name = request.name.to_string();
        let server_name = self.service_id.to_string();

        let (request_context, _auth_token) = authenticate_tool_request(
            &self.db_pool,
            &tool_name,
            &self.service_id,
            &ctx,
            &self.authz_hook,
        )
        .await?;

        let profile = client_profile_from_peer(&ctx);
        Box::pin(serve::serve(&serve::Serve {
            db_pool: &self.db_pool,
            executor: &self.executor,
            client: &self.client,
            source: &self.source,
            request: &request,
            request_context: &request_context,
            profile: &profile,
            server_name: &server_name,
        }))
        .await
        .map(Into::into)
    }

    fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListResourcesResult, McpError>> + MaybeSendFuture + '_ {
        std::future::ready(Ok(build_artifact_viewer_resource(&ArtifactViewerConfig {
            server_name: &McpServerId::new(SERVER_NAME),
            title: "Odoo Artifact Viewer",
            description: "Interactive UI viewer for Odoo artifacts. Receives the tool result \
                          via the MCP Apps ui/notifications/tool-result protocol and mounts \
                          the server-rendered artifact HTML it carries.",
            template: &artifact_shell_template(),
            icons: None,
        })))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        if parse_artifact_resource_uri(&request.uri).is_some() {
            let repo = McpArtifactRepository::new(&self.db_pool);
            return read_artifact_resource(&request, &McpServerId::new(SERVER_NAME), &repo)
                .await
                .map(Into::into);
        }

        read_artifact_viewer_resource(
            &request,
            &McpServerId::new(SERVER_NAME),
            &artifact_shell_template(),
        )
        .map(Into::into)
    }
}
