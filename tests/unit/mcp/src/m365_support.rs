//! A simulated Microsoft Graph (and broker) on loopback for the
//! `microsoft-365` tests: every request is recorded, and a per-test script
//! decides each reply from the request and its index.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Query, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use serde_json::Value;
use systemprompt::traits::{ExtensionSecret, SecretLookup};
use systemprompt_mcp_microsoft_365::graph::{
    GraphClient, GraphClientParts, GraphEndpoint, GraphError, GraphLimits, GraphTokens, http_client,
};

#[derive(Debug, Clone)]
pub(crate) struct Seen {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) raw_query: Option<String>,
    pub(crate) query: HashMap<String, String>,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Vec<u8>,
}

impl Seen {
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub(crate) fn q(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(String::as_str)
    }

    pub(crate) fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Reply {
    pub(crate) status: u16,
    pub(crate) headers: Vec<(&'static str, String)>,
    pub(crate) body: Vec<u8>,
}

impl Reply {
    pub(crate) fn json(status: u16, value: &Value) -> Self {
        Self {
            status,
            headers: vec![("content-type", "application/json".to_owned())],
            body: serde_json::to_vec(value).unwrap(),
        }
    }

    pub(crate) fn bytes(status: u16, body: &[u8]) -> Self {
        Self {
            status,
            headers: vec![("content-type", "application/octet-stream".to_owned())],
            body: body.to_vec(),
        }
    }

    pub(crate) fn with_header(mut self, name: &'static str, value: impl Into<String>) -> Self {
        self.headers.push((name, value.into()));
        self
    }
}

type Script = Arc<dyn Fn(&Seen, usize) -> Reply + Send + Sync>;

#[derive(Clone)]
struct SimState {
    script: Script,
    calls: Arc<Mutex<Vec<Seen>>>,
}

async fn handle(
    State(state): State<SimState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let query = Query::<HashMap<String, String>>::try_from_uri(&uri)
        .map(|q| q.0)
        .unwrap_or_default();
    let seen = Seen {
        method: method.to_string(),
        path: uri.path().to_owned(),
        raw_query: uri.query().map(str::to_owned),
        query,
        headers: headers
            .iter()
            .map(|(k, v)| (k.as_str().to_owned(), v.to_str().unwrap_or("").to_owned()))
            .collect(),
        body: body.to_vec(),
    };
    let index = {
        let mut calls = state.calls.lock().unwrap();
        calls.push(seen.clone());
        calls.len() - 1
    };
    let reply = (state.script)(&seen, index);
    let mut response = (StatusCode::from_u16(reply.status).unwrap(), reply.body).into_response();
    for (name, value) in reply.headers {
        response.headers_mut().insert(
            HeaderName::from_static(name),
            HeaderValue::from_str(&value).unwrap(),
        );
    }
    response
}

pub(crate) struct Sim {
    pub(crate) base: String,
    calls: Arc<Mutex<Vec<Seen>>>,
}

impl Sim {
    pub(crate) async fn start(
        script: impl Fn(&Seen, usize) -> Reply + Send + Sync + 'static,
    ) -> Self {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let state = SimState {
            script: Arc::new(script),
            calls: Arc::clone(&calls),
        };
        // Upload-session chunks run to 3 MiB, past axum's 2 MB default.
        let app = Router::new()
            .fallback(handle)
            .layer(DefaultBodyLimit::max(64 * 1024 * 1024))
            .with_state(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Self {
            base: format!("http://{addr}"),
            calls,
        }
    }

    // The simulated Graph root, shaped like `https://graph.microsoft.com/v1.0`.
    pub(crate) fn graph_url(&self) -> String {
        format!("{}/v1.0", self.base)
    }

    pub(crate) fn calls(&self) -> Vec<Seen> {
        self.calls.lock().unwrap().clone()
    }
}

// Hands out `sequence` in order (repeating the last) and records each ask.
#[derive(Debug)]
pub(crate) struct FakeTokens {
    sequence: Vec<String>,
    asked: Mutex<Vec<Option<String>>>,
}

impl FakeTokens {
    pub(crate) fn new(sequence: &[&str]) -> Arc<Self> {
        Arc::new(Self {
            sequence: sequence.iter().map(|s| (*s).to_owned()).collect(),
            asked: Mutex::new(Vec::new()),
        })
    }

    pub(crate) fn asked(&self) -> Vec<Option<String>> {
        self.asked.lock().unwrap().clone()
    }
}

#[async_trait]
impl GraphTokens for FakeTokens {
    async fn access_token(&self, stale: Option<&str>) -> Result<String, GraphError> {
        let mut asked = self.asked.lock().unwrap();
        asked.push(stale.map(str::to_owned));
        let index = (asked.len() - 1).min(self.sequence.len() - 1);
        drop(asked);
        Ok(self.sequence[index].clone())
    }
}

#[derive(Debug, Default)]
pub(crate) struct Secrets(pub(crate) HashMap<String, String>);

impl Secrets {
    pub(crate) fn with(pairs: &[(&str, &str)]) -> Arc<Self> {
        Arc::new(Self(
            pairs
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
        ))
    }
}

impl SecretLookup for Secrets {
    fn lookup(&self, name: &str) -> Option<ExtensionSecret> {
        self.0.get(name).cloned().map(ExtensionSecret::new)
    }
}

// Retries that do not wait: `Retry-After: 0` and a 10 ms default backoff.
pub(crate) fn fast_limits() -> GraphLimits {
    GraphLimits {
        max_retry_wait: Duration::from_secs(5),
        default_backoff: Duration::from_millis(10),
        ..GraphLimits::default()
    }
}

pub(crate) fn client(base: &str, tokens: Arc<dyn GraphTokens>, limits: GraphLimits) -> GraphClient {
    GraphClient::new(GraphClientParts {
        endpoint: GraphEndpoint::parse(base).unwrap(),
        http: http_client(&limits).unwrap(),
        tokens,
        limits,
    })
}

pub(crate) fn collection(items: &[Value], next: Option<&str>) -> Value {
    let mut body = serde_json::json!({ "value": items });
    if let Some(link) = next {
        body["@odata.nextLink"] = Value::String(link.to_owned());
    }
    body
}
