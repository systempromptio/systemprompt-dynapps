//! Loopback HTTP plumbing shared by the fake upstreams: bind `127.0.0.1:0`,
//! serve an axum router on a Tokio task, and record what each request asked.

use std::sync::{Arc, Mutex};

use tokio::task::JoinHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
    pub query: Option<String>,
    pub authorization: Option<String>,
    pub body: String,
}

pub type RequestLog = Arc<Mutex<Vec<RecordedRequest>>>;

pub(crate) fn record(log: &RequestLog, request: RecordedRequest) {
    log.lock().expect("request log").push(request);
}

pub(crate) fn snapshot(log: &RequestLog) -> Vec<RecordedRequest> {
    log.lock().expect("request log").clone()
}

/// A bound loopback listener and its `http://127.0.0.1:{port}` base URL, so
/// a fake can bake its own address into what it serves before it starts.
pub(crate) async fn bind() -> (tokio::net::TcpListener, String) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind loopback");
    let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
    (listener, base_url)
}

pub(crate) fn spawn(listener: tokio::net::TcpListener, router: axum::Router) -> JoinHandle<()> {
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("fake server");
    })
}
