//! A 1Password Connect server on loopback HTTP, for the secrets resolver.
//!
//! Serves the three read routes the resolver uses -- vault list, item list,
//! item detail -- with Connect's `filter=name eq "x"` / `title eq "x"`
//! queries and bearer-token auth. Failure modes are armed at runtime: a
//! queue of forced statuses ([`FakeConnect::fail_next`],
//! [`FakeConnect::rate_limit_once`]) consumed one per request, and a fixed
//! per-request [`FakeConnect::delay`].
//!
//! A field is `STRING` when its label is `username`, `url` or `notes`, and
//! `CONCEALED` otherwise, which is how Connect types a login item's fields.

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::http::{HeaderMap, Method, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use tokio::task::JoinHandle;

use crate::fake_http::{RecordedRequest, RequestLog, bind, record, snapshot, spawn};

#[derive(Debug, Clone, Default)]
struct Item {
    title: String,
    fields: Vec<(String, String, String)>,
}

#[derive(Debug, Clone, Default)]
struct Vault {
    name: String,
    items: BTreeMap<String, Item>,
}

#[derive(Debug, Default)]
struct Inner {
    token: String,
    vaults: Mutex<BTreeMap<String, Vault>>,
    failures: Mutex<VecDeque<StatusCode>>,
    delay: Mutex<Duration>,
    requests: RequestLog,
}

#[derive(Debug)]
pub struct FakeConnect {
    inner: Arc<Inner>,
    base_url: String,
    task: JoinHandle<()>,
}

impl FakeConnect {
    pub async fn start(token: &str) -> Self {
        let (listener, base_url) = bind().await;
        let inner = Arc::new(Inner {
            token: token.to_owned(),
            ..Inner::default()
        });
        let state = Arc::clone(&inner);
        let router = Router::new().fallback(move |method: Method, uri: Uri, headers: HeaderMap| {
            let state = Arc::clone(&state);
            async move { handle(&state, &method, &uri, &headers).await }
        });
        let task = spawn(listener, router);
        Self { inner, base_url, task }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn add_vault(&self, name: &str) -> String {
        let id = new_id();
        let vault = Vault { name: name.to_owned(), items: BTreeMap::new() };
        self.inner.vaults.lock().expect("vaults").insert(id.clone(), vault);
        id
    }

    pub fn add_item(&self, vault_id: &str, title: &str, fields: &[(&str, &str)]) -> String {
        let id = new_id();
        let fields = fields
            .iter()
            .map(|(label, value)| (new_id(), (*label).to_owned(), (*value).to_owned()))
            .collect();
        let item = Item { title: title.to_owned(), fields };
        self.with_vault(vault_id, |v| v.items.insert(id.clone(), item));
        id
    }

    /// Overwrite `label` on an item, adding the field when it is absent.
    pub fn set_field(&self, vault_id: &str, item_id: &str, label: &str, value: &str) {
        self.with_vault(vault_id, |v| {
            let item = v.items.get_mut(item_id).expect("unknown item id");
            match item.fields.iter_mut().find(|(_, l, _)| l == label) {
                Some(field) => value.clone_into(&mut field.2),
                None => item.fields.push((new_id(), label.to_owned(), value.to_owned())),
            }
        });
    }

    /// The next `n` requests answer `status` before any routing or auth.
    pub fn fail_next(&self, n: usize, status: StatusCode) {
        let mut queue = self.inner.failures.lock().expect("failures");
        queue.extend(std::iter::repeat_n(status, n));
    }

    /// The next request answers 429; the one after it is served normally.
    pub fn rate_limit_once(&self) {
        self.fail_next(1, StatusCode::TOO_MANY_REQUESTS);
    }

    pub fn delay(&self, delay: Duration) {
        *self.inner.delay.lock().expect("delay") = delay;
    }

    pub fn requests(&self) -> Vec<RecordedRequest> {
        snapshot(&self.inner.requests)
    }

    pub fn shutdown(&self) {
        self.task.abort();
    }

    fn with_vault<R>(&self, vault_id: &str, f: impl FnOnce(&mut Vault) -> R) -> R {
        let mut vaults = self.inner.vaults.lock().expect("vaults");
        f(vaults.get_mut(vault_id).expect("unknown vault id"))
    }

    /// A snapshot of every vault, for [`crate::onepassword_fake_cli`].
    pub(crate) fn export(&self) -> Value {
        let out: Vec<Value> = self
            .inner
            .vaults
            .lock()
            .expect("vaults")
            .iter()
            .map(|(id, v)| {
                let items: Vec<Value> = v.items.iter().map(|(iid, i)| item_json(id, iid, i)).collect();
                json!({ "id": id, "name": v.name, "items": items })
            })
            .collect();
        json!({ "vaults": out })
    }
}

impl Drop for FakeConnect {
    fn drop(&mut self) {
        self.task.abort();
    }
}

fn new_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..26].to_owned()
}

fn field_type(label: &str) -> &'static str {
    if matches!(label, "username" | "url" | "notes") { "STRING" } else { "CONCEALED" }
}

fn item_json(vault_id: &str, item_id: &str, item: &Item) -> Value {
    let fields: Vec<Value> = item
        .fields
        .iter()
        .map(|(id, label, value)| json!({ "id": id, "label": label, "type": field_type(label), "value": value }))
        .collect();
    json!({ "id": item_id, "title": item.title, "vault": { "id": vault_id }, "category": "LOGIN", "fields": fields })
}

fn error(status: StatusCode, message: &str) -> Response {
    let body = axum::Json(json!({ "status": status.as_u16(), "message": message }));
    if status == StatusCode::TOO_MANY_REQUESTS {
        return (status, [(header::RETRY_AFTER, "1")], body).into_response();
    }
    (status, body).into_response()
}

/// The value of a Connect `filter=<attr> eq "<value>"` query, if present.
fn filter(uri: &Uri, attr: &str) -> Option<String> {
    let query = uri.query()?;
    let raw = url::form_urlencoded::parse(query.as_bytes())
        .find(|(k, _)| k == "filter")
        .map(|(_, v)| v.into_owned())?;
    let rest = raw.strip_prefix(attr)?.trim_start().strip_prefix("eq")?.trim();
    Some(rest.trim_matches('"').to_owned())
}

async fn handle(inner: &Inner, method: &Method, uri: &Uri, headers: &HeaderMap) -> Response {
    let authorization = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    record(
        &inner.requests,
        RecordedRequest {
            method: method.to_string(),
            path: uri.path().to_owned(),
            query: uri.query().map(str::to_owned),
            authorization: authorization.clone(),
            body: String::new(),
        },
    );
    let delay = *inner.delay.lock().expect("delay");
    if !delay.is_zero() {
        tokio::time::sleep(delay).await;
    }
    let injected = inner.failures.lock().expect("failures").pop_front();
    if let Some(status) = injected {
        return error(status, "injected failure");
    }
    if authorization.as_deref() != Some(&format!("Bearer {}", inner.token)) {
        return error(StatusCode::UNAUTHORIZED, "Invalid token signature");
    }
    if method != Method::GET {
        return error(StatusCode::METHOD_NOT_ALLOWED, "read-only fake");
    }
    let segments: Vec<&str> = uri.path().trim_matches('/').split('/').collect();
    let vaults = inner.vaults.lock().expect("vaults");
    match segments.as_slice() {
        ["v1", "vaults"] => {
            let wanted = filter(uri, "name");
            let list: Vec<Value> = vaults
                .iter()
                .filter(|(_, v)| wanted.as_ref().is_none_or(|w| *w == v.name))
                .map(|(id, v)| json!({ "id": id, "name": v.name }))
                .collect();
            axum::Json(list).into_response()
        },
        ["v1", "vaults", vault_id, "items"] => {
            let Some(vault) = vaults.get(*vault_id) else {
                return error(StatusCode::NOT_FOUND, "vault not found");
            };
            let wanted = filter(uri, "title");
            let list: Vec<Value> = vault
                .items
                .iter()
                .filter(|(_, i)| wanted.as_ref().is_none_or(|w| *w == i.title))
                .map(|(id, i)| json!({ "id": id, "title": i.title, "vault": { "id": vault_id } }))
                .collect();
            axum::Json(list).into_response()
        },
        ["v1", "vaults", vault_id, "items", item_id] => {
            vaults.get(*vault_id).and_then(|v| v.items.get(*item_id)).map_or_else(
                || error(StatusCode::NOT_FOUND, "item not found"),
                |item| axum::Json(item_json(vault_id, item_id, item)).into_response(),
            )
        },
        _ => error(StatusCode::NOT_FOUND, "no such route"),
    }
}
