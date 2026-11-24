//! The installation the broker suite drives: a profile over the run's
//! throwaway database with the document store on and the `artifacts-fake`
//! MCP server declared, the API served on the loopback port the profile
//! names (the broker reaches core's MCP proxy there), and the fake server
//! hosted in-process by core's toolset host, booted from the environment
//! core's spawner hands a server process and registered as running, so the
//! proxy forwards to it exactly as it forwards to a spawned server.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::Value;
use systemprompt::database::{CreateServiceInput, ServiceModule, ServiceStatus};
use systemprompt::identifiers::{AccessTokenId, DocumentId, McpServerId, ServiceName, UserId};
use systemprompt::mcp::services::process::spawner;
use systemprompt::models::auth::{AuthenticatedUser, JwtAudience, Permission, UserJwtRequest};
use systemprompt::system::{AppContext, HostContext};
use systemprompt_security::keys::RsaSigningKey;

use crate::docs_smoke::stack as docs;
use crate::globals;

pub(super) const SERVER: &str = "artifacts-fake";
const FAKE_SERVER_YAML: &str = include_str!("../../fixtures/artifacts_fake.yaml");
const FAKE_INCLUDE: &str = "  - ../mcp/artifacts_fake.yaml\n";

pub(super) struct Stack {
    pub(super) ctx: Arc<AppContext>,
    pub(super) api: String,
    pub(super) origin: String,
    _api: tokio::task::JoinHandle<()>,
    _fake: tokio::task::JoinHandle<()>,
}

pub(super) struct Reply {
    pub(super) status: reqwest::StatusCode,
    pub(super) body: String,
}

impl Reply {
    pub(super) fn json(&self) -> Value {
        serde_json::from_str(&self.body)
            .unwrap_or_else(|e| panic!("{} is not JSON ({e}): {}", self.status, self.body))
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

fn write_profile(database_url: &str, api_port: u16, ports: [u16; 2]) -> PathBuf {
    let dir = globals::repo_root().join(format!(
        "tests/target/artifacts-broker-profile-{}",
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
    let repo_storage = format!("storage: {}/storage", globals::repo_root().display());
    let run_storage = format!("storage: {}", storage.display());
    let signing = format!("signing_key_path: {}", key.display());
    edit(
        &profile,
        &[
            ("port: 8099", &format!("port: {api_port}")),
            ("signing_key_path: signing_key.pem", &signing),
            (&repo_storage, &run_storage),
        ],
    );
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
    std::fs::write(
        services.join("mcp/artifacts_fake.yaml"),
        FAKE_SERVER_YAML.replace("__PORT__", &ports[1].to_string()),
    )
    .expect("write the fake server's YAML");
    let anchor = "  - ../mcp/dynbrain.yaml\n";
    edit(
        &services.join("config/config.yaml"),
        &[(anchor, &format!("{anchor}{FAKE_INCLUDE}"))],
    );
    profile
}

fn server_environment(ctx: &AppContext, profile: &Path, port: u16) -> Vec<(String, String)> {
    let config = ctx
        .mcp_registry()
        .get_server(SERVER)
        .expect("the fake server is in the run's services config");
    let (tools_json, model_json) =
        spawner::serialize_server_configs(&config).expect("serialise the server's configs");
    let mut environment = spawner::spawn_environment(
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
    .expect("build the environment the API's spawner hands the server");
    environment.push(("MCP_SERVICE_ID".to_owned(), SERVER.to_owned()));
    environment
}

async fn start_fake(ctx: &AppContext, profile: &Path, port: u16) -> tokio::task::JoinHandle<()> {
    let environment = server_environment(ctx, profile, port);
    let host = HostContext::boot(move |name| {
        environment
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
    })
    .await
    .expect("the fake server's host context boots");
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .expect("bind the fake server's port");
    let fake = tokio::spawn(async move {
        systemprompt::system::mcp_host::serve_on(
            Arc::new(host),
            McpServerId::new(SERVER),
            listener,
        )
        .await
        .expect("the fake server serves");
    });
    ctx.service_repository()
        .create_service(CreateServiceInput {
            name: &ServiceName::new(SERVER),
            module_name: ServiceModule::Mcp,
            status: ServiceStatus::Running,
            port,
            binary_mtime: None,
        })
        .await
        .expect("register the fake server as running");
    fake
}

// The `mcp_server/artifacts-fake` band, as `config apply` writes one for a
// server declared closed and open to every signed-in user.
async fn seed_fake_access(pool: &sqlx::PgPool) {
    sqlx::query(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
         VALUES ('mcp_server', $1, false, 'console:artifacts-broker')",
    )
    .bind(SERVER)
    .execute(pool)
    .await
    .expect("register the fake server");
    sqlx::query(
        "INSERT INTO access_control_rules
             (id, entity_type, entity_id, rule_type, rule_value, access, justification, source)
         VALUES ($1, 'mcp_server', $2, 'role', 'user', 'allow',
                 'every signed-in user reaches the fake server', 'console:artifacts-broker')",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(SERVER)
    .execute(pool)
    .await
    .expect("open the fake server to users");
}

pub(super) async fn start(pool: &sqlx::PgPool, database_url: &str) -> Stack {
    docs::seed_system_admin(pool).await;
    seed_fake_access(pool).await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind the API listener");
    let api_port = listener.local_addr().expect("the API address").port();
    let ports = free_ports();
    let profile = write_profile(database_url, api_port, ports);
    let ctx = docs::boot(&profile).await;
    // Why: the server's run loop registers the document folders, the root
    // among them, as access-control entities before it serves; the router
    // alone does not, and the root's rule decides who may create there.
    ctx.documents()
        .reconcile_folder_entities()
        .await
        .expect("register the folder entities");
    let readiness = Arc::new(systemprompt::api::services::server::ReadinessState::new());
    let router = systemprompt::api::services::server::setup_api_server(&ctx, None, &readiness)
        .expect("the API router builds");
    let api = tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .expect("the API serves");
    });
    let fake = start_fake(&ctx, &profile, ports[1]).await;
    let origin = url::Url::parse(&ctx.config().api_external_url)
        .expect("the run's external URL parses")
        .origin()
        .ascii_serialization();
    Stack {
        ctx,
        api: format!("http://127.0.0.1:{api_port}"),
        origin,
        _api: api,
        _fake: fake,
    }
}

impl Stack {
    // The web login's session as the browser holds it: a live session row
    // and a `browser_session` token the runtime mints with the run's key.
    pub(super) async fn session_cookie(&self, pool: &sqlx::PgPool, user: &UserId) -> String {
        let session = docs::seed_session(pool, user).await;
        let token = self
            .ctx
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
                ui_session: false,
                auth_time: Some(chrono::Utc::now().timestamp()),
            })
            .expect("mint the session token");
        format!("access_token={token}")
    }

    pub(super) async fn owned_document(&self, owner: &UserId) -> DocumentId {
        use systemprompt::files::documents::agents::{DocumentActor, act_as};
        use systemprompt::files::documents::models::{RevisionAttribution, RevisionOrigin};
        use systemprompt::files::documents::service::{Author, CreateRequest};
        use systemprompt::identifiers::{DocumentTemplateId, TraceId};
        use systemprompt::models::documents::ActorKind;

        let caller = AuthenticatedUser::new(
            owner.clone(),
            owner.as_str().to_owned(),
            format!("{}@contract.test", owner.as_str()),
            vec![Permission::User],
        );
        let service = self.ctx.documents().service();
        let trace = TraceId::generate();
        let created = service.create_from_template(
            &caller,
            CreateRequest {
                template_id: DocumentTemplateId::new("dynapps-word"),
                folder: None,
                name: "artifacts-broker".to_owned(),
                params: systemprompt_provider_contracts::documents::TemplateParams::default(),
                ops: Vec::new(),
                author: Author {
                    actor_kind: ActorKind::User,
                    origin: RevisionOrigin::Api,
                    attribution: RevisionAttribution::default(),
                },
                message: None,
                content: None,
                media: systemprompt::files::documents::model::Media::new(),
            },
            &trace,
        );
        let (document, _) = Box::pin(act_as(DocumentActor::Human, created))
            .await
            .expect("seed the owner's document");
        document.id
    }

    // One broker call as the Drive page makes it: the session cookie, from
    // the site's own page.
    pub(super) async fn call(
        &self,
        cookie: Option<&str>,
        document: &DocumentId,
        body: &Value,
    ) -> Reply {
        let mut request = reqwest::Client::new()
            .post(format!(
                "{}/api/v1/artifacts/{}/mcp/call",
                self.api,
                document.as_str()
            ))
            .header("origin", &self.origin)
            .header("sec-fetch-site", "same-origin")
            .json(body);
        if let Some(cookie) = cookie {
            request = request.header("cookie", cookie);
        }
        let response = request.send().await.expect("reach the API");
        let status = response.status();
        let body = response.text().await.expect("read the API response");
        Reply { status, body }
    }
}
