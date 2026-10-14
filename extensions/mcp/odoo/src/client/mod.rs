//! Model-level Odoo calls, built on the JSON-RPC transport in [`rpc`].
//!
//! Every method here takes the acting caller's [`Credentials`] explicitly
//! rather than holding them on the client. That is not ceremony: the client is
//! shared across all callers of the server, and the credential changes per
//! request. A client that remembered a credential would be one refactor away
//! from executing one user's tool call as another user.
//!
//! The per-model wrappers over [`OdooClient::execute_kw`] live in [`crud`].

pub mod crud;
pub mod rpc;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::apps::{map_access_denied, map_missing_app};
use crate::error::OdooError;
pub use rpc::OdooConnection;

/// The acting caller's Odoo credential, resolved per request from the sealed
/// `odoo_api_key` machine credential its service account holds.
///
/// The connection travels with the credential because the credential names
/// it: one key is pinned to one Odoo URL and database.
#[derive(Clone)]
pub struct Credentials {
    pub conn: OdooConnection,
    pub login: String,
    pub uid: i32,
    pub api_key: String,
    pub footer: Option<String>,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("conn", &self.conn)
            .field("login", &self.login)
            .field("uid", &self.uid)
            .field("api_key", &"<redacted>")
            .field("footer", &self.footer)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct OdooClient {
    http: reqwest::Client,
    // Why: Odoo's `authenticate` is a full password-hash check, so the uid it
    // returns is remembered per (url, db, login) for the life of the process
    // rather than paid for on every tool call.
    uids: Arc<Mutex<HashMap<String, i32>>>,
}

/// Optional keyword arguments for a `search_read`.
#[derive(Debug, Default, Serialize)]
pub struct SearchOptions {
    pub fields: Vec<String>,
    pub limit: u32,
    pub order: Option<String>,
}

// JSON: Odoo JSON-RPC protocol boundary; records stay untyped until shaped.
/// One `execute_kw` invocation: which model method, with which positional and
/// keyword arguments.
///
/// Bundled rather than passed as four parameters because `execute_kw` is the
/// call every other method here funnels through, and a positional list that
/// long is easy to transpose — `model` and `method` are both strings.
#[derive(Debug)]
pub struct ModelCall<'a> {
    pub model: &'a str,
    pub method: &'a str,
    pub args: serde_json::Value,
    pub kwargs: serde_json::Value,
}

/// A `read_group` aggregation: what to count, and how to bucket it.
#[derive(Debug)]
pub struct GroupQuery<'a> {
    pub domain: serde_json::Value,
    pub fields: &'a [&'a str],
    pub group_by: &'a [&'a str],
}

fn uid_key(conn: &OdooConnection, login: &str) -> String {
    format!("{}|{}|{login}", conn.url, conn.db)
}

impl OdooClient {
    pub fn new() -> Result<Self, OdooError> {
        // Why: without a request timeout a stalled Odoo hangs the MCP tool
        // call until the *caller's* client gives up, which surfaces as a
        // silent dashboard timeout instead of a tool error.
        let http = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(OdooError::HttpClient)?;
        Ok(Self {
            http,
            uids: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub async fn authenticate(
        &self,
        conn: &OdooConnection,
        login: &str,
        api_key: &str,
    ) -> Result<Option<i32>, OdooError> {
        let args = [
            serde_json::json!(conn.db),
            serde_json::json!(login),
            serde_json::json!(api_key),
            serde_json::json!({}),
        ];
        let result = rpc::call(&self.http, conn, "common", "authenticate", &args).await?;
        Ok(result.as_i64().and_then(|uid| i32::try_from(uid).ok()))
    }

    fn cached_uid(&self, key: &str) -> Option<i32> {
        self.uids
            .lock()
            .ok()
            .and_then(|uids| uids.get(key).copied())
    }

    fn remember_uid(&self, key: String, uid: i32) {
        if let Ok(mut uids) = self.uids.lock() {
            uids.insert(key, uid);
        }
    }

    pub async fn sign_in(
        &self,
        conn: OdooConnection,
        login: String,
        api_key: String,
    ) -> Result<Credentials, OdooError> {
        let key = uid_key(&conn, &login);
        if let Some(uid) = self.cached_uid(&key) {
            return Ok(Credentials {
                conn,
                login,
                uid,
                api_key,
                footer: None,
            });
        }
        let uid = self
            .authenticate(&conn, &login, &api_key)
            .await?
            .ok_or_else(|| {
                OdooError::AccessDenied(format!(
                    "Odoo rejected the API key for `{login}`. Rotate the odoo_api_key machine \
                     credential on the service account."
                ))
            })?;
        self.remember_uid(key, uid);
        Ok(Credentials {
            conn,
            login,
            uid,
            api_key,
            footer: None,
        })
    }

    // JSON: Odoo JSON-RPC protocol boundary; records stay untyped until shaped.
    fn execute_kw_args(
        uid: i32,
        creds: &Credentials,
        call: &ModelCall<'_>,
    ) -> [serde_json::Value; 7] {
        [
            serde_json::json!(creds.conn.db),
            serde_json::json!(uid),
            serde_json::json!(creds.api_key),
            serde_json::json!(call.model),
            serde_json::json!(call.method),
            call.args.clone(),
            call.kwargs.clone(),
        ]
    }

    pub async fn execute_kw(
        &self,
        creds: &Credentials,
        call: ModelCall<'_>,
    ) -> Result<serde_json::Value, OdooError> {
        let first = rpc::call(
            &self.http,
            &creds.conn,
            "object",
            "execute_kw",
            &Self::execute_kw_args(creds.uid, creds, &call),
        )
        .await;

        // Why: every Odoo model call funnels through here, so this is the one place
        // a fault can be recognised while the model name and the acting login
        // are still in hand. Callers get an error naming the app or the
        // credential, not the table.
        let err = match first {
            Ok(value) => return Ok(value),
            Err(e) => map_missing_app(call.model, e),
        };
        let err = map_access_denied(&creds.login, call.model, err);

        // Why: Odoo raises AccessDenied when a cached uid stops matching the
        // login — a re-provisioned Odoo account, a database restored from elsewhere —
        // makes Odoo raise AccessDenied on a key that is perfectly good, so ask
        // Odoo who this key is before believing the diagnosis.
        if !matches!(err, OdooError::AccessDenied(_)) {
            return Err(err);
        }
        let Ok(Some(fresh_uid)) = self
            .authenticate(&creds.conn, &creds.login, &creds.api_key)
            .await
        else {
            return Err(err);
        };
        if fresh_uid == creds.uid {
            return Err(err);
        }
        tracing::warn!(
            login = %creds.login,
            stale_uid = creds.uid,
            fresh_uid,
            "Cached Odoo uid was stale; refreshed it from the credential and retrying"
        );
        self.remember_uid(uid_key(&creds.conn, &creds.login), fresh_uid);
        rpc::call(
            &self.http,
            &creds.conn,
            "object",
            "execute_kw",
            &Self::execute_kw_args(fresh_uid, creds, &call),
        )
        .await
        .map_err(|e| map_missing_app(call.model, e))
        .map_err(|e| map_access_denied(&creds.login, call.model, e))
    }
}

// JSON: protocol boundary
// Why: execute_kw takes a positional args list whose first element is the id
// list, so the wire shape is `[[id]]`; a flat `[id]` is the classic Odoo
// mistake and unlinks nothing.
#[must_use]
pub fn unlink_args(ids: &[i64]) -> serde_json::Value {
    serde_json::json!([ids])
}
