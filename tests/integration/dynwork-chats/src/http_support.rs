//! The chats router as core mounts it, beside a stand-in for the relay's
//! `/api/dynwork` mount, over the memory store or Postgres, with a real RS256
//! token authority, a stepping clock and a device check that is either a fake
//! (memory) or the relay's own device table (Postgres), and a scripted model
//! in place of the instance's AI service.

use std::collections::HashSet;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, LazyLock, Mutex, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use axum::response::Response;
use axum::routing::get;
use chrono::{DateTime, Utc};
use dynapps_test_common::TempDb;
use dynwork_chats::auth::{DeviceCheck, RelayDevices};
use dynwork_chats::clock::Clock;
use dynwork_chats::reply::registry::Replies;
use dynwork_chats::store::ChatStore;
use dynwork_chats::store::memory::MemoryChatStore;
use dynwork_chats::store::postgres::PgChatStore;
use dynwork_chats::{ChatsState, EXTENSION_BASE_PATH, Limits, Shared, chats_router};
use dynwork_relay::ids::DeviceId;
use dynwork_relay::mobile_auth::jwt;
use dynwork_relay::store::postgres::PgRelayStore;
use dynwork_relay::store::{Account, RelayStore, RevokeOutcome, StoreError};
use http_body_util::BodyExt;
use serde_json::Value;
use systemprompt::identifiers::{AccessTokenId, ClientId, SessionId, UserId};
use systemprompt::models::auth::{
    AuthenticatedUser, JwtAudience, Permission, TokenAuthority, UserJwtRequest,
};
use systemprompt_security::jwt::DeploymentTokenAuthority;
use systemprompt_security::keys::{RsaSigningKey, SigningKeys};
use tower::ServiceExt;

use crate::fixture::{at, seed_users};
use crate::hints::RecordingHints;
use crate::reply_support::{FakeModel, FakeSource};

pub(crate) const ISSUER: &str = "https://chats.test";
// The suites' `/log` heartbeat and model idle limit: short, so a revoked phone
// and a stalled model show within a test's wait.
pub(crate) const HEARTBEAT: Duration = Duration::from_millis(200);
pub(crate) const REPLY_IDLE: Duration = Duration::from_secs(2);
pub(crate) const RELAY_PING: &str = "/api/dynwork/relay/v1/ping";

fn authority_with_seed(seed: u8) -> Arc<DeploymentTokenAuthority> {
    let key = RsaSigningKey::generate().expect("generate an RSA key");
    let keys = SigningKeys::new(key, &[seed; 32]).expect("build signing keys");
    Arc::new(DeploymentTokenAuthority::new(keys, ISSUER))
}

static AUTHORITY: LazyLock<Arc<DeploymentTokenAuthority>> =
    LazyLock::new(|| authority_with_seed(11));
static FOREIGN: LazyLock<Arc<DeploymentTokenAuthority>> = LazyLock::new(|| authority_with_seed(12));

fn authority() -> Arc<dyn TokenAuthority> {
    let authority: Arc<DeploymentTokenAuthority> = Arc::clone(&AUTHORITY);
    authority
}

pub(crate) fn user(name: &str) -> UserId {
    UserId::try_new(name).expect("a test user id")
}

fn mint(
    authority: &dyn TokenAuthority,
    name: &str,
    permissions: Vec<Permission>,
    audience: Vec<JwtAudience>,
) -> String {
    mint_for_client(authority, name, permissions, audience, None)
}

fn mint_for_client(
    authority: &dyn TokenAuthority,
    name: &str,
    permissions: Vec<Permission>,
    audience: Vec<JwtAudience>,
    client_id: Option<ClientId>,
) -> String {
    authority
        .mint_user_jwt(&UserJwtRequest {
            user: AuthenticatedUser::new(
                user(name),
                name.to_owned(),
                format!("{name}@example.test"),
                permissions.clone(),
            ),
            session_id: SessionId::generate(),
            jti: AccessTokenId::generate(),
            permissions,
            audience,
            ttl: chrono::Duration::minutes(30),
            client_id,
            purpose: None,
            plugin_id: None,
            interaction: None,
            browser_session: false,
        })
        .expect("mint a token")
}

// A desktop JWT that expires `ttl` from now.
pub(crate) fn desktop_token_expiring(name: &str, ttl: chrono::Duration) -> String {
    let permissions = vec![Permission::User];
    AUTHORITY
        .mint_user_jwt(&UserJwtRequest {
            user: AuthenticatedUser::new(
                user(name),
                name.to_owned(),
                format!("{name}@example.test"),
                permissions.clone(),
            ),
            session_id: SessionId::generate(),
            jti: AccessTokenId::generate(),
            permissions,
            audience: vec![JwtAudience::Bridge, JwtAudience::Mcp],
            ttl,
            client_id: None,
            purpose: None,
            plugin_id: None,
            interaction: None,
            browser_session: false,
        })
        .expect("mint a short-lived desktop token")
}

// The phone's JWT for `device` as the relay mints it, but expiring `ttl` from
// now instead of in an hour.
pub(crate) fn mobile_token_expiring(name: &str, device: DeviceId, ttl: chrono::Duration) -> String {
    let permissions = vec![Permission::User];
    AUTHORITY
        .mint_user_jwt(&UserJwtRequest {
            user: AuthenticatedUser::new(
                user(name),
                name.to_owned(),
                format!("{name}@example.test"),
                permissions.clone(),
            ),
            session_id: SessionId::new(device.to_string()),
            jti: AccessTokenId::generate(),
            permissions,
            audience: vec![JwtAudience::Resource(jwt::MOBILE_AUDIENCE.to_owned())],
            ttl,
            client_id: None,
            purpose: Some(jwt::MOBILE_PURPOSE.to_owned()),
            plugin_id: None,
            interaction: None,
            browser_session: false,
        })
        .expect("mint a short-lived mobile token")
}

// The DynWork desktop's JWT, as core's bridge device link mints it.
pub(crate) fn desktop_token(name: &str) -> String {
    mint(
        AUTHORITY.as_ref(),
        name,
        vec![Permission::User],
        vec![JwtAudience::Bridge, JwtAudience::Mcp],
    )
}

// A desktop JWT that names an OAuth client.
pub(crate) fn desktop_token_for_client(name: &str, client: &str) -> String {
    mint_for_client(
        AUTHORITY.as_ref(),
        name,
        vec![Permission::User],
        vec![JwtAudience::Bridge, JwtAudience::Mcp],
        Some(ClientId::new(client)),
    )
}

pub(crate) fn token_with_audience(name: &str, audience: JwtAudience) -> String {
    mint(
        AUTHORITY.as_ref(),
        name,
        vec![Permission::User],
        vec![audience],
    )
}

pub(crate) fn foreign_token(name: &str) -> String {
    mint(
        FOREIGN.as_ref(),
        name,
        vec![Permission::User],
        vec![JwtAudience::Bridge],
    )
}

pub(crate) fn anonymous_token(name: &str) -> String {
    mint(
        AUTHORITY.as_ref(),
        name,
        vec![Permission::Anonymous],
        JwtAudience::standard(),
    )
}

// The phone's JWT for `device`, minted by the relay's own sign-in code.
pub(crate) fn mobile_token(name: &str, device: DeviceId) -> String {
    let account = Account {
        user: user(name),
        name: name.to_owned(),
        email: format!("{name}@example.test"),
        roles: Vec::new(),
    };
    jwt::mint(AUTHORITY.as_ref(), &account, device)
        .expect("mint a mobile token")
        .token
}

#[derive(Debug)]
struct StepClock(AtomicI64);

impl Clock for StepClock {
    fn now(&self) -> DateTime<Utc> {
        at(self.0.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Debug, Default)]
struct FakeDevices(Mutex<HashSet<(String, DeviceId)>>);

#[async_trait]
impl DeviceCheck for FakeDevices {
    async fn phone_is_live(&self, user: &UserId, device: DeviceId) -> Result<bool, StoreError> {
        let live = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        Ok(live.contains(&(user.as_str().to_owned(), device)))
    }
}

enum Phones {
    Fake(Arc<FakeDevices>),
    Relay(TempDb),
}

pub(crate) struct App {
    router: Router,
    pub(crate) store: Arc<dyn ChatStore>,
    pub(crate) state: ChatsState,
    pub(crate) model: Arc<FakeModel>,
    pub(crate) source: Arc<FakeSource>,
    pub(crate) hints: Arc<RecordingHints>,
    phones: Phones,
}

fn mount(store: Arc<dyn ChatStore>, devices: Arc<dyn DeviceCheck>, phones: Phones) -> App {
    let model = Arc::new(FakeModel::default());
    let source = Arc::new(FakeSource::new(Arc::clone(&model)));
    let hints = Arc::new(RecordingHints::default());
    let state = ChatsState::new(Shared {
        store: Arc::clone(&store),
        authority: authority(),
        devices,
        clock: Arc::new(StepClock(AtomicI64::new(0))),
        models: Arc::<FakeSource>::clone(&source),
        hints: Arc::<RecordingHints>::clone(&hints),
        replies: Replies::default(),
        limits: Limits {
            heartbeat: HEARTBEAT,
            reply_idle: REPLY_IDLE,
            ..Limits::default()
        },
    });
    let relay = Router::new().route("/relay/v1/ping", get(|| async { "relay" }));
    let router = Router::new()
        .nest("/api/dynwork", relay)
        .nest(EXTENSION_BASE_PATH, chats_router(state.clone()));
    App {
        router,
        store,
        state,
        model,
        source,
        hints,
        phones,
    }
}

pub(crate) fn memory() -> App {
    let store: Arc<dyn ChatStore> = Arc::new(MemoryChatStore::new());
    let fake = Arc::new(FakeDevices::default());
    let devices: Arc<dyn DeviceCheck> = Arc::<FakeDevices>::clone(&fake);
    mount(store, devices, Phones::Fake(fake))
}

pub(crate) async fn postgres(db: TempDb) -> App {
    seed_users(&db).await;
    let store: Arc<dyn ChatStore> = Arc::new(PgChatStore::new(Arc::clone(&db.pool)));
    let devices: Arc<dyn DeviceCheck> = Arc::new(RelayDevices::new(Arc::clone(&db.pool)));
    mount(store, devices, Phones::Relay(db))
}

pub(crate) struct Answer {
    pub(crate) status: StatusCode,
    pub(crate) headers: HeaderMap,
    pub(crate) body: Value,
}

impl Answer {
    pub(crate) fn key(&self) -> &str {
        self.body["error"]["key"].as_str().unwrap_or("")
    }
}

pub(crate) struct Call<'a> {
    app: &'a App,
    request: axum::http::request::Builder,
    body: String,
}

impl Call<'_> {
    pub(crate) fn bearer(self, token: &str) -> Self {
        self.header(header::AUTHORIZATION.as_str(), &format!("Bearer {token}"))
    }

    pub(crate) fn header(mut self, name: &str, value: &str) -> Self {
        self.request = self.request.header(name, value);
        self
    }

    pub(crate) fn if_match(self, revision: &str) -> Self {
        self.header(header::IF_MATCH.as_str(), revision)
    }

    pub(crate) fn json(mut self, body: &Value) -> Self {
        self.body = body.to_string();
        self.header(header::CONTENT_TYPE.as_str(), "application/json")
    }

    pub(crate) fn raw(mut self, body: &str) -> Self {
        self.body = body.to_owned();
        self
    }

    // The response as it arrives, for a streamed body such as `/log`.
    pub(crate) async fn open(self) -> Response {
        let request = self
            .request
            .body(Body::from(self.body))
            .expect("build a request");
        self.app
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("the router answers")
    }

    pub(crate) async fn send(self) -> Answer {
        let response = self.open().await;
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("read the body")
            .to_bytes();
        let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        Answer {
            status,
            headers,
            body,
        }
    }
}

impl App {
    pub(crate) fn call(&self, method: Method, path: &str) -> Call<'_> {
        let uri = if path.starts_with("/api/") {
            path.to_owned()
        } else {
            format!("{EXTENSION_BASE_PATH}{path}")
        };
        Call {
            app: self,
            request: Request::builder().method(method).uri(uri),
            body: String::new(),
        }
    }

    // A signed-in phone of `name`: a live device row and its mobile JWT.
    pub(crate) async fn phone(&self, name: &str) -> (DeviceId, String) {
        let device = DeviceId::generate();
        match &self.phones {
            Phones::Fake(fake) => {
                fake.0
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .insert((name.to_owned(), device));
            },
            Phones::Relay(db) => {
                sqlx::query(
                    "INSERT INTO dynwork_relay_devices (id, user_id, kind, peer_id, name, credential_hash)
                     VALUES ($1, $2, 'phone', $3, 'test phone', $4)",
                )
                .bind(device.to_string())
                .bind(name)
                .bind(device.as_uuid().simple().to_string())
                .bind(format!("hash-{device}"))
                .execute(&*db.pool)
                .await
                .expect("seed a phone device");
            },
        }
        (device, mobile_token(name, device))
    }

    pub(crate) async fn revoke(&self, name: &str, device: DeviceId) {
        match &self.phones {
            Phones::Fake(fake) => {
                fake.0
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .remove(&(name.to_owned(), device));
            },
            Phones::Relay(db) => {
                let outcome = PgRelayStore::new(Arc::clone(&db.pool))
                    .revoke_device(&user(name), device, at(0))
                    .await
                    .expect("revoke the phone");
                assert!(matches!(outcome, RevokeOutcome::Revoked), "{outcome:?}");
            },
        }
    }

    pub(crate) async fn finish(self) {
        let drained = self
            .state
            .replies
            .tasks()
            .drain(Duration::from_secs(10))
            .await;
        assert!(drained.is_drained(), "a reply task outlived its test");
        if let Phones::Relay(db) = self.phones {
            db.cleanup().await;
        }
    }
}

macro_rules! http_suite {
    ($($name:ident),* $(,)?) => {
        mod memory_store {
            $(
                #[tokio::test]
                async fn $name() {
                    let app = crate::http_support::memory();
                    super::$name(&app).await;
                    app.finish().await;
                }
            )*
        }

        mod postgres_store {
            $(
                #[tokio::test]
                async fn $name() {
                    let db = dynapps_test_common::db_or_skip!();
                    let app = crate::http_support::postgres(db).await;
                    super::$name(&app).await;
                    app.finish().await;
                }
            )*
        }
    };
}

pub(crate) use http_suite;
