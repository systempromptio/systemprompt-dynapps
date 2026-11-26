//! The Brain router as core mounts it, over a throwaway database with the
//! full extension schema, a real RS256 token authority, the relay's own
//! device table for phones, and core's local-disk file storage in a
//! throwaway directory for the files of articles.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use dynapps_test_common::TempDb;
use dynbrain::assets::AssetFiles;
use dynbrain::assets::link::LinkSigner;
use dynbrain::brain::native::NativeBrain;
use dynbrain::router::state::BrainState;
use dynbrain::router::{EXTENSION_BASE_PATH, brain_router};
use dynbrain::search::query::EmbedderSlot;
use dynwork_relay::ids::DeviceId;
use dynwork_relay::mobile_auth::jwt;
use dynwork_relay::store::Account;
use dynwork_relay::store::postgres::PgRelayStore;
use http_body_util::BodyExt;
use serde_json::Value;
use systemprompt::identifiers::{AccessTokenId, SessionId, UserId};
use systemprompt::models::auth::{
    AuthenticatedUser, JwtAudience, Permission, TokenAuthority, UserJwtRequest,
};
use systemprompt::storage::LocalFileStorage;
use systemprompt_security::jwt::DeploymentTokenAuthority;
use systemprompt_security::keys::{RsaSigningKey, SigningKeys};
use tower::ServiceExt;

use crate::fixture::Fixture;

const ISSUER: &str = "https://brain.test";
pub(crate) const BASE_URL: &str = "https://brain.test";
pub(crate) const MAX_UPLOAD: u64 = 64 * 1024;
const LINK_SEED: [u8; 32] = [9; 32];
static ROOTS: AtomicU64 = AtomicU64::new(0);

// The signer of the suite's links, the one the app under test holds.
pub(crate) fn signer() -> LinkSigner {
    LinkSigner::new(&LINK_SEED).expect("a link signer")
}

// The path a signed URL names, for the in-process router.
pub(crate) fn local(url: &str) -> &str {
    url.strip_prefix(BASE_URL).expect("a URL of this instance")
}

fn authority_with_seed(seed: u8) -> Arc<DeploymentTokenAuthority> {
    let key = RsaSigningKey::generate().expect("generate an RSA key");
    let keys = SigningKeys::new(key, &[seed; 32]).expect("build signing keys");
    Arc::new(DeploymentTokenAuthority::new(keys, ISSUER))
}

static AUTHORITY: LazyLock<Arc<DeploymentTokenAuthority>> =
    LazyLock::new(|| authority_with_seed(21));
static FOREIGN: LazyLock<Arc<DeploymentTokenAuthority>> = LazyLock::new(|| authority_with_seed(22));

fn mint(
    authority: &DeploymentTokenAuthority,
    name: &str,
    permissions: Vec<Permission>,
    audience: Vec<JwtAudience>,
) -> String {
    let id = UserId::try_new(name).expect("a test user id");
    authority
        .mint_user_jwt(&UserJwtRequest {
            user: AuthenticatedUser::new(
                id,
                name.to_owned(),
                format!("{name}@example.test"),
                permissions.clone(),
            ),
            session_id: SessionId::generate(),
            jti: AccessTokenId::generate(),
            permissions,
            audience,
            ttl: chrono::Duration::minutes(30),
            client_id: None,
            purpose: None,
            plugin_id: None,
            interaction: None,
            browser_session: false,
            ui_session: false,
            auth_time: None,
        })
        .expect("mint a token")
}

// The DynWork desktop's JWT, as core's bridge device link mints it.
pub(crate) fn desktop_token(name: &str) -> String {
    mint(
        &AUTHORITY,
        name,
        vec![Permission::User],
        vec![JwtAudience::Bridge, JwtAudience::Mcp],
    )
}

pub(crate) fn token_with_audience(name: &str, audience: JwtAudience) -> String {
    mint(&AUTHORITY, name, vec![Permission::User], vec![audience])
}

pub(crate) fn foreign_token(name: &str) -> String {
    mint(
        &FOREIGN,
        name,
        vec![Permission::User],
        vec![JwtAudience::Bridge],
    )
}

pub(crate) fn anonymous_token(name: &str) -> String {
    mint(
        &AUTHORITY,
        name,
        vec![Permission::Anonymous],
        JwtAudience::standard(),
    )
}

// The phone's JWT for `device`, minted by the relay's own sign-in code.
pub(crate) fn mobile_token(name: &str, device: DeviceId) -> String {
    let account = Account {
        user: UserId::try_new(name).expect("a test user id"),
        name: name.to_owned(),
        email: format!("{name}@example.test"),
        roles: Vec::new(),
    };
    jwt::mint(AUTHORITY.as_ref(), &account, device)
        .expect("mint a mobile token")
        .token
}

pub(crate) struct Answer {
    pub(crate) status: StatusCode,
    pub(crate) body: Value,
}

impl Answer {
    pub(crate) fn key(&self) -> &str {
        self.body["error"]["key"].as_str().unwrap_or("")
    }
}

// A raw answer: status, headers and the body's bytes.
pub(crate) struct Raw {
    pub(crate) status: StatusCode,
    pub(crate) headers: HeaderMap,
    pub(crate) bytes: Vec<u8>,
}

impl Raw {
    pub(crate) fn header(&self, name: header::HeaderName) -> &str {
        self.headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
    }

    pub(crate) fn json(&self) -> Value {
        serde_json::from_slice(&self.bytes).unwrap_or(Value::Null)
    }
}

pub(crate) struct App {
    router: Router,
    pub(crate) brain: NativeBrain,
    // The file storage's directory: what core's storage wrote, on disk.
    pub(crate) root: PathBuf,
    pub(crate) fixture: Fixture,
}

impl App {
    pub(crate) async fn new(db: TempDb) -> Self {
        let fixture = Fixture::new(db).await;
        let root = std::env::temp_dir().join(format!(
            "dynbrain-files-{}-{}",
            std::process::id(),
            ROOTS.fetch_add(1, Ordering::Relaxed)
        ));
        let files = AssetFiles {
            max_upload_bytes: MAX_UPLOAD,
            ..AssetFiles::new(
                Arc::new(LocalFileStorage::new(root.clone())),
                signer(),
                BASE_URL.to_owned(),
            )
        };
        let brain = NativeBrain::new(fixture.repo.clone()).with_files(Some(files));
        let authority: Arc<dyn TokenAuthority> = Arc::<DeploymentTokenAuthority>::clone(&AUTHORITY);
        let state = BrainState {
            brain: Arc::new(brain.clone()),
            authority,
            devices: Arc::new(PgRelayStore::new(fixture.shared_pool())),
            embedder: EmbedderSlot::default(),
        };
        let router = Router::new().nest(EXTENSION_BASE_PATH, brain_router(state));
        Self {
            router,
            brain,
            root,
            fixture,
        }
    }

    pub(crate) async fn finish(self) {
        std::fs::remove_dir_all(&self.root).ok();
        self.fixture.finish().await;
    }

    // Sends any request, `path` being the whole path after the host.
    pub(crate) async fn send(&self, request: Request<Body>) -> Raw {
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("the router answers");
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("read the body")
            .to_bytes()
            .to_vec();
        Raw {
            status,
            headers,
            bytes,
        }
    }

    // Bytes POSTed to an upload URL, with no bearer token: the URL is the
    // permission.
    pub(crate) async fn post_bytes(&self, url: &str, content_type: &str, bytes: &[u8]) -> Raw {
        let request = Request::builder()
            .method(Method::POST)
            .uri(local(url))
            .header(header::CONTENT_TYPE, content_type)
            .body(Body::from(bytes.to_vec()))
            .expect("build an upload");
        self.send(request).await
    }

    // A GET of `path` (the whole path after the host), with or without the
    // desktop token of `who`.
    pub(crate) async fn fetch(&self, path: &str, who: Option<&str>) -> Raw {
        let mut request = Request::builder().method(Method::GET).uri(path);
        if let Some(name) = who {
            request = request.header(
                header::AUTHORIZATION,
                format!("Bearer {}", desktop_token(name)),
            );
        }
        self.send(request.body(Body::empty()).expect("build a GET"))
            .await
    }

    pub(crate) async fn request(
        &self,
        method: Method,
        path: &str,
        bearer: Option<&str>,
        body: Option<&Value>,
    ) -> Answer {
        let mut request = Request::builder()
            .method(method)
            .uri(format!("{EXTENSION_BASE_PATH}{path}"));
        if let Some(token) = bearer {
            request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        if body.is_some() {
            request = request.header(header::CONTENT_TYPE, "application/json");
        }
        let request = request
            .body(Body::from(body.map(Value::to_string).unwrap_or_default()))
            .expect("build a request");
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("the router answers");
        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("read the body")
            .to_bytes();
        Answer {
            status,
            body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        }
    }

    pub(crate) async fn get(&self, path: &str, who: &str) -> Answer {
        self.request(Method::GET, path, Some(&desktop_token(who)), None)
            .await
    }

    pub(crate) async fn post(&self, path: &str, who: &str, body: &Value) -> Answer {
        self.request(Method::POST, path, Some(&desktop_token(who)), Some(body))
            .await
    }

    pub(crate) async fn patch(&self, path: &str, who: &str, body: &Value) -> Answer {
        self.request(Method::PATCH, path, Some(&desktop_token(who)), Some(body))
            .await
    }

    pub(crate) async fn put(&self, path: &str, who: &str, body: &Value) -> Answer {
        self.request(Method::PUT, path, Some(&desktop_token(who)), Some(body))
            .await
    }

    // A signed-in phone of `name`: a live device row and its mobile JWT.
    pub(crate) async fn phone(&self, name: &str) -> (DeviceId, String) {
        let device = DeviceId::generate();
        sqlx::query(
            "INSERT INTO dynwork_relay_devices (id, user_id, kind, peer_id, name, credential_hash)
             VALUES ($1, $2, 'phone', $3, 'test phone', $4)",
        )
        .bind(device.to_string())
        .bind(name)
        .bind(device.as_uuid().simple().to_string())
        .bind(format!("hash-{device}"))
        .execute(self.fixture.pool())
        .await
        .expect("seed a phone device");
        (device, mobile_token(name, device))
    }

    pub(crate) async fn revoke(&self, device: DeviceId) {
        sqlx::query(
            "UPDATE dynwork_relay_devices SET revoked_at = CURRENT_TIMESTAMP WHERE id = $1",
        )
        .bind(device.to_string())
        .execute(self.fixture.pool())
        .await
        .expect("revoke the phone");
    }
}
