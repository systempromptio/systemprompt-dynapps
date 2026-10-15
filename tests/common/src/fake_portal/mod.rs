//! The Dynapps portal (`portal.dynapps.es`) on loopback TLS: the OAuth
//! surface the generic connector discovers and the streamable-HTTP MCP
//! endpoints the gateway proxies to.
//!
//! - RFC 9728 protected-resource metadata, path-suffixed per MCP path and bare.
//! - RFC 8414 authorization-server metadata, PKCE `S256` only.
//! - RFC 7591 `POST /register` for public clients.
//! - `GET /authorize`: validates the client, redirect URI and S256 challenge,
//!   and redirects to the callback with `code`, `state` and `iss`. The code is
//!   issued to whichever portal account [`FakePortal::sign_in_as`] chose, the
//!   way a browser carries the portal login.
//! - `POST /token`: `authorization_code` (verifier checked against the
//!   challenge) and `refresh_token` (rotating). Every access token is distinct
//!   and remembers its subject.
//! - MCP at [`ODOO_PATH`] and [`DYNBRAIN_PATH`]: `initialize` (issues an
//!   `Mcp-Session-Id`), `notifications/initialized`, `tools/list`,
//!   `tools/call`; `whoami` answers the bearer's subject. A missing, unknown or
//!   expired bearer gets 401 with `WWW-Authenticate: Bearer
//!   resource_metadata=...`.
//!
//! Every request is recorded with its `Authorization` header, so a test asks
//! which bearers reached which path and whose they were.

mod mcp;
mod routes;
mod tls;

pub mod env;

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::Bytes;
use axum::http::{HeaderMap, Method, Uri};
use tokio::task::JoinHandle;

use crate::fake_http::{RecordedRequest, RequestLog, bind, snapshot};

// The portal's routing path for the Dynapps Odoo MCP
// (`services/mcp/dynapps-odoo.yaml`).
pub const ODOO_PATH: &str = "/mcp/uwX00RzEnfs5Yw_tVlx7aw";
// The portal's routing path for DynBrain (`services/mcp/dynbrain.yaml`).
pub const DYNBRAIN_PATH: &str = "/mcp/HHqBulo1Jyae9zA-IE95WA";
pub const MCP_PATHS: [&str; 2] = [ODOO_PATH, DYNBRAIN_PATH];

#[derive(Debug, Clone)]
struct PendingCode {
    client_id: String,
    redirect_uri: String,
    challenge: String,
    subject: String,
}

#[derive(Debug, Clone)]
struct Issued {
    subject: String,
    expired: bool,
}

#[derive(Debug, Default)]
struct Inner {
    base_url: String,
    requests: RequestLog,
    login: Mutex<String>,
    expires_in: Mutex<i64>,
    clients: Mutex<BTreeMap<String, Vec<String>>>,
    codes: Mutex<BTreeMap<String, PendingCode>>,
    access: Mutex<BTreeMap<String, Issued>>,
    refresh: Mutex<BTreeMap<String, String>>,
    sessions: Mutex<BTreeSet<String>>,
    counter: AtomicU64,
}

impl Inner {
    fn next(&self, prefix: &str) -> String {
        let n = self.counter.fetch_add(1, Ordering::SeqCst);
        format!("{prefix}-{n}-{}", uuid::Uuid::new_v4().simple())
    }

    // A fresh access/refresh pair for `subject`.
    fn issue(&self, subject: &str) -> (String, String) {
        let access = self.next("portal-at");
        let refresh = self.next("portal-rt");
        self.access.lock().expect("access").insert(
            access.clone(),
            Issued {
                subject: subject.to_owned(),
                expired: false,
            },
        );
        self.refresh
            .lock()
            .expect("refresh")
            .insert(refresh.clone(), subject.to_owned());
        (access, refresh)
    }

    // The live subject behind `Authorization: Bearer <token>`.
    fn subject_of_header(&self, authorization: Option<&str>) -> Option<String> {
        let token = authorization?.strip_prefix("Bearer ")?.trim();
        let access = self.access.lock().expect("access");
        access
            .get(token)
            .filter(|issued| !issued.expired)
            .map(|issued| issued.subject.clone())
    }
}

// An issued access/refresh pair, for seeding a grant without the browser leg.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug)]
pub struct FakePortal {
    inner: Arc<Inner>,
    certs: tempfile::TempDir,
    task: JoinHandle<()>,
}

impl FakePortal {
    pub async fn start() -> Self {
        let certs = tempfile::tempdir().expect("fake portal cert dir");
        tls::write_certificates(certs.path());
        let (listener, http_url) = bind().await;
        let base_url = http_url.replacen("http://", "https://", 1);
        let inner = Arc::new(Inner {
            base_url,
            login: Mutex::new("portal-user".to_owned()),
            expires_in: Mutex::new(3600),
            ..Inner::default()
        });
        let state = Arc::clone(&inner);
        let router = Router::new().fallback(
            move |method: Method, uri: Uri, headers: HeaderMap, body: Bytes| {
                let state = Arc::clone(&state);
                async move { routes::handle(&state, &method, &uri, &headers, &body) }
            },
        );
        let task = tls::spawn(listener, tls::acceptor(certs.path()), router);
        Self { inner, certs, task }
    }

    // `https://127.0.0.1:<port>`, the issuer and resource origin.
    pub fn base_url(&self) -> &str {
        &self.inner.base_url
    }

    // The MCP endpoint for one of [`MCP_PATHS`].
    pub fn endpoint(&self, path: &str) -> String {
        format!("{}{path}", self.inner.base_url)
    }

    // The PEM of the CA that signed the portal's certificate.
    pub fn ca_pem_path(&self) -> PathBuf {
        self.certs.path().join("ca.pem")
    }

    pub fn ca_pem(&self) -> Vec<u8> {
        std::fs::read(self.ca_pem_path()).expect("read fake portal CA")
    }

    // The portal account the next `/authorize` issues its code to.
    pub fn sign_in_as(&self, subject: &str) {
        subject.clone_into(&mut self.inner.login.lock().expect("login"));
    }

    // `expires_in` on every later token response (must be positive).
    pub fn set_expires_in(&self, seconds: i64) {
        *self.inner.expires_in.lock().expect("expires_in") = seconds;
    }

    // Issue a pair for `subject` directly, as if its consent had completed.
    pub fn issue_tokens(&self, subject: &str) -> TokenPair {
        let (access_token, refresh_token) = self.inner.issue(subject);
        TokenPair {
            access_token,
            refresh_token,
        }
    }

    // From now on `access_token` is refused like an expired one.
    pub fn expire(&self, access_token: &str) {
        if let Some(issued) = self
            .inner
            .access
            .lock()
            .expect("access")
            .get_mut(access_token)
        {
            issued.expired = true;
        }
    }

    // Whose bearer `access_token` is, expired or not.
    pub fn subject_of(&self, access_token: &str) -> Option<String> {
        let token = access_token.strip_prefix("Bearer ").unwrap_or(access_token);
        self.inner
            .access
            .lock()
            .expect("access")
            .get(token)
            .map(|issued| issued.subject.clone())
    }

    pub fn requests(&self) -> Vec<RecordedRequest> {
        snapshot(&self.inner.requests)
    }

    // Every bearer presented to `path`, in order (without the `Bearer ` prefix).
    pub fn bearers_seen(&self, path: &str) -> Vec<String> {
        self.requests()
            .into_iter()
            .filter(|r| r.path == path)
            .filter_map(|r| r.authorization)
            .filter_map(|a| a.strip_prefix("Bearer ").map(str::to_owned))
            .collect()
    }

    // The subjects behind every bearer presented to `path`.
    pub fn subjects_seen(&self, path: &str) -> Vec<String> {
        self.bearers_seen(path)
            .iter()
            .filter_map(|b| self.subject_of(b))
            .collect()
    }

    // The `grant_type` of every `POST /token`, in order.
    pub fn token_grant_types(&self) -> Vec<String> {
        self.requests()
            .into_iter()
            .filter(|r| r.method == "POST" && r.path == "/token")
            .filter_map(|r| {
                url::form_urlencoded::parse(r.body.as_bytes())
                    .find(|(k, _)| k == "grant_type")
                    .map(|(_, v)| v.into_owned())
            })
            .collect()
    }

    pub fn shutdown(&self) {
        self.task.abort();
    }
}

impl Drop for FakePortal {
    fn drop(&mut self) {
        self.task.abort();
    }
}
