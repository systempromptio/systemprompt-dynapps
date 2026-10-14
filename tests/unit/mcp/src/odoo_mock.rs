//! An in-process Odoo: the whole integration surface is one POST `/jsonrpc`.
//!
//! `common.authenticate` proves a key and returns a uid; `object.execute_kw`
//! serves `read` and `search_read` from per-model fixtures and records every
//! call, so a test can assert both what a guarded tool answered and whether it
//! wrote anything to Odoo before refusing.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

pub(crate) const UID: i64 = 7;
pub(crate) const GOOD_KEY: &str = "unit-odoo-key";

#[derive(Default)]
struct Odoo {
    records: HashMap<String, Vec<serde_json::Value>>,
    calls: Vec<(String, String)>,
    authentications: usize,
}

#[derive(Clone)]
pub(crate) struct OdooMock {
    pub(crate) url: String,
    state: Arc<Mutex<Odoo>>,
}

fn by_ids(records: &[serde_json::Value], ids: &serde_json::Value) -> Vec<serde_json::Value> {
    let wanted: Vec<i64> = ids
        .as_array()
        .map(|ids| ids.iter().filter_map(serde_json::Value::as_i64).collect())
        .unwrap_or_default();
    records
        .iter()
        .filter(|r| r["id"].as_i64().is_some_and(|id| wanted.contains(&id)))
        .cloned()
        .collect()
}

fn answer(odoo: &mut Odoo, params: &serde_json::Value) -> serde_json::Value {
    if params["method"] == "authenticate" {
        odoo.authentications += 1;
        let ok = params["args"][2].as_str() == Some(GOOD_KEY);
        return if ok {
            serde_json::json!(UID)
        } else {
            serde_json::json!(false)
        };
    }
    let model = params["args"][3].as_str().unwrap_or_default().to_owned();
    let method = params["args"][4].as_str().unwrap_or_default().to_owned();
    let records = odoo.records.get(&model).cloned().unwrap_or_default();
    let value = match method.as_str() {
        "read" => serde_json::Value::Array(by_ids(&records, &params["args"][5][0])),
        "search_read" => serde_json::Value::Array(records),
        _ => serde_json::json!(true),
    };
    odoo.calls.push((model, method));
    value
}

async fn rpc(
    State(state): State<Arc<Mutex<Odoo>>>,
    Json(body): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let value = answer(&mut state.lock().expect("mock state"), &body["params"]);
    Json(serde_json::json!({ "jsonrpc": "2.0", "id": 1, "result": value }))
}

impl OdooMock {
    pub(crate) async fn start() -> Self {
        let state = Arc::new(Mutex::new(Odoo::default()));
        let app = Router::new()
            .route("/jsonrpc", post(rpc))
            .with_state(Arc::clone(&state));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind a loopback port");
        let addr = listener.local_addr().expect("bound address");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("mock serves");
        });
        Self {
            url: format!("http://{addr}"),
            state,
        }
    }

    pub(crate) fn put(&self, model: &str, records: &serde_json::Value) {
        let records = records.as_array().cloned().unwrap_or_default();
        self.state
            .lock()
            .expect("mock state")
            .records
            .insert(model.to_owned(), records);
    }

    pub(crate) fn wrote(&self, method: &str) -> Vec<String> {
        self.state
            .lock()
            .expect("mock state")
            .calls
            .iter()
            .filter(|(_, m)| m == method)
            .map(|(model, _)| model.clone())
            .collect()
    }

    pub(crate) fn authentications(&self) -> usize {
        self.state.lock().expect("mock state").authentications
    }
}
