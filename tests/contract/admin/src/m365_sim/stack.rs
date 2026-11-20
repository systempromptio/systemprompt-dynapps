//! The installation the Microsoft 365 simulation drives: a profile over the
//! run's throwaway database with the document store, the `documents` server
//! and the `microsoft-365` server switched on, the Entra connector pointed at
//! the simulated Entra and Graph (`tests/fixtures/oauth/m365_sim.py`), the
//! API served from that profile on the loopback port the profile names (the
//! connector broker and the signed file links are reached there), and both
//! MCP binaries started as their own processes with the environment core's
//! spawner hands them.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;
use systemprompt::identifiers::{AccessTokenId, McpServerId, UserId};
use systemprompt::mcp::services::process::spawner;
use systemprompt::models::auth::{
    AuthenticatedUser, JwtAudience, McpHopIdentity, Permission, TokenInteraction, UserJwtRequest,
    UserType,
};
use systemprompt::system::AppContext;
use systemprompt_security::keys::RsaSigningKey;
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};

use crate::docs_smoke::mcp::McpCaller;
use crate::docs_smoke::stack as docs;
use crate::globals;

pub(super) const M365: &str = "microsoft-365";
pub(super) const DOCUMENTS: &str = "documents";
const READY_TIMEOUT: Duration = Duration::from_secs(120);

// What the simulator printed: its two origins and the Entra app it accepts.
pub(super) struct Sim {
    pub(super) entra: String,
    pub(super) graph: String,
    pub(super) tenants: Value,
    client_id: String,
    client_secret: String,
    _process: Child,
}

pub(super) struct Stack {
    pub(super) ctx: Arc<AppContext>,
    pub(super) api: String,
    pub(super) sim: Sim,
    pub(super) secrets: PathBuf,
    ports: [u16; 2],
    _server: tokio::task::JoinHandle<()>,
    _binaries: Vec<Child>,
}

fn binary(name: &str) -> PathBuf {
    let path = globals::repo_root().join("target/debug").join(name);
    assert!(
        path.is_file(),
        "{} is missing; `just m365-sim` builds it before the run",
        path.display()
    );
    path
}

pub(super) async fn start_sim() -> Sim {
    let tls = std::env::var("M365_SIM_TLS_DIR")
        .expect("M365_SIM_TLS_DIR is unset; run the simulation through `just m365-sim`");
    let script = globals::repo_root().join("tests/fixtures/oauth/m365_sim.py");
    let mut process = Command::new("python3")
        .arg(script)
        .arg("--serve")
        .arg(&tls)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .expect("start the Entra and Graph simulator");
    let stdout = process.stdout.take().expect("the simulator's stdout");
    let mut line = String::new();
    tokio::io::BufReader::new(stdout)
        .read_line(&mut line)
        .await
        .expect("read the simulator's origins");
    let printed: Value = serde_json::from_str(&line).expect("the simulator prints JSON");
    let text = |key: &str| printed[key].as_str().expect("a simulator field").to_owned();
    Sim {
        entra: text("entra"),
        graph: text("graph"),
        client_id: text("client_id"),
        client_secret: text("client_secret"),
        tenants: printed["tenants"].clone(),
        _process: process,
    }
}

fn free_ports() -> [u16; 2] {
    let declared = docs::declared_mcp_ports();
    let start = u16::try_from(std::process::id() % 1000).unwrap_or(0);
    let mut free = (0..1000u16)
        .map(|step| 5000 + (start + step) % 1000)
        .filter(|port| !declared.contains(port))
        .filter(|port| std::net::TcpListener::bind(("127.0.0.1", *port)).is_ok());
    [
        free.next().expect("a free MCP port"),
        free.next().expect("a second free MCP port"),
    ]
}

fn edit(path: &Path, edits: &[(&str, &str)]) {
    let mut text = std::fs::read_to_string(path).expect("read a run file");
    for (from, to) in edits {
        assert!(text.contains(from), "{} has no `{from}`", path.display());
        text = text.replacen(from, to, 1);
    }
    std::fs::write(path, text).expect("write a run file");
}

fn write_profile(database_url: &str, api_port: u16, ports: [u16; 2], sim: &Sim) -> PathBuf {
    let dir = globals::repo_root().join(format!(
        "tests/target/m365-sim-profile-{}",
        std::process::id()
    ));
    let profile = globals::write_profile(&dir, database_url);
    let key = dir.join("signing_key.pem");
    RsaSigningKey::generate()
        .and_then(|generated| generated.write_pem_file(&key))
        .expect("write the run's signing key");
    let storage = dir.join("storage");
    std::fs::create_dir_all(&storage).expect("create the run's storage");
    if !storage.join("files").exists() {
        std::os::unix::fs::symlink(
            globals::repo_root().join("storage/files"),
            storage.join("files"),
        )
        .expect("link the shipped storage/files tree");
    }
    let api = format!("http://127.0.0.1:{api_port}");
    let repo_storage = format!("storage: {}/storage", globals::repo_root().display());
    let run_storage = format!("storage: {}", storage.display());
    let signing = format!("signing_key_path: {}", key.display());
    edit(
        &profile,
        &[
            ("port: 8099", &format!("port: {api_port}")),
            (
                "external_url: http://localhost:8099",
                &format!("external_url: {api}"),
            ),
            ("signing_key_path: signing_key.pem", &signing),
            (&repo_storage, &run_storage),
        ],
    );
    let secrets_path = dir.join("secrets.json");
    let mut secrets: Value =
        serde_json::from_str(&std::fs::read_to_string(&secrets_path).expect("read secrets"))
            .expect("the fixture secrets are JSON");
    let allowed = format!("{},{}", sim.tenants["a"], sim.tenants["b"]).replace('"', "");
    for (name, value) in [
        ("microsoft_365_client_id", sim.client_id.as_str()),
        ("microsoft_365_client_secret", sim.client_secret.as_str()),
        ("microsoft_365_authority", sim.entra.as_str()),
        ("microsoft_365_allowed_tenants", allowed.as_str()),
        ("microsoft_365_graph_url", sim.graph.as_str()),
    ] {
        secrets[name] = Value::String(value.to_owned());
    }
    std::fs::write(&secrets_path, secrets.to_string()).expect("write the run's secrets");
    let services = dir.join("services");
    std::fs::write(
        services.join("config/files.yaml"),
        "files:\n  documents:\n    enabled: true\n",
    )
    .expect("switch the document store on");
    edit(
        &services.join("mcp/documents.yaml"),
        &[("port: 5012", &format!("port: {}", ports[0]))],
    );
    edit(
        &services.join("mcp/microsoft-365.yaml"),
        &[
            ("port: 5013", &format!("port: {}", ports[1])),
            ("enabled: false", "enabled: true"),
            (
                "    - custom:microsoft_365_client_id\n",
                "    - custom:microsoft_365_client_id\n    - custom:microsoft_365_graph_url\n",
            ),
        ],
    );
    profile
}

fn child_environment(
    ctx: &AppContext,
    profile: &Path,
    port: u16,
    server: &str,
) -> Vec<(String, String)> {
    let config = ctx
        .mcp_registry()
        .get_server(server)
        .expect("the server is in the run's services config");
    let (tools_json, model_json) =
        spawner::serialize_server_configs(&config).expect("serialise the server's configs");
    spawner::spawn_environment(
        &spawner::SpawnEnvSpec {
            config: &config,
            port,
            system_root: ctx.app_paths().system().root(),
            database_type: &ctx.config().database_type,
            profile_path: profile.to_str().expect("the run's profile path is UTF-8"),
            tools_config_json: &tools_json,
            server_model_config_json: &model_json,
        },
        ctx.config(),
        ctx.secrets(),
        |name| std::env::var(name).ok(),
    )
    .expect("build the environment the API's spawner hands the server")
}

async fn start_binary(ctx: &AppContext, profile: &Path, port: u16, server: &str) -> Child {
    let mut child = Command::new(binary(&format!("systemprompt-mcp-{server}")))
        .env_clear()
        .envs(child_environment(ctx, profile, port, server))
        .env("MCP_SERVICE_ID", server)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .expect("start an MCP binary");
    let deadline = tokio::time::Instant::now() + READY_TIMEOUT;
    loop {
        if let Some(status) = child.try_wait().expect("poll the MCP binary") {
            panic!("the {server} MCP binary exited before it served: {status}");
        }
        let url = format!("http://127.0.0.1:{port}/mcp");
        if reqwest::Client::new().post(url).send().await.is_ok() {
            return child;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "the {server} MCP binary did not serve within {READY_TIMEOUT:?}"
        );
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

// The `mcp_server/microsoft-365` band as services/access-control/rules.yaml
// declares it: closed, and open to every signed-in user.
async fn seed_m365_access(pool: &sqlx::PgPool) {
    sqlx::query(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
         VALUES ('mcp_server', 'microsoft-365', false, 'console:m365-sim')",
    )
    .execute(pool)
    .await
    .expect("register the microsoft-365 server");
    sqlx::query(
        "INSERT INTO access_control_rules
             (id, entity_type, entity_id, rule_type, rule_value, access, justification, source)
         VALUES ($1, 'mcp_server', 'microsoft-365', 'role', 'user', 'allow',
                 'every signed-in user reaches the microsoft-365 server', 'console:m365-sim')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .execute(pool)
    .await
    .expect("open the microsoft-365 server to users");
}

pub(super) async fn seed_person(pool: &sqlx::PgPool, name: &str, email: &str) -> UserId {
    let id = UserId::new(uuid::Uuid::new_v4().to_string());
    sqlx::query(
        "INSERT INTO users (id, name, email, roles, email_verified)
         VALUES ($1, $2, $3, ARRAY['user'], true)",
    )
    .bind(id.as_str())
    .bind(name)
    .bind(email)
    .execute(pool)
    .await
    .expect("seed a person");
    id
}

pub(super) async fn start(pool: &sqlx::PgPool, database_url: &str, sim: Sim) -> Stack {
    docs::seed_system_admin(pool).await;
    docs::seed_documents_access(pool).await;
    seed_m365_access(pool).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the API listener");
    let api_port = listener.local_addr().expect("the API address").port();
    let ports = free_ports();
    let profile = write_profile(database_url, api_port, ports, &sim);
    let ctx = docs::boot(&profile).await;
    ctx.documents()
        .reconcile_folder_entities()
        .await
        .expect("register the folder entities");
    let readiness = Arc::new(systemprompt::api::services::server::ReadinessState::new());
    let router = systemprompt::api::services::server::setup_api_server(&ctx, None, &readiness)
        .expect("the API router builds");
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("the API serves");
    });
    let documents = start_binary(&ctx, &profile, ports[0], DOCUMENTS).await;
    let m365 = start_binary(&ctx, &profile, ports[1], M365).await;
    Stack {
        ctx,
        api: format!("http://127.0.0.1:{api_port}"),
        sim,
        secrets: profile.with_file_name("secrets.json"),
        ports,
        _server: server,
        _binaries: vec![documents, m365],
    }
}

impl Stack {
    pub(super) fn mcp_url(&self, server: &str) -> String {
        let port = if server == DOCUMENTS {
            self.ports[0]
        } else {
            self.ports[1]
        };
        format!("http://127.0.0.1:{port}/mcp")
    }

    // A person signed in on a live session: the bearer the connector routes
    // take, and the MCP caller (bearer plus the hop the API proxy would sign)
    // for one server, on that same session.
    pub(super) async fn sign_in(
        &self,
        pool: &sqlx::PgPool,
        user: &UserId,
        server: &str,
    ) -> McpCaller {
        let session = docs::seed_session(pool, user).await;
        let authenticated = AuthenticatedUser::new_with_roles(
            user.clone(),
            user.as_str().to_owned(),
            format!("{}@m365-sim.test", user.as_str()),
            vec![Permission::User],
            vec!["user".to_owned()],
        );
        let tokens = self.ctx.token_authority();
        let bearer = tokens
            .mint_user_jwt(&UserJwtRequest {
                user: authenticated.clone(),
                session_id: session.clone(),
                jti: AccessTokenId::generate(),
                permissions: vec![Permission::User],
                audience: JwtAudience::standard(),
                ttl: chrono::Duration::hours(1),
                client_id: None,
                purpose: None,
                plugin_id: None,
                interaction: Some(TokenInteraction::Interactive),
                browser_session: true,
                ui_session: false,
                auth_time: None,
            })
            .expect("mint the session token");
        McpCaller {
            bearer,
            tokens,
            server: McpServerId::new(server),
            identity: McpHopIdentity {
                user_id: user.clone(),
                user: Some(authenticated),
                session_id: session,
                user_type: UserType::User,
                client_id: None,
                act_chain: Vec::new(),
                purpose: None,
                interaction: Some(TokenInteraction::Interactive),
                mcp_session: None,
            },
        }
    }
}
