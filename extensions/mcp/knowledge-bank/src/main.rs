//! Entry point for the knowledge-bank MCP server binary.

use anyhow::{Context, Result};
use std::env;
use std::sync::Arc;
use systemprompt::config::{ProfileBootstrap, SecretsBootstrap, init_config};
use systemprompt::identifiers::McpServerId;
use systemprompt::system::AppContext;
use systemprompt_mcp_knowledge_bank::config::{KnowledgeBankConfig, SECRET_KEY_NAME};
use systemprompt_mcp_knowledge_bank::{
    BedrockClient, KnowledgeBankServer, KnowledgeRetriever, PgRetriever, StubRetriever,
};
use tokio::net::TcpListener;

const DEFAULT_SERVICE_ID: &str = "knowledge-bank";
const DEFAULT_PORT: u16 = 5030;

#[tokio::main]
async fn main() -> Result<()> {
    systemprompt::logging::init_console_logging();

    ProfileBootstrap::init().context("Failed to initialize profile")?;
    SecretsBootstrap::init().context("Failed to initialize secrets")?;
    init_config().context("Failed to initialize configuration")?;

    let ctx = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );

    let service_id = env::var("MCP_SERVICE_ID").map_or_else(
        |_| {
            tracing::warn!(
                default = DEFAULT_SERVICE_ID,
                "MCP_SERVICE_ID not set, using default"
            );
            McpServerId::new(DEFAULT_SERVICE_ID)
        },
        McpServerId::new,
    );

    let port = env::var("MCP_PORT").map_or_else(
        |_| {
            tracing::warn!(default = DEFAULT_PORT, "MCP_PORT not set, using default");
            DEFAULT_PORT
        },
        |p| {
            p.parse::<u16>().unwrap_or_else(|e| {
                tracing::warn!(error = %e, port = %p, default = DEFAULT_PORT, "Invalid MCP_PORT, using default");
                DEFAULT_PORT
            })
        },
    );

    let retriever = build_retriever(&ctx)?;

    let server = KnowledgeBankServer::new(
        Arc::clone(ctx.db_pool()),
        service_id.clone(),
        Arc::clone(ctx.authz_hook()),
        retriever,
    )
    .context("Failed to initialize KnowledgeBankServer")?;
    let router = systemprompt::mcp::create_router(
        server,
        Arc::clone(ctx.mcp_session_repository()),
        systemprompt::mcp::McpHttpConfig::default(),
    );
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!(
        service_id = %service_id,
        addr = %addr,
        "Knowledge-bank MCP server listening"
    );

    axum::serve(listener, router).await?;

    Ok(())
}

// Why: no Bedrock key is a supported state, not a failure — the server still
// starts and every tool answers BACKEND_NOT_CONFIGURED. Falling back to a
// retriever that returns nothing would be indistinguishable from an empty
// index, which is exactly the confusion the stub exists to prevent.
fn build_retriever(ctx: &AppContext) -> Result<Arc<dyn KnowledgeRetriever>> {
    let config = KnowledgeBankConfig::load(&KnowledgeBankConfig::resolve_path())
        .context("Failed to load knowledge-bank config")?;

    let secrets = SecretsBootstrap::require().context("Secrets are not initialized")?;
    let Some(api_key) = secrets.get(SECRET_KEY_NAME) else {
        tracing::warn!(
            secret = SECRET_KEY_NAME,
            "no Bedrock key in the profile secret store; serving the stub backend"
        );
        return Ok(Arc::new(StubRetriever));
    };

    let bedrock = BedrockClient::new(config.bedrock.clone(), api_key.clone())
        .context("Failed to initialize the Bedrock client")?;
    tracing::info!(
        region = %config.bedrock.region,
        embed_model = %config.bedrock.embed_model,
        rerank = config.retrieval.rerank_enabled,
        "knowledge-bank retrieval backend ready"
    );
    let pool = ctx
        .db_pool()
        .pool_arc()
        .context("Knowledge-bank retrieval needs a Postgres pool")?;
    Ok(Arc::new(PgRetriever::new(pool, Arc::new(bedrock), config)))
}
