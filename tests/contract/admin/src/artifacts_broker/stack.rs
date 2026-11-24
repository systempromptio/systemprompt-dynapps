//! The installation the broker suite drives: a profile over the run's
//! throwaway database with the document store on, the `artifacts-fake` MCP
//! server declared and a live-pages policy of the suite's own; the API served
//! on the loopback port the profile names (the broker reaches core's MCP
//! proxy there); and two servers hosted in-process, booted from the
//! environment core's spawner hands a server process and registered as
//! running: the fake, by core's toolset host, and the extension's own
//! `documents` server, by the same router its binary serves.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::Value;
use systemprompt::database::{CreateServiceInput, ServiceModule, ServiceStatus};
use systemprompt::identifiers::{AccessTokenId, DocumentId, McpServerId, ServiceName, UserId};
use systemprompt::mcp::services::process::spawner;
use systemprompt::models::auth::{
    AuthenticatedUser, JwtAudience, McpHopIdentity, Permission, TokenInteraction, UserJwtRequest,
    UserType,
};
use systemprompt::system::{AppContext, HostContext};
use systemprompt_security::keys::RsaSigningKey;

use crate::docs_smoke::mcp::{McpCaller, McpClient};
use crate::docs_smoke::stack as docs;
use crate::globals;

pub(super) const SERVER: &str = "artifacts-fake";
pub(super) const DOCUMENTS: &str = "documents";
const FAKE_SERVER_YAML: &str = include_str!("../../fixtures/artifacts_fake.yaml");
const FAKE_INCLUDE: &str = "  - ../mcp/artifacts_fake.yaml\n";
// The suite's operator policy: the fake's `off` tool and every other server
// are left out, so a page may not ask for them.
const LIVE_PAGES_POLICY: &str = "live_pages:\n  servers:\n    artifacts-fake:\n      tools:\n      \
     - artifacts_fake_echo\n      - artifacts_fake_secret\n      - artifacts_fake_write\n";

pub(super) struct Stack {
    pub(super) ctx: Arc<AppContext>,
    pub(super) api: String,
    pub(super) origin: String,
    documents_mcp: String,
    _api: tokio::task::JoinHandle<()>,
    _servers: [tokio::task::JoinHandle<()>; 2],
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

    pub(super) fn key(&self) -> String {
        self.json()["error_key"]
            .as_str()
            .unwrap_or_else(|| panic!("no error_key in {}: {}", self.status, self.body))
            .to_owned()
    }
}

// Who a request presents itself as.
#[derive(Clone, Copy)]
pub(super) enum As<'a> {
    Nobody,
    Cookie(&'a str),
    Bearer(&'a str),
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
    std::fs::write(
        services.join("documents/live-pages.yaml"),
        LIVE_PAGES_POLICY,
    )
    .expect("write the suite's live-pages policy");
    let anchor = "  - ../mcp/dynbrain.yaml\n";
    edit(
        &services.join("config/config.yaml"),
        &[(anchor, &format!("{anchor}{FAKE_INCLUDE}"))],
    );
    profile
}

async fn host(ctx: &AppContext, profile: &Path, server: &str, port: u16) -> HostContext {
    let config = ctx
        .mcp_registry()
        .get_server(server)
        .expect("the server is in the run's services config");
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
    environment.push(("MCP_SERVICE_ID".to_owned(), server.to_owned()));
    HostContext::boot(move |name| {
        environment
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
    })
    .await
    .expect("the server's host context boots")
}

async fn register_running(ctx: &AppContext, server: &str, port: u16) {
    ctx.service_repository()
        .create_service(CreateServiceInput {
            name: &ServiceName::new(server),
            module_name: ServiceModule::Mcp,
            status: ServiceStatus::Running,
            port,
            binary_mtime: None,
        })
        .await
        .expect("register the server as running");
}

async fn start_fake(ctx: &AppContext, profile: &Path, port: u16) -> tokio::task::JoinHandle<()> {
    let host = host(ctx, profile, SERVER, port).await;
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
    register_running(ctx, SERVER, port).await;
    fake
}

async fn start_documents(
    ctx: &AppContext,
    profile: &Path,
    port: u16,
) -> tokio::task::JoinHandle<()> {
    let host = host(ctx, profile, DOCUMENTS, port).await;
    let router = systemprompt_mcp_documents::router(&host, &McpServerId::new(DOCUMENTS))
        .expect("the documents server's router builds");
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .expect("bind the documents server's port");
    let served = tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("the documents server serves");
    });
    register_running(ctx, DOCUMENTS, port).await;
    served
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
    docs::seed_documents_access(pool).await;
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
    let documents = start_documents(&ctx, &profile, ports[0]).await;
    let fake = start_fake(&ctx, &profile, ports[1]).await;
    let origin = url::Url::parse(&ctx.config().api_external_url)
        .expect("the run's external URL parses")
        .origin()
        .ascii_serialization();
    Stack {
        ctx,
        api: format!("http://127.0.0.1:{api_port}"),
        origin,
        documents_mcp: format!("http://127.0.0.1:{}/mcp", ports[0]),
        _api: api,
        _servers: [documents, fake],
    }
}

fn user(id: &UserId) -> AuthenticatedUser {
    AuthenticatedUser::new_with_roles(
        id.clone(),
        id.as_str().to_owned(),
        format!("{}@contract.test", id.as_str()),
        vec![Permission::User],
        vec!["user".to_owned()],
    )
}

impl Stack {
    // A token over a live session row, minted with the run's key: the web
    // login's `browser_session` one, or an attended agent's.
    async fn token(&self, pool: &sqlx::PgPool, id: &UserId, browser: bool) -> String {
        let session = docs::seed_session(pool, id).await;
        self.ctx
            .token_authority()
            .mint_user_jwt(&UserJwtRequest {
                user: user(id),
                session_id: session,
                jti: AccessTokenId::generate(),
                permissions: vec![Permission::User],
                audience: JwtAudience::standard(),
                ttl: chrono::Duration::hours(1),
                client_id: None,
                purpose: None,
                plugin_id: None,
                interaction: (!browser).then_some(TokenInteraction::Interactive),
                browser_session: browser,
                ui_session: false,
                auth_time: browser.then(|| chrono::Utc::now().timestamp()),
            })
            .expect("mint the token")
    }

    pub(super) async fn session_cookie(&self, pool: &sqlx::PgPool, id: &UserId) -> String {
        format!("access_token={}", self.token(pool, id, true).await)
    }

    pub(super) async fn agent_bearer(&self, pool: &sqlx::PgPool, id: &UserId) -> String {
        self.token(pool, id, false).await
    }

    // An attended agent on the `documents` server, as the proxy hands it on:
    // the agent's bearer and a hop token naming the same person.
    pub(super) async fn agent(&self, pool: &sqlx::PgPool, id: &UserId) -> McpClient {
        let bearer = self.agent_bearer(pool, id).await;
        let session = docs::seed_session(pool, id).await;
        let caller = McpCaller {
            bearer,
            tokens: self.ctx.token_authority(),
            server: McpServerId::new(DOCUMENTS),
            identity: McpHopIdentity {
                user_id: id.clone(),
                user: Some(user(id)),
                session_id: session,
                user_type: UserType::User,
                client_id: None,
                act_chain: Vec::new(),
                purpose: None,
                interaction: Some(TokenInteraction::Interactive),
                mcp_session: None,
            },
        };
        McpClient::connect(&self.documents_mcp, caller).await
    }

    pub(super) async fn owned_document(&self, owner: &UserId, template: &str) -> DocumentId {
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
                template_id: DocumentTemplateId::new(template),
                folder: None,
                name: format!("artifacts-broker-{template}"),
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

    // One request as the shell makes it: from the site's own page, with
    // whichever credential `who` names.
    pub(super) async fn send(
        &self,
        method: reqwest::Method,
        path: &str,
        who: As<'_>,
        body: Option<&Value>,
    ) -> Reply {
        let mut request = reqwest::Client::new()
            .request(method, format!("{}{path}", self.api))
            .header("origin", &self.origin)
            .header("sec-fetch-site", "same-origin");
        request = match who {
            As::Nobody => request,
            As::Cookie(cookie) => request.header("cookie", cookie),
            As::Bearer(bearer) => request.bearer_auth(bearer),
        };
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().await.expect("reach the API");
        let status = response.status();
        let body = response.text().await.expect("read the API response");
        Reply { status, body }
    }
}
