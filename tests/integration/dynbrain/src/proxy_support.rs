//! A fake source for proxy mode: an MCP server on rmcp's streamable HTTP
//! transport that records every tool call with the bearer it came with, and
//! the token accessor core asks for the person's source token. A bearer that
//! names a revoked token gets a 401, as a source refuses an expired one.
//!
//! The proxy Brain under test calls it through core's real MCP client, with
//! a config from the contract boot fixture pointed at the fake server, so the
//! person's token travels the way it does in production (`external_auth`).

use std::collections::BTreeMap;
use std::future::Future;
use std::sync::{Arc, Mutex, PoisonError};

use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::http::request::Parts;
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use dynbrain::brain::Brain;
use dynbrain::brain::operation::Operation;
use dynbrain::brain::proxy::ProxyBrain;
use dynbrain::brain::proxy::source::McpSource;
use dynbrain::config::{ProxyConfig, ToolMapping};
use dynbrain::router::state::BrainState;
use dynbrain::router::{EXTENSION_BASE_PATH, brain_router};
use dynbrain::search::query::EmbedderSlot;
use dynwork_relay::store::postgres::PgRelayStore;
use http_body_util::BodyExt;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, Implementation, ProtocolVersion,
    ServerCapabilities, ServerConfig,
};
use rmcp::service::MaybeSendFuture;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{ErrorData, RoleServer, ServerHandler};
use serde_json::{Value, json};
use systemprompt::config::{LoadedProfile, SecretsService};
use systemprompt::identifiers::{
    Actor, AgentName, ContextId, JwtToken, SessionId, TraceId, UserId,
};
use systemprompt::loader::{BootContext, ServicesState};
use systemprompt::manifest::services::root::ActiveServicesRoot;
use systemprompt::mcp::McpServerConfig;
use systemprompt::mcp::services::client::{McpClient, McpClientLimits};
use systemprompt::mcp::services::deployment::DeploymentService;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::security::credential::TokenCache;
use systemprompt::traits::{ExtensionSecret, SecretLookup};
use tokio::net::TcpListener;
use tower::ServiceExt;

use crate::http_support::authority;

pub(crate) const SERVER: &str = "brain-source";
const ACCESSOR: &str = "/identity/brain-source/token";
const FIXTURE_PROFILE: &str = include_str!("../../../contract/admin/fixtures/profile.yaml");

// One call the source received.
#[derive(Debug, Clone)]
pub(crate) struct Seen {
    pub(crate) tool: String,
    pub(crate) arguments: Value,
    pub(crate) bearer: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Journal(Arc<Mutex<Vec<Seen>>>);

impl Journal {
    fn push(&self, seen: Seen) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(seen);
    }

    pub(crate) fn all(&self) -> Vec<Seen> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub(crate) fn last(&self) -> Seen {
        self.all().pop().expect("the source received a call")
    }
}

#[derive(Debug, Clone)]
struct FakeSource {
    journal: Journal,
}

impl ServerHandler for FakeSource {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2026_07_28)
            .with_server_info(Implementation::new("fake-source", "1.0.0"))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: rmcp::service::RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResponse, ErrorData>> + MaybeSendFuture + '_ {
        let bearer = context
            .extensions
            .get::<Parts>()
            .and_then(|parts| parts.headers.get(header::AUTHORIZATION))
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_owned();
        let tool = request.name.to_string();
        let arguments = Value::Object(request.arguments.unwrap_or_default());
        self.journal.push(Seen {
            tool: tool.clone(),
            arguments: arguments.clone(),
            bearer,
        });
        std::future::ready(Ok(
            CallToolResult::structured(answer(&tool, &arguments)).into()
        ))
    }
}

// What the source answers: its own JSON, the call echoed, plus a document
// with files for `get` and the files' links.
fn answer(tool: &str, arguments: &Value) -> Value {
    let mut out = json!({ "tool": tool, "arguments": arguments });
    match tool {
        "docs_get" => {
            out["article_id"] = json!(7);
            out["markdown"] = json!(
                "# Backups\n![diagram](https://source.test/files/image/41/diagram.png)\n[manual](/files/content/42?download=true)\n![red](img/red.svg)\n![hidden](/files/image/99)\n"
            );
        },
        "docs_asset_list" => {
            out["files"] = json!([
                {"attachment_id": 43, "name": "red.svg", "path": "docs/backups/img/red.svg"},
                {"attachment_id": 44, "name": "notes.pdf", "path": ""}
            ]);
        },
        "docs_asset_link" => {
            let id = arguments["attachment_id"].as_i64().unwrap_or(0);
            if id == 99 {
                return json!({"found": false, "error": "No such file in a documentation you can read."});
            }
            out["found"] = json!(true);
            out["download_url"] = json!(format!("https://source.test/download/{id}?token=t"));
        },
        "docs_request_upload" => {
            out["upload_url"] = json!("https://source.test/upload/abc");
        },
        _ => {},
    }
    out
}

async fn refuse_revoked(request: Request, next: Next) -> Response {
    let revoked = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.contains("revoked"));
    if revoked {
        return (
            StatusCode::UNAUTHORIZED,
            [(
                header::WWW_AUTHENTICATE,
                "Bearer error=\"invalid_token\", error_description=\"the token was revoked\"",
            )],
        )
            .into_response();
    }
    next.run(request).await
}

// The instance's token accessor, as core calls it: the person's JWT in, a
// source token for that person out.
async fn accessor(headers: HeaderMap) -> Response {
    let jwt = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or("")
        .to_owned();
    axum::Json(json!({ "access_token": format!("as-{jwt}") })).into_response()
}

// The broker secret core's client presents to the accessor.
#[derive(Debug)]
struct Broker;

impl SecretLookup for Broker {
    fn lookup(&self, name: &str) -> Option<ExtensionSecret> {
        (name == "mcp_credential_broker_secret").then(|| ExtensionSecret::new("broker".to_owned()))
    }
}

pub(crate) struct Source {
    pub(crate) journal: Journal,
    client: McpClient,
    server: McpServerConfig,
    _dir: tempfile::TempDir,
}

impl Source {
    pub(crate) async fn start() -> Self {
        install_tls();
        let journal = Journal::default();
        let shared = journal.clone();
        let service = StreamableHttpService::new(
            move || {
                Ok(FakeSource {
                    journal: shared.clone(),
                })
            },
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig::default(),
        );
        let routes = Router::new()
            .route_service("/mcp", service)
            .route_layer(middleware::from_fn(refuse_revoked))
            .route(ACCESSOR, get(accessor));
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind the source");
        let base = format!("http://{}", listener.local_addr().expect("its address"));
        tokio::spawn(async move { axum::serve(listener, routes).await });
        let (dir, boot) = boot();
        let mut config = (**boot
            .try_init_config(Some(
                ActiveServicesRoot::baked(boot.profile()).path.as_path(),
            ))
            .expect("config from the fixture profile"))
        .clone();
        config.api_external_url.clone_from(&base);
        config.api_server_url.clone_from(&base);
        let empty = dir.path().join("empty-services.yaml");
        std::fs::write(&empty, "{}").expect("write an empty services file");
        let view = ServicesState::load_from_path(&empty, boot.profile())
            .expect("load the services fixture")
            .view();
        let client = McpClient::new(
            Arc::new(TokenCache::new()),
            DeploymentService::new(view),
            Arc::new(config),
            Arc::new(Broker),
            McpClientLimits::default(),
        );
        let server: McpServerConfig = serde_json::from_value(json!({
            "name": SERVER,
            "owner": "system",
            "server_type": "external",
            "binary": null,
            "enabled": true,
            "display_in_web": false,
            "port": null,
            "crate_path": "/tmp",
            "display_name": "Source",
            "description": "the fake source",
            "capabilities": ["tools"],
            "oauth": {"required": false, "scopes": [], "audience": "mcp", "client_id": null},
            "version": "1.0.0",
            "host": "127.0.0.1",
            "module_name": "mcp",
            "protocol": "mcp",
            "remote_endpoint": format!("{base}/mcp"),
            "external_auth": {"token_endpoint": ACCESSOR, "header": "Authorization", "scheme": "Bearer"}
        }))
        .expect("the source's server config");
        Self {
            journal,
            client,
            server,
            _dir: dir,
        }
    }

    // A proxy Brain with a mapping for `operations`: tool `docs_<name>`,
    // `query` sent as `q` and `synthesize: false` on `answer`.
    pub(crate) fn brain(&self, operations: &[Operation]) -> ProxyBrain {
        let operations: BTreeMap<Operation, ToolMapping> = operations
            .iter()
            .map(|operation| {
                let mut mapping = ToolMapping {
                    tool: format!("docs_{}", operation.name()),
                    ..ToolMapping::default()
                };
                if *operation == Operation::Search {
                    mapping.args.insert("query".to_owned(), "q".to_owned());
                }
                if *operation == Operation::Answer {
                    mapping.fixed.insert("synthesize".to_owned(), json!(false));
                }
                (*operation, mapping)
            })
            .collect();
        ProxyBrain::new(
            Arc::new(McpSource::new(self.client.clone(), self.server.clone())),
            ProxyConfig {
                server: SERVER.to_owned(),
                operations,
                file_urls: vec![
                    "/files/image/{id}".to_owned(),
                    "/files/content/{id}".to_owned(),
                ],
            },
        )
    }

    pub(crate) fn full(&self) -> ProxyBrain {
        self.brain(&Operation::ALL)
    }
}

// The request context of a person whose JWT is `jwt`.
pub(crate) fn person(name: &str, jwt: &str) -> RequestContext {
    RequestContext::new(
        SessionId::generate(),
        TraceId::new("trace".to_owned()),
        ContextId::generate(),
        AgentName::new("brain-tests"),
        Actor::user(UserId::new(name.to_owned())),
    )
    .with_user(systemprompt::models::auth::AuthenticatedUser::new(
        UserId::new(name.to_owned()),
        name.to_owned(),
        format!("{name}@example.test"),
        vec![systemprompt::models::auth::Permission::User],
    ))
    .with_auth_token(JwtToken::new(jwt))
}

pub(crate) fn user(name: &str) -> UserId {
    UserId::new(name.to_owned())
}

// The REST routes over a proxy Brain; the desktop token opens them and needs
// no database.
pub(crate) fn router(brain: ProxyBrain) -> Router {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://unused:unused@127.0.0.1:1/unused")
        .expect("a lazy pool");
    let brain: Arc<dyn Brain> = Arc::new(brain);
    let state = BrainState {
        brain,
        authority: authority(),
        devices: Arc::new(PgRelayStore::new(Arc::new(pool))),
        embedder: EmbedderSlot::default(),
    };
    Router::new().nest(EXTENSION_BASE_PATH, brain_router(state))
}

pub(crate) struct Reply {
    pub(crate) status: StatusCode,
    pub(crate) location: Option<String>,
    pub(crate) body: Value,
}

pub(crate) async fn call(
    router: &Router,
    method: Method,
    path: &str,
    token: &str,
    body: Option<&Value>,
) -> Reply {
    let mut request = axum::http::Request::builder()
        .method(method)
        .uri(format!("{EXTENSION_BASE_PATH}{path}"))
        .header(header::AUTHORIZATION, format!("Bearer {token}"));
    if body.is_some() {
        request = request.header(header::CONTENT_TYPE, "application/json");
    }
    let request = request
        .body(Body::from(body.map(Value::to_string).unwrap_or_default()))
        .expect("build a request");
    let response = router
        .clone()
        .oneshot(request)
        .await
        .expect("the router answers");
    let status = response.status();
    let location = response
        .headers()
        .get(header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("read the body")
        .to_bytes();
    Reply {
        status,
        location,
        body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    }
}

fn boot() -> (tempfile::TempDir, BootContext) {
    let dir = tempfile::tempdir().expect("profile directory");
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let yaml = FIXTURE_PROFILE
        .replace("__PROFILE_DIR__", &dir.path().to_string_lossy())
        .replace("__REPO__", &repo.to_string_lossy());
    std::fs::create_dir_all(dir.path().join("bin")).expect("bin directory");
    std::os::unix::fs::symlink(repo.join("services"), dir.path().join("services"))
        .expect("link the repository services tree");
    let path = dir.path().join("profile.yaml");
    std::fs::write(&path, yaml).expect("fixture profile");
    let profile = LoadedProfile::from_path(&path).expect("load the fixture profile");
    let document = json!({
        "database_url": "postgres://unused:unused@localhost:5432/postgres",
        "oauth_at_rest_pepper": "unit-fixture-pepper-not-a-real-secret",
        "manifest_signing_secret_seed": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "encryption_master_key":
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    });
    let secrets = serde_json::from_value(document).expect("fixture secrets document");
    let secrets = SecretsService::from_secrets(Arc::clone(&profile), secrets);
    (dir, BootContext::new(profile, secrets))
}

// Why: reqwest's rustls has no default provider; core installs ring at
// server start, which this suite never runs.
fn install_tls() {
    if rustls::crypto::ring::default_provider()
        .install_default()
        .is_err()
    {
        // already installed by an earlier test in this process
    }
}
