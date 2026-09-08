//! Entry point for the knowledge-bank MCP server binary.
//!
//! Serve only: users reach this process through the platform's MCP proxy
//! and sync with their own credentials, so there is no ingest subcommand and
//! no schedule. Boot opens the knowledge database (the `knowledge_database_url`
//! secret, or the instance pool without it), requires pgvector, builds the
//! configured model provider — `local` needs nothing, `bedrock` needs the
//! `bedrock_api_key` secret — and exits non-zero naming what is missing: an
//! index that cannot exist must stop the deploy, not serve empty answers.

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;
use systemprompt::config::{ProfileBootstrap, SecretsBootstrap, init_config};
use systemprompt::identifiers::McpServerId;
use systemprompt::system::AppContext;
use systemprompt_mcp_knowledge_bank::config::{KnowledgeBankConfig, SECRET_KEY_NAME};
use systemprompt_mcp_knowledge_bank::maintenance::Pruner;
use systemprompt_mcp_knowledge_bank::model::build_provider;
use systemprompt_mcp_knowledge_bank::server::KnowledgeBackend;
use systemprompt_mcp_knowledge_bank::{Ingestor, KnowledgeBankServer, PgRetriever, schema};
use tokio::net::TcpListener;

const DEFAULT_SERVICE_ID: &str = "knowledge-bank";
const DEFAULT_PORT: u16 = 5030;
const KNOWLEDGE_POOL_SIZE: u32 = 8;

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
    let backend = build_backend(&ctx).await?;
    systemprompt_mcp_knowledge_bank::drive::worker::spawn(Arc::clone(&backend.ingestor));
    serve(ctx, backend).await
}

async fn serve(ctx: Arc<AppContext>, backend: KnowledgeBackend) -> Result<()> {
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

    let server = KnowledgeBankServer::new(
        Arc::clone(ctx.db_pool()),
        service_id.clone(),
        Arc::clone(ctx.authz_hook()),
        backend,
    )
    .context("Failed to initialize KnowledgeBankServer")?;
    let router = systemprompt::mcp::create_router(
        server,
        Arc::clone(ctx.mcp_session_repository()),
        systemprompt::mcp::McpHttpConfig::default(),
    );
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!(service_id = %service_id, addr = %addr, "Knowledge-bank MCP server listening");
    axum::serve(listener, router).await?;
    Ok(())
}

async fn build_backend(ctx: &AppContext) -> Result<KnowledgeBackend> {
    let config = Arc::new(
        KnowledgeBankConfig::load(&KnowledgeBankConfig::resolve_path())
            .context("Failed to load knowledge-bank config")?,
    );
    let secrets = SecretsBootstrap::require().context("Secrets are not initialized")?;
    let instance_pool = ctx
        .db_pool()
        .pool_arc()
        .context("Knowledge-bank needs a Postgres pool")?;

    let (knowledge_pool, database) =
        if let Some(url) = secrets.get(&config.knowledge_database.secret) {
            let described = schema::describe_database_url(url);
            let pool = PgPoolOptions::new()
                .max_connections(KNOWLEDGE_POOL_SIZE)
                .connect(url)
                .await
                .with_context(|| format!("knowledge-bank: cannot connect to {described}"))?;
            (Arc::new(pool), described)
        } else {
            tracing::info!(
                secret = %config.knowledge_database.secret,
                "no knowledge database secret; the index lives in the instance database"
            );
            (
                Arc::clone(&instance_pool),
                "the instance database".to_owned(),
            )
        };
    schema::require(&knowledge_pool, &database)
        .await
        .context("Failed to prepare the knowledge database")?;

    let api_key = secrets.get(SECRET_KEY_NAME).cloned();
    let model = build_provider(&config.model, api_key)
        .context("Failed to initialize the model provider")?;

    let region = config
        .model
        .provider
        .requires_api_key()
        .then_some(config.model.bedrock.region.as_str());
    tracing::info!(
        provider = config.model.provider.as_str(),
        embed_model = model.embed_model(),
        region,
        chat = model.supports_chat(),
        rerank = config.retrieval.rerank_enabled,
        sources = config.sources.len(),
        database = %database,
        "knowledge-bank backend ready"
    );
    Ok(KnowledgeBackend {
        retriever: Arc::new(PgRetriever::new(
            Arc::clone(&knowledge_pool),
            Arc::clone(&model),
            (*config).clone(),
        )),
        ingestor: Arc::new(Ingestor::new(
            Arc::clone(&knowledge_pool),
            model,
            Arc::clone(&config),
        )),
        pruner: Arc::new(Pruner::new(knowledge_pool, instance_pool)),
    })
}
