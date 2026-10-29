//! The running installation docs-smoke drives: a profile over the run's
//! throwaway database with the document store and the `documents` MCP
//! server switched on, the API served from that profile on a loopback
//! socket, and the `systemprompt-mcp-documents` binary started as its own
//! process on that same profile. Both read one signing key, so a token the
//! runtime mints is honoured by the API and the MCP server alike. Document
//! blobs land in the run's own storage directory, beside a link to the
//! shipped `storage/files` tree the pages render from.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use systemprompt::identifiers::{AccessTokenId, SessionId, UserId};
use systemprompt::models::auth::{AuthenticatedUser, JwtAudience, Permission, UserJwtRequest};
use systemprompt::system::AppContext;

use systemprompt_security::keys::RsaSigningKey;
use tokio::process::{Child, Command};

use crate::globals;

pub(super) const SYSTEM_ADMIN_ID: &str = "00000000-0000-4000-8000-00000000ad01";
const BINARY: &str = "systemprompt-mcp-documents";
const READY_TIMEOUT: Duration = Duration::from_secs(120);

pub(super) struct Stack {
    pub(super) ctx: Arc<AppContext>,
    pub(super) api: String,
    pub(super) mcp: String,
    _server: tokio::task::JoinHandle<()>,
    _binary: Child,
}

// Why: the services config refuses an MCP server port outside 5000-5999.
fn free_mcp_port() -> u16 {
    let start = 5000 + u16::try_from(std::process::id() % 1000).unwrap_or(0);
    (0..1000u16)
        .map(|step| 5000 + (start - 5000 + step) % 1000)
        .find(|port| std::net::TcpListener::bind(("127.0.0.1", *port)).is_ok())
        .expect("a free port in the MCP range")
}

pub(super) fn binary_path() -> PathBuf {
    let path = globals::repo_root().join("target/debug").join(BINARY);
    assert!(
        path.is_file(),
        "{} is missing; `just docs-smoke` builds it before the run",
        path.display()
    );
    path
}

fn write_profile(database_url: &str, mcp_port: u16) -> PathBuf {
    let dir = globals::repo_root().join(format!(
        "tests/target/docs-smoke-profile-{}",
        std::process::id()
    ));
    let profile = globals::write_profile(&dir, database_url);
    let key = dir.join("signing_key.pem");
    RsaSigningKey::generate()
        .and_then(|generated| generated.write_pem_file(&key))
        .expect("write the run's signing key");
    let yaml = std::fs::read_to_string(&profile).expect("read the written profile");
    std::fs::write(
        &profile,
        yaml.replace(
            "signing_key_path: signing_key.pem",
            &format!("signing_key_path: {}", key.display()),
        ),
    )
    .expect("point the profile at the run's signing key");
    let storage = dir.join("storage");
    std::fs::create_dir_all(&storage).expect("create the run's storage");
    let files = storage.join("files");
    if !files.exists() {
        std::os::unix::fs::symlink(globals::repo_root().join("storage/files"), &files)
            .expect("link the shipped storage/files tree");
    }
    let yaml = std::fs::read_to_string(&profile).expect("read the written profile");
    std::fs::write(
        &profile,
        yaml.replace(
            &format!("storage: {}/storage", globals::repo_root().display()),
            &format!("storage: {}", storage.display()),
        ),
    )
    .expect("keep the run's blobs out of the repository");
    let services = dir.join("services");
    std::fs::write(
        services.join("config/files.yaml"),
        "files:\n  documents:\n    enabled: true\n",
    )
    .expect("switch the document store on");
    let server = services.join("mcp/documents.yaml");
    let yaml = std::fs::read_to_string(&server).expect("read documents.yaml");
    std::fs::write(
        &server,
        yaml.replacen("enabled: false", "enabled: true", 1)
            .replacen("port: 5012", &format!("port: {mcp_port}"), 1),
    )
    .expect("switch the documents server on");
    profile
}

async fn boot(profile: &Path) -> Arc<AppContext> {
    let loaded =
        systemprompt::config::LoadedProfile::from_path(profile).expect("load the run's profile");
    let secrets = systemprompt::config::SecretsService::load(Arc::clone(&loaded))
        .await
        .expect("load the run's secrets");
    let boot = systemprompt::loader::BootContext::new(loaded, secrets);
    boot.try_init_config(None)
        .expect("build config from the run's profile");
    Arc::new(
        AppContext::new(boot)
            .await
            .expect("the runtime boots on the run's profile"),
    )
}

async fn serve_api(ctx: &AppContext) -> (String, tokio::task::JoinHandle<()>) {
    let readiness = Arc::new(systemprompt::api::services::server::ReadinessState::new());
    let router = systemprompt::api::services::server::setup_api_server(ctx, None, &readiness)
        .expect("the API router builds");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the API listener");
    let addr: SocketAddr = listener.local_addr().expect("the API listener's address");
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("the API serves");
    });
    (format!("http://{addr}"), server)
}

async fn start_binary(profile: &Path, port: u16) -> Child {
    let mut child = Command::new(binary_path())
        .env("SYSTEMPROMPT_PROFILE", profile)
        .env("MCP_PORT", port.to_string())
        .env("MCP_SERVICE_ID", "documents")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .expect("start the documents MCP binary");
    let deadline = tokio::time::Instant::now() + READY_TIMEOUT;
    loop {
        if let Some(status) = child.try_wait().expect("poll the documents MCP binary") {
            panic!("the documents MCP binary exited before it served: {status}");
        }
        let probe = reqwest::Client::new()
            .post(format!("http://127.0.0.1:{port}/mcp"))
            .send()
            .await;
        if probe.is_ok() {
            return child;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "the documents MCP binary did not serve within {READY_TIMEOUT:?}"
        );
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

// Why: the runtime refuses to boot until the profile's `system_admin`
// account exists.
pub(super) async fn seed_system_admin(pool: &sqlx::PgPool) {
    sqlx::query(
        "INSERT INTO users (id, name, email, roles, email_verified)
         VALUES ($1, 'admin', 'admin@contract.test', ARRAY['admin', 'user'], true)",
    )
    .bind(SYSTEM_ADMIN_ID)
    .execute(pool)
    .await
    .expect("seed the system admin");
}

// The `mcp_server/documents` band as services/access-control/rules.yaml
// declares it, which `config apply` writes on an installation: closed, and
// open to every signed-in user.
pub(super) async fn seed_documents_access(pool: &sqlx::PgPool) {
    sqlx::query(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
         VALUES ('mcp_server', 'documents', false, 'console:docs-smoke')",
    )
    .execute(pool)
    .await
    .expect("register the documents server");
    sqlx::query(
        "INSERT INTO access_control_rules
             (id, entity_type, entity_id, rule_type, rule_value, access, justification, source)
         VALUES ($1, 'mcp_server', 'documents', 'role', 'user', 'allow',
                 'every signed-in user reaches the documents server', 'console:docs-smoke')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .execute(pool)
    .await
    .expect("open the documents server to users");
}

pub(super) async fn start(database_url: &str) -> Stack {
    let mcp_port = free_mcp_port();
    let profile = write_profile(database_url, mcp_port);
    let ctx = boot(&profile).await;
    // Why: the server's run loop registers the document folders as
    // access-control entities before it serves; the router alone does not.
    ctx.documents()
        .reconcile_folder_entities()
        .await
        .expect("register the folder entities");
    let (api, server) = serve_api(&ctx).await;
    let binary = start_binary(&profile, mcp_port).await;
    Stack {
        ctx,
        api,
        mcp: format!("http://127.0.0.1:{mcp_port}/mcp"),
        _server: server,
        _binary: binary,
    }
}

impl Stack {
    // A signed-in person's bearer: a live session row and a user token the
    // runtime mints with the run's key.
    pub(super) async fn token(&self, pool: &sqlx::PgPool, user: &UserId) -> String {
        let session = SessionId::new(uuid::Uuid::new_v4().to_string());
        sqlx::query("INSERT INTO user_sessions (session_id, user_id) VALUES ($1, $2)")
            .bind(session.as_str())
            .bind(user.as_str())
            .execute(pool)
            .await
            .expect("seed the session");
        self.ctx
            .token_authority()
            .mint_user_jwt(&UserJwtRequest {
                user: AuthenticatedUser::new_with_roles(
                    user.clone(),
                    user.as_str().to_owned(),
                    format!("{}@contract.test", user.as_str()),
                    vec![Permission::User],
                    vec!["user".to_owned()],
                ),
                session_id: session,
                jti: AccessTokenId::generate(),
                permissions: vec![Permission::User],
                audience: JwtAudience::standard(),
                ttl: chrono::Duration::hours(1),
                client_id: None,
                purpose: None,
                plugin_id: None,
                interaction: None,
                browser_session: true,
            })
            .expect("mint the session token")
    }
}
