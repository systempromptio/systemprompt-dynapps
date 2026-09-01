//! Entry point for the knowledge-bank MCP server binary.
//!
//! Two modes share one bootstrap: `serve` (what the platform spawns) and
//! `ingest` (an operator's one-shot run). Both probe the database for
//! pgvector and the secret store for credentials, and both degrade the same
//! way — no pgvector or no model key means the stub backend and no ingestor,
//! never a failed boot.

use anyhow::{Context, Result};
use std::env;
use std::sync::Arc;
use systemprompt::config::{ProfileBootstrap, SecretsBootstrap, init_config};
use systemprompt::identifiers::McpServerId;
use systemprompt::system::AppContext;
use systemprompt_mcp_knowledge_bank::cli::{self, Command, USAGE};
use systemprompt_mcp_knowledge_bank::config::{KnowledgeBankConfig, SECRET_KEY_NAME};
use systemprompt_mcp_knowledge_bank::ingest::scheduler;
use systemprompt_mcp_knowledge_bank::ingest::sources::{AtlassianClient, AtlassianCredentials};
use systemprompt_mcp_knowledge_bank::model::build_provider;
use systemprompt_mcp_knowledge_bank::server::format_report;
use systemprompt_mcp_knowledge_bank::{
    Ingestor, KnowledgeBankServer, KnowledgeRetriever, PgRetriever, StubRetriever, schema,
};
use tokio::net::TcpListener;

const DEFAULT_SERVICE_ID: &str = "knowledge-bank";
const DEFAULT_PORT: u16 = 5030;

struct Backend {
    retriever: Arc<dyn KnowledgeRetriever>,
    ingestor: Option<Arc<Ingestor>>,
    config: Arc<KnowledgeBankConfig>,
}

#[tokio::main]
async fn main() -> Result<()> {
    systemprompt::logging::init_console_logging();

    let command = cli::parse(env::args().skip(1)).map_err(|message| anyhow::anyhow!(message))?;
    if command == Command::Help {
        print_usage();
        return Ok(());
    }

    ProfileBootstrap::init().context("Failed to initialize profile")?;
    SecretsBootstrap::init().context("Failed to initialize secrets")?;
    init_config().context("Failed to initialize configuration")?;

    let ctx = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );
    let backend = build_backend(&ctx).await?;

    match command {
        Command::Ingest(options) => run_ingest(&backend, &options).await,
        Command::Serve | Command::Help => serve(ctx, backend).await,
    }
}

async fn run_ingest(
    backend: &Backend,
    options: &systemprompt_mcp_knowledge_bank::RunOptions,
) -> Result<()> {
    let Some(ingestor) = &backend.ingestor else {
        anyhow::bail!(
            "ingestion is unavailable on this instance: the database has no pgvector or the \
             profile secret store has no `{SECRET_KEY_NAME}`"
        );
    };
    let report = ingestor.run(options).await?;
    print_report(&report);
    if report.has_errors() {
        anyhow::bail!("ingestion finished with errors; see the report above");
    }
    Ok(())
}

#[expect(
    clippy::print_stdout,
    reason = "the ingest subcommand's report is its stdout"
)]
fn print_report(report: &systemprompt_mcp_knowledge_bank::IngestReport) {
    println!("{}", format_report(report));
}

#[expect(
    clippy::print_stdout,
    reason = "--help prints usage to stdout by convention"
)]
fn print_usage() {
    println!("{USAGE}");
}

async fn serve(ctx: Arc<AppContext>, backend: Backend) -> Result<()> {
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

    if let Some(ingestor) = &backend.ingestor
        && backend.config.ingestion.enabled
    {
        scheduler::spawn(Arc::clone(ingestor), &backend.config.ingestion.schedule)
            .context("Failed to start the ingestion schedule")?;
    }

    let server = KnowledgeBankServer::new(
        Arc::clone(ctx.db_pool()),
        service_id.clone(),
        Arc::clone(ctx.authz_hook()),
        backend.retriever,
        backend.ingestor,
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

// Why: an instance that cannot index is a supported state, not a failure —
// the server still starts and every tool answers BACKEND_NOT_CONFIGURED with
// the reason. Falling back to a retriever that returns nothing would be
// indistinguishable from an empty corpus, which is the confusion the stub
// exists to prevent.
async fn build_backend(ctx: &AppContext) -> Result<Backend> {
    let config = Arc::new(
        KnowledgeBankConfig::load(&KnowledgeBankConfig::resolve_path())
            .context("Failed to load knowledge-bank config")?,
    );
    let inert = |reason: String| Backend {
        retriever: Arc::new(StubRetriever::new(reason)),
        ingestor: None,
        config: Arc::clone(&config),
    };

    let pool = ctx
        .db_pool()
        .pool_arc()
        .context("Knowledge-bank needs a Postgres pool")?;
    if !schema::ensure(&pool).await? {
        let reason = "pgvector is not available on this database; the knowledge bank is inert \
                      here (container deployments run pgvector/pgvector:pg18)";
        tracing::warn!("{reason}");
        return Ok(inert(reason.to_owned()));
    }

    let secrets = SecretsBootstrap::require().context("Secrets are not initialized")?;
    let Some(api_key) = secrets.get(SECRET_KEY_NAME) else {
        let reason = format!("no `{SECRET_KEY_NAME}` in the profile secret store");
        tracing::warn!("{reason}; serving the stub backend");
        return Ok(inert(reason));
    };
    let model = build_provider(&config.model, api_key.clone())
        .context("Failed to initialize the model provider")?;

    let atlassian = AtlassianCredentials::from_lookup(|key| secrets.get(key).cloned())
        .map(AtlassianClient::new)
        .transpose()
        .context("Failed to initialize the Atlassian client")?;
    if atlassian.is_none() {
        tracing::info!(
            "no Atlassian service-account secrets; Confluence and Jira sources will be skipped"
        );
    }

    tracing::info!(
        provider = ?config.model.provider,
        region = %config.model.bedrock.region,
        rerank = config.retrieval.rerank_enabled,
        sources = config.sources.len(),
        "knowledge-bank retrieval backend ready"
    );
    let ingestor = Arc::new(Ingestor::new(
        Arc::clone(&pool),
        Arc::clone(&model),
        Arc::clone(&config),
        atlassian,
    ));
    Ok(Backend {
        retriever: Arc::new(PgRetriever::new(pool, model, (*config).clone())),
        ingestor: Some(ingestor),
        config,
    })
}
