//! Entry point for the `documents` MCP server binary.

use anyhow::{Context, Result};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use systemprompt::config::{LoadedProfile, SecretsService};
use systemprompt::identifiers::McpServerId;
use systemprompt::loader::BootContext;
use systemprompt::runtime::documents::mcp::DocumentsMcpTools;
use systemprompt::system::AppContext;
use systemprompt_mcp_documents::DocumentsServer;
use tokio::net::TcpListener;
// Why: core discovers the governance authz hook, the connection snapshot
// source and the group/project/role subject providers through inventory, so
// the crates that register them must be linked into this binary or rules on
// those bands never fire here. The external secrets source is found the same
// way, so a profile naming `onepassword` needs that crate linked too.
use systemprompt_secrets_onepassword as _;
use systemprompt_web_admin_governance as _;
use systemprompt_web_admin_identity as _;
use systemprompt_web_admin_platform as _;

const DEFAULT_SERVICE_ID: &str = "documents";
const DEFAULT_PORT: u16 = 5012;
const DEFAULT_STARTUP_TIMEOUT_SECS: u64 = 60;

fn port() -> u16 {
    env::var("MCP_PORT").map_or_else(
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
    )
}

fn startup_timeout() -> Duration {
    let secs = env::var("MCP_STARTUP_TIMEOUT_SECS").map_or(DEFAULT_STARTUP_TIMEOUT_SECS, |s| {
        s.parse::<u64>().unwrap_or_else(|e| {
            tracing::warn!(error = %e, value = %s, default = DEFAULT_STARTUP_TIMEOUT_SECS, "Invalid MCP_STARTUP_TIMEOUT_SECS, using default");
            DEFAULT_STARTUP_TIMEOUT_SECS
        })
    });
    Duration::from_secs(secs)
}

async fn init_context() -> Result<Arc<AppContext>> {
    let profile = LoadedProfile::from_env().context("Failed to initialize profile")?;
    let secrets = SecretsService::load(Arc::clone(&profile))
        .await
        .context("Failed to initialize secrets")?;
    Ok(Arc::new(
        AppContext::new(BootContext::new(profile, secrets))
            .await
            .context("Failed to initialize application context")?,
    ))
}

// Why: reqwest is built without a bundled crypto provider (core enables
// `rustls-no-provider`), so this binary, which does not enter through
// `systemprompt::cli::run`, installs ring before the first client is built.
fn install_tls_crypto_provider() {
    if rustls::crypto::CryptoProvider::get_default().is_some() {
        return;
    }
    if rustls::crypto::ring::default_provider()
        .install_default()
        .is_err()
    {
        tracing::debug!("rustls crypto provider was installed concurrently");
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    install_tls_crypto_provider();
    systemprompt::logging::init_console_logging();

    let addr = format!("0.0.0.0:{}", port());
    // Why: core's supervisor judges this process by pid and port, so the port
    // is claimed before the slow, bounded bring-up and a clash fails at once.
    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind {addr}"))?;
    let timeout = startup_timeout();
    let ctx = tokio::time::timeout(timeout, init_context())
        .await
        .with_context(|| {
            format!(
                "Startup did not complete within {}s (MCP_STARTUP_TIMEOUT_SECS)",
                timeout.as_secs()
            )
        })??;

    let service_id = env::var("MCP_SERVICE_ID")
        .map_or_else(
            |_| {
                tracing::warn!(
                    default = DEFAULT_SERVICE_ID,
                    "MCP_SERVICE_ID not set, using default"
                );
                McpServerId::try_new(DEFAULT_SERVICE_ID)
            },
            McpServerId::try_new,
        )
        .context("Invalid MCP_SERVICE_ID")?;

    let server = DocumentsServer::new(
        Arc::clone(ctx.db_pool()),
        systemprompt_mcp_shared::auth::ServerAccess {
            authz_hook: Arc::clone(ctx.authz_hook()),
            services: ctx.services_watch().clone(),
            token_authority: ctx.token_authority(),
            config: Arc::clone(ctx.config_arc()),
        },
        ctx.artifact_ingest_arc(),
        DocumentsMcpTools::from_context(&ctx, service_id.clone())
            .context("Document store unavailable")?,
    );
    let router = systemprompt::mcp::create_router(
        server,
        Arc::clone(ctx.mcp_session_repository()),
        systemprompt::mcp::McpHttpConfig {
            server_id: Some(service_id.clone()),
            ..systemprompt::mcp::McpHttpConfig::default()
        },
    );

    tracing::info!(
        service_id = %service_id,
        addr = %addr,
        "documents MCP server listening"
    );

    axum::serve(listener, router).await?;

    Ok(())
}
