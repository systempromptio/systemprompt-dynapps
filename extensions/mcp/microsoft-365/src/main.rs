//! Entry point for the `microsoft-365` MCP server binary.

use anyhow::{Context, Result};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use systemprompt::config::GrantedSecrets;
use systemprompt::identifiers::McpServerId;
use systemprompt::system::HostContext;
use systemprompt::traits::SecretLookup;
use systemprompt_mcp_microsoft_365::graph::{Broker, GraphLimits, UploadSessions, http_client};
use systemprompt_mcp_microsoft_365::{GraphHost, Microsoft365Server, ToolBackends, UPLOAD_TOOL};
use systemprompt_mcp_shared::files::{FilePolicy, McpFiles};
// Why: core discovers the governance authz hook, the connection snapshot
// source, the group/project/role subject providers and the document template
// providers through inventory, so the crates that register them must be
// linked into this binary or rules on those bands never fire here. The
// external secrets source is found the same way, so a profile naming
// `onepassword` needs that crate too.
use systemprompt_secrets_onepassword as _;
use systemprompt_web_admin_governance as _;
use systemprompt_web_admin_identity as _;
use systemprompt_web_admin_platform as _;
use systemprompt_web_shared as _;

const DEFAULT_SERVICE_ID: &str = systemprompt_mcp_microsoft_365::SERVER_NAME;
const DEFAULT_PORT: u16 = 5013;
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

async fn init_context() -> Result<Arc<HostContext>> {
    Ok(Arc::new(
        HostContext::from_process_env()
            .await
            .context("Failed to initialize the MCP host context")?,
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

    let addr = format!("127.0.0.1:{}", port());
    // Why: core's supervisor judges this process by pid and port, so the port
    // is claimed before the slow, bounded bring-up and a clash fails at once.
    let listener = systemprompt::mcp::bind_loopback(port())
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

    let limits = GraphLimits::default();
    let secrets: Arc<dyn SecretLookup> = Arc::<GrantedSecrets>::clone(ctx.granted_secrets());
    let graph = GraphHost {
        http: http_client(&limits).context("Failed to build the Graph HTTP client")?,
        broker: Broker::for_instance(ctx.config(), Arc::clone(&secrets), service_id.as_str()),
        secrets,
        limits,
        upload_sessions: UploadSessions::default(),
    };
    let files = McpFiles::from_host(&ctx, service_id.clone(), FilePolicy::new(UPLOAD_TOOL))
        .context("Failed to build the file layer")?;
    let server = Microsoft365Server::new(
        service_id.clone(),
        Arc::clone(ctx.db_pool()),
        systemprompt_mcp_shared::auth::ServerAccess {
            authz_hook: Arc::clone(ctx.authz_hook()),
            services: ctx.services_state().watch().clone(),
        },
        ctx.artifact_ingest(),
        ToolBackends { graph, files },
    );
    let requires_oauth = ctx
        .services_state()
        .config()
        .mcp_servers
        .get(service_id.as_str())
        .is_some_and(|deployment| deployment.oauth.required);
    let router = systemprompt::mcp::create_router(
        server,
        Arc::clone(ctx.mcp_session_repository()),
        systemprompt::mcp::McpHttpConfig::default(),
        systemprompt::mcp::middleware::HopVerifier::new(
            ctx.verification_keys().clone(),
            &ctx.config().jwt_issuer,
            service_id.clone(),
        )
        .requiring_hop(requires_oauth),
    );

    tracing::info!(
        service_id = %service_id,
        addr = %addr,
        "microsoft-365 MCP server listening"
    );

    axum::serve(listener, router).await?;

    Ok(())
}
