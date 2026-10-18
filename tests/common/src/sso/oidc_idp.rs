//! An Entra-shaped OIDC provider on loopback HTTP.
//!
//! Serves discovery, JWKS, an authorize endpoint that echoes its query as
//! JSON (tests read `state`, `nonce` and `code_challenge` from it instead of
//! driving a browser) and a token endpoint that redeems codes minted by
//! [`FakeOidcIdp::issue_code`], checking the client secret and PKCE `S256`.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use axum::extract::{Form, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tokio::task::JoinHandle;

use super::keys::TestIdpKeys;
pub use crate::fake_http::RecordedRequest;
use crate::fake_http::{RequestLog, bind, record, snapshot, spawn};

#[derive(Debug, Clone, Default)]
pub struct IdTokenSpec {
    pub sub: String,
    pub oid: String,
    pub email: Option<String>,
    pub preferred_username: Option<String>,
    pub name: String,
    pub groups: Vec<String>,
    // Emit `_claim_names`/`_claim_sources` in place of `groups`.
    pub groups_overage: bool,
    pub nonce: String,
    pub aud: String,
    pub tid: String,
    pub iat: Option<DateTime<Utc>>,
    pub exp: Option<DateTime<Utc>>,
    pub jti: Option<String>,
    pub sign_with_other_key: bool,
    pub wrong_issuer: bool,
}

#[derive(Debug)]
struct Inner {
    issuer: String,
    client_secret: String,
    codes: Mutex<HashMap<String, (IdTokenSpec, String)>>,
    requests: RequestLog,
}

#[derive(Debug)]
pub struct FakeOidcIdp {
    inner: Arc<Inner>,
    base_url: String,
    tenant: String,
    task: JoinHandle<()>,
}

impl FakeOidcIdp {
    // Returns only once the IdP can answer every endpoint within the identity
    // crate's 10s HTTP timeout: the listener is bound (connections queue in its
    // backlog before the accept loop runs) and both signing keys exist. Left to
    // the token and JWKS handlers, a first RSA-2048 keygen overruns that
    // timeout under an instrumented build and the sign-in is refused as
    // `unavailable`.
    pub async fn start(tenant_id: &str) -> Self {
        tokio::task::spawn_blocking(|| {
            TestIdpKeys::get();
            TestIdpKeys::other();
        })
        .await
        .expect("generate the fake IdP signing keys");
        let (listener, base_url) = bind().await;
        let inner = Arc::new(Inner {
            issuer: format!("{base_url}/{tenant_id}/v2.0"),
            client_secret: format!("fake-oidc-secret-{tenant_id}"),
            codes: Mutex::default(),
            requests: RequestLog::default(),
        });
        let t = tenant_id;
        let router = Router::new()
            .route(
                &format!("/{t}/v2.0/.well-known/openid-configuration"),
                get(discovery),
            )
            .route(&format!("/{t}/discovery/v2.0/keys"), get(jwks))
            .route(&format!("/{t}/oauth2/v2.0/authorize"), get(authorize))
            .route(&format!("/{t}/oauth2/v2.0/token"), post(token))
            .with_state(Arc::clone(&inner));
        let task = spawn(listener, router);
        Self {
            inner,
            base_url,
            tenant: tenant_id.to_owned(),
            task,
        }
    }

    pub fn issuer(&self) -> &str {
        &self.inner.issuer
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn tenant(&self) -> &str {
        &self.tenant
    }

    pub fn discovery_url(&self) -> String {
        format!("{}/.well-known/openid-configuration", self.inner.issuer)
    }

    pub fn client_secret(&self) -> &str {
        &self.inner.client_secret
    }

    pub fn requests(&self) -> Vec<RecordedRequest> {
        snapshot(&self.inner.requests)
    }

    pub fn request_log(&self) -> RequestLog {
        Arc::clone(&self.inner.requests)
    }

    // A single-use code the token endpoint redeems for `spec`'s ID token,
    // provided the caller presents a verifier hashing to `code_challenge`.
    pub fn issue_code(&self, spec: IdTokenSpec, code_challenge: &str) -> String {
        let code = format!("code-{}", uuid::Uuid::new_v4().simple());
        self.inner
            .codes
            .lock()
            .expect("codes")
            .insert(code.clone(), (spec, code_challenge.to_owned()));
        code
    }

    pub fn mint_id_token(&self, spec: &IdTokenSpec) -> String {
        mint(&self.inner.issuer, spec)
    }

    pub fn shutdown(&self) {
        self.task.abort();
    }
}

impl Drop for FakeOidcIdp {
    fn drop(&mut self) {
        self.task.abort();
    }
}

// The PKCE `S256` challenge for `verifier`.
pub fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn signing_keys(spec: &IdTokenSpec) -> &'static TestIdpKeys {
    if spec.sign_with_other_key {
        TestIdpKeys::other()
    } else {
        TestIdpKeys::get()
    }
}

fn mint(issuer: &str, spec: &IdTokenSpec) -> String {
    let keys = signing_keys(spec);
    let now = Utc::now();
    let iat = spec.iat.unwrap_or(now);
    let exp = spec.exp.unwrap_or(iat + Duration::hours(1));
    let iss = if spec.wrong_issuer {
        format!("{issuer}/not-this-one")
    } else {
        issuer.to_owned()
    };
    let mut claims = json!({
        "iss": iss, "sub": spec.sub, "oid": spec.oid, "aud": spec.aud, "tid": spec.tid,
        "nonce": spec.nonce, "name": spec.name, "ver": "2.0",
        "iat": iat.timestamp(), "nbf": iat.timestamp(), "exp": exp.timestamp(),
    });
    let obj = claims.as_object_mut().expect("claims object");
    if let Some(email) = &spec.email {
        obj.insert("email".into(), json!(email));
    }
    if let Some(upn) = &spec.preferred_username {
        obj.insert("preferred_username".into(), json!(upn));
    }
    if let Some(jti) = &spec.jti {
        obj.insert("jti".into(), json!(jti));
    }
    if spec.groups_overage {
        obj.insert("_claim_names".into(), json!({ "groups": "src1" }));
        let endpoint = format!(
            "https://graph.microsoft.com/v1.0/users/{}/getMemberObjects",
            spec.oid
        );
        obj.insert(
            "_claim_sources".into(),
            json!({ "src1": { "endpoint": endpoint } }),
        );
    } else if !spec.groups.is_empty() {
        obj.insert("groups".into(), json!(spec.groups));
    }
    let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    header.kid = Some(keys.kid.clone());
    jsonwebtoken::encode(&header, &claims, &keys.encoding_key()).expect("sign ID token")
}

fn log(inner: &Inner, method: &str, path: &str, query: Option<String>, body: String) {
    let request = RecordedRequest {
        method: method.to_owned(),
        path: path.to_owned(),
        query,
        authorization: None,
        body,
    };
    record(&inner.requests, request);
}

async fn discovery(State(inner): State<Arc<Inner>>) -> Json<Value> {
    log(&inner, "GET", "openid-configuration", None, String::new());
    let base = inner.issuer.trim_end_matches("/v2.0");
    Json(json!({
        "issuer": inner.issuer,
        "authorization_endpoint": format!("{base}/oauth2/v2.0/authorize"),
        "token_endpoint": format!("{base}/oauth2/v2.0/token"),
        "jwks_uri": format!("{base}/discovery/v2.0/keys"),
        "response_types_supported": ["code"],
        "subject_types_supported": ["pairwise"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "code_challenge_methods_supported": ["S256"],
    }))
}

async fn jwks(State(inner): State<Arc<Inner>>) -> Json<Value> {
    log(&inner, "GET", "jwks", None, String::new());
    Json(TestIdpKeys::get().jwks())
}

async fn authorize(
    State(inner): State<Arc<Inner>>,
    Query(params): Query<BTreeMap<String, String>>,
) -> Json<BTreeMap<String, String>> {
    let query = serde_json::to_string(&params).expect("query json");
    log(&inner, "GET", "authorize", Some(query), String::new());
    Json(params)
}

async fn token(
    State(inner): State<Arc<Inner>>,
    Form(form): Form<BTreeMap<String, String>>,
) -> Response {
    log(
        &inner,
        "POST",
        "token",
        None,
        serde_json::to_string(&form).expect("form json"),
    );
    let field = |k: &str| form.get(k).map(String::as_str).unwrap_or_default();
    if field("client_secret") != inner.client_secret {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid_client" })),
        )
            .into_response();
    }
    let redeemed = inner.codes.lock().expect("codes").remove(field("code"));
    let valid = field("grant_type") == "authorization_code"
        && redeemed
            .as_ref()
            .is_some_and(|(_, c)| pkce_challenge(field("code_verifier")) == *c);
    let Some((spec, _)) = redeemed.filter(|_| valid) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_grant" })),
        )
            .into_response();
    };
    Json(json!({
        "id_token": mint(&inner.issuer, &spec),
        "access_token": format!("access-{}", uuid::Uuid::new_v4().simple()),
        "token_type": "Bearer",
        "expires_in": 3600,
    }))
    .into_response()
}
