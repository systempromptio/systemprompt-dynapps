//! The relay served over loopback TCP, with an in-memory store, a manual
//! clock, a recording audit sink and a real RS256 token authority.

use std::net::SocketAddr;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use chrono::{DateTime, Utc};
use dynwork_relay::audit::{AuditEvent, AuditSink};
use dynwork_relay::clock::Clock;
use dynwork_relay::ids::PeerId;
use dynwork_relay::ws::PEER_HEADER;
use dynwork_relay::{BASE_PATH, EXTENSION_BASE_PATH, RelayLimits, RelayState, extension_router};
use futures_util::{SinkExt, StreamExt};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use systemprompt::identifiers::{AccessTokenId, PluginId, SessionId, UserId};
use systemprompt::models::auth::{
    AuthenticatedUser, JwtAudience, Permission, TokenAuthority, UserJwtRequest,
};
use systemprompt::traits::BackgroundTasks;
use systemprompt_security::jwt::DeploymentTokenAuthority;
use systemprompt_security::keys::{RsaSigningKey, SigningKeys};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tower::ServiceExt;

use crate::memory_store::MemoryStore;

pub(crate) type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub(crate) const ISSUER: &str = "https://relay.test";

fn authority_with_seed(seed: u8) -> Arc<DeploymentTokenAuthority> {
    let key = RsaSigningKey::generate().expect("generate an RSA key");
    let keys = SigningKeys::new(key, &[seed; 32]).expect("build signing keys");
    Arc::new(DeploymentTokenAuthority::new(keys, ISSUER))
}

static AUTHORITY: LazyLock<Arc<DeploymentTokenAuthority>> =
    LazyLock::new(|| authority_with_seed(7));
static FOREIGN: LazyLock<Arc<DeploymentTokenAuthority>> = LazyLock::new(|| authority_with_seed(8));

pub(crate) fn authority() -> Arc<dyn TokenAuthority> {
    let authority: Arc<DeploymentTokenAuthority> = Arc::clone(&AUTHORITY);
    authority
}

pub(crate) fn user(name: &str) -> UserId {
    UserId::try_new(name).expect("a test user id")
}

pub(crate) fn peer(byte: u8) -> PeerId {
    PeerId::from_bytes([byte; 16])
}

pub(crate) struct Token<'a> {
    pub(crate) name: &'a str,
    pub(crate) permissions: Vec<Permission>,
    pub(crate) audience: Vec<JwtAudience>,
    pub(crate) plugin: Option<&'a str>,
    pub(crate) purpose: Option<&'a str>,
}

impl<'a> Token<'a> {
    pub(crate) fn person(name: &'a str) -> Self {
        Self {
            name,
            permissions: vec![Permission::User],
            audience: vec![JwtAudience::Bridge, JwtAudience::Mcp],
            plugin: None,
            purpose: None,
        }
    }

    pub(crate) fn mint(&self) -> String {
        mint_with(AUTHORITY.as_ref(), self)
    }
}

fn mint_with(authority: &dyn TokenAuthority, token: &Token<'_>) -> String {
    authority
        .mint_user_jwt(&UserJwtRequest {
            user: AuthenticatedUser::new(
                user(token.name),
                token.name.to_owned(),
                format!("{}@example.test", token.name),
                token.permissions.clone(),
            ),
            session_id: SessionId::generate(),
            jti: AccessTokenId::generate(),
            permissions: token.permissions.clone(),
            audience: token.audience.clone(),
            ttl: chrono::Duration::minutes(30),
            client_id: None,
            purpose: token.purpose.map(str::to_owned),
            plugin_id: token.plugin.map(PluginId::new),
            interaction: None,
            browser_session: false,
        })
        .expect("mint a token")
}

// The DynWork desktop's JWT for `name`: a user, audiences `bridge` and `mcp`,
// as core's bridge device link mints it.
pub(crate) fn desktop_token(name: &str) -> String {
    Token::person(name).mint()
}

pub(crate) fn token_with_audience(name: &str, audience: JwtAudience) -> String {
    Token {
        audience: vec![audience],
        ..Token::person(name)
    }
    .mint()
}

// Core's anonymous site session: a real anonymous user, first-party audiences.
pub(crate) fn anonymous_token(name: &str) -> String {
    Token {
        permissions: vec![Permission::Anonymous],
        audience: JwtAudience::standard(),
        ..Token::person(name)
    }
    .mint()
}

// A well-formed token signed by another deployment's key.
pub(crate) fn foreign_token(name: &str) -> String {
    mint_with(FOREIGN.as_ref(), &Token::person(name))
}

#[derive(Debug)]
pub(crate) struct ManualClock(Mutex<DateTime<Utc>>);

impl ManualClock {
    pub(crate) fn now_plus(&self, seconds: i64) -> DateTime<Utc> {
        *self.0.lock().unwrap() + chrono::Duration::seconds(seconds)
    }

    pub(crate) fn advance(&self, by: chrono::Duration) {
        let mut now = self.0.lock().unwrap();
        *now += by;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> DateTime<Utc> {
        *self.0.lock().unwrap()
    }
}

#[derive(Debug, Default)]
pub(crate) struct RecordingAudit(Mutex<Vec<AuditEvent>>);

impl RecordingAudit {
    pub(crate) fn events(&self) -> Vec<AuditEvent> {
        self.0.lock().unwrap().clone()
    }
}

impl AuditSink for RecordingAudit {
    fn record(&self, event: &AuditEvent) {
        self.0.lock().unwrap().push(event.clone());
    }
}

pub(crate) struct Relay {
    pub(crate) state: RelayState,
    pub(crate) store: Arc<MemoryStore>,
    pub(crate) clock: Arc<ManualClock>,
    pub(crate) audit: Arc<RecordingAudit>,
    pub(crate) tasks: BackgroundTasks,
    pub(crate) router: Router,
    pub(crate) addr: SocketAddr,
}

impl Relay {
    pub(crate) async fn start() -> Self {
        Self::with_limits(RelayLimits::default()).await
    }

    pub(crate) async fn with_limits(limits: RelayLimits) -> Self {
        let state = |store: &Arc<MemoryStore>,
                     clock: &Arc<ManualClock>,
                     audit: &Arc<RecordingAudit>,
                     tasks: &BackgroundTasks| {
            let store_dyn: Arc<dyn dynwork_relay::store::RelayStore> =
                Arc::<MemoryStore>::clone(store);
            let clock_dyn: Arc<dyn Clock> = Arc::<ManualClock>::clone(clock);
            let audit_dyn: Arc<dyn AuditSink> = Arc::<RecordingAudit>::clone(audit);
            RelayState::builder(store_dyn, authority(), tasks.clone())
                .limits(limits)
                .clock(clock_dyn)
                .audit(audit_dyn)
                .build()
                .expect("valid limits")
        };
        let store = Arc::new(MemoryStore::default());
        for name in ["alice", "mallory"] {
            store.add_user(&user(name));
        }
        let clock = Arc::new(ManualClock(Mutex::new(Utc::now())));
        let audit = Arc::new(RecordingAudit::default());
        let tasks = BackgroundTasks::new();
        let state = state(&store, &clock, &audit, &tasks);
        let router = Router::new().nest(EXTENSION_BASE_PATH, extension_router(state.clone()));
        let addr = serve(router.clone()).await;
        Self {
            state,
            store,
            clock,
            audit,
            tasks,
            router,
            addr,
        }
    }

    pub(crate) fn ws_url(&self, path: &str) -> String {
        format!("ws://{}{BASE_PATH}{path}", self.addr)
    }

    // One JSON request through the router.
    pub(crate) async fn request(
        &self,
        method: Method,
        path: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut builder = Request::builder()
            .method(method)
            .uri(format!("{BASE_PATH}{path}"));
        if let Some(token) = token {
            builder = builder.header("authorization", format!("Bearer {token}"));
        }
        let request = match body {
            Some(body) => builder
                .header("content-type", "application/json")
                .body(Body::from(body.to_string())),
            None => builder.body(Body::empty()),
        }
        .expect("a request");
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("a response");
        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("a body")
            .to_bytes();
        let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, value)
    }

    pub(crate) async fn create_pairing(&self, owner: &str) -> String {
        let (status, body) = self
            .request(Method::POST, "/pairings", Some(&desktop_token(owner)), None)
            .await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
        body["token"].as_str().expect("a token").to_owned()
    }

    pub(crate) async fn claim(&self, token: &str, phone: PeerId) -> (StatusCode, Value) {
        self.request(
            Method::POST,
            "/pairings/claim",
            None,
            Some(json!({ "token": token, "peer_id": phone.to_hex(), "name": "Pixel" })),
        )
        .await
    }

    // Pairs a phone for `owner`; returns its device id and credential.
    pub(crate) async fn pair_phone(&self, owner: &str, phone: PeerId) -> (String, String) {
        let token = self.create_pairing(owner).await;
        let (status, body) = self.claim(&token, phone).await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
        (
            body["device_id"].as_str().expect("a device id").to_owned(),
            body["credential"]
                .as_str()
                .expect("a credential")
                .to_owned(),
        )
    }

    pub(crate) async fn connect_desktop(
        &self,
        token: &str,
        peer: Option<PeerId>,
    ) -> Result<Ws, tungstenite::Error> {
        connect_desktop_at(&self.ws_url("/desktop"), token, peer).await
    }

    pub(crate) async fn desktop(&self, owner: &str, peer: PeerId) -> Ws {
        self.connect_desktop(&desktop_token(owner), Some(peer))
            .await
            .expect("desktop connects")
    }

    pub(crate) async fn connect_phone(&self, credential: &str) -> Result<Ws, tungstenite::Error> {
        let mut request = self.ws_url("/device").into_client_request()?;
        request.headers_mut().insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {credential}")).expect("a header"),
        );
        tokio_tungstenite::connect_async(request)
            .await
            .map(|(ws, _)| ws)
    }

    pub(crate) async fn phone(&self, credential: &str) -> Ws {
        self.connect_phone(credential)
            .await
            .expect("phone connects")
    }
}

pub(crate) async fn connect_desktop_at(
    url: &str,
    token: &str,
    peer: Option<PeerId>,
) -> Result<Ws, tungstenite::Error> {
    let mut request = url.into_client_request()?;
    request.headers_mut().insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {token}")).expect("a header"),
    );
    if let Some(peer) = peer {
        request.headers_mut().insert(
            PEER_HEADER,
            HeaderValue::from_str(&peer.to_hex()).expect("a header"),
        );
    }
    tokio_tungstenite::connect_async(request)
        .await
        .map(|(ws, _)| ws)
}

pub(crate) async fn serve(router: Router) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind loopback");
    let addr = listener.local_addr().expect("local address");
    tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("serve the relay");
    });
    addr
}

// The HTTP status a refused handshake answered with.
pub(crate) fn refused_status(result: Result<Ws, tungstenite::Error>) -> u16 {
    match result {
        Err(tungstenite::Error::Http(response)) => response.status().as_u16(),
        Err(other) => panic!("handshake failed without an HTTP status: {other}"),
        Ok(_) => panic!("handshake was accepted"),
    }
}

pub(crate) fn envelope(to: PeerId, from: PeerId, seq: u32, ciphertext: &[u8]) -> Vec<u8> {
    let mut bytes = vec![1];
    bytes.extend_from_slice(to.as_bytes());
    bytes.extend_from_slice(from.as_bytes());
    bytes.extend_from_slice(&seq.to_be_bytes());
    bytes.extend_from_slice(ciphertext);
    bytes
}

pub(crate) async fn send(ws: &mut Ws, bytes: Vec<u8>) {
    ws.send(Message::binary(bytes))
        .await
        .expect("send an envelope");
}

// The next envelope within `within`, skipping pings and relay presence
// messages.
pub(crate) async fn recv(ws: &mut Ws, within: Duration) -> Option<Vec<u8>> {
    let deadline = tokio::time::Instant::now() + within;
    loop {
        let message = recv_any(ws, deadline).await?;
        if message.first() != Some(&dynwork_relay::envelope::CONTROL_VERSION) {
            return Some(message);
        }
    }
}

// The next relay presence message within `within`, skipping envelopes.
pub(crate) async fn recv_presence(ws: &mut Ws, within: Duration) -> Option<Vec<u8>> {
    let deadline = tokio::time::Instant::now() + within;
    loop {
        let message = recv_any(ws, deadline).await?;
        if message.first() == Some(&dynwork_relay::envelope::CONTROL_VERSION) {
            return Some(message);
        }
    }
}

async fn recv_any(ws: &mut Ws, deadline: tokio::time::Instant) -> Option<Vec<u8>> {
    loop {
        let next = tokio::time::timeout_at(deadline, ws.next()).await.ok()??;
        match next.ok()? {
            Message::Binary(bytes) => return Some(bytes.to_vec()),
            Message::Ping(_) | Message::Pong(_) => {},
            _ => return None,
        }
    }
}

// The close code the relay sent within `within`.
pub(crate) async fn close_code(ws: &mut Ws, within: Duration) -> Option<u16> {
    let deadline = tokio::time::Instant::now() + within;
    loop {
        let next = tokio::time::timeout_at(deadline, ws.next()).await.ok()??;
        match next {
            Ok(Message::Close(Some(frame))) => return Some(u16::from(frame.code)),
            Ok(Message::Close(None)) => return Some(u16::from(CloseCode::Status)),
            Ok(_) => {},
            Err(_) => return None,
        }
    }
}
