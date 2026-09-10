//! Requirements workflow MCP service entry point.
use anyhow::{Context, Result};
use std::sync::Arc;
use systemprompt::config::{ProfileBootstrap, SecretsBootstrap, init_config};
use systemprompt::system::AppContext;
use systemprompt_requirements_workflow::server::WorkflowServer;
use systemprompt_requirements_workflow::store;

systemprompt::security::register_subject_attribute_provider!(|ctx| {
    let provider: systemprompt::security::authz::SharedSubjectAttributeProvider = Arc::new(
        systemprompt_requirements_workflow::authz::MembershipProvider::groups(Arc::clone(
            &ctx.pool,
        )),
    );
    provider
});
systemprompt::security::register_subject_attribute_provider!(|ctx| {
    let provider: systemprompt::security::authz::SharedSubjectAttributeProvider = Arc::new(
        systemprompt_requirements_workflow::authz::MembershipProvider::projects(Arc::clone(
            &ctx.pool,
        )),
    );
    provider
});

#[tokio::main]
async fn main() -> Result<()> {
    systemprompt::logging::init_console_logging();
    ProfileBootstrap::init()?;
    SecretsBootstrap::init().await?;
    init_config(None)?;
    let ctx = AppContext::new().await?;
    let pool = ctx.db_pool().pool_arc().context("PostgreSQL is required")?;
    store::migrate(&pool).await?;
    let secrets = SecretsBootstrap::require()?;
    let knowledge = if let Some(url) = secrets.get("knowledge_database_url") {
        Arc::new(
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(4)
                .connect(url)
                .await?,
        )
    } else {
        Arc::clone(&pool)
    };
    let server = WorkflowServer::new(
        Arc::clone(ctx.db_pool()),
        pool,
        knowledge,
        Arc::clone(ctx.authz_hook()),
    )?;
    let router = systemprompt::mcp::create_router(
        server,
        Arc::clone(ctx.mcp_session_repository()),
        systemprompt::mcp::McpHttpConfig::default(),
    );
    let port = std::env::var("MCP_PORT")
        .unwrap_or_else(|_| "5031".to_owned())
        .parse::<u16>()?;
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
