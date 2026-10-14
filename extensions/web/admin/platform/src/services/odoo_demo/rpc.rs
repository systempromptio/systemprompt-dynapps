//! The JSON-RPC session the demo seeder talks to Odoo through.
//!
//! One session per seed: authenticate once as the farm's service account,
//! then every `execute_kw` reuses the uid. Odoo reports a failed call as a
//! JSON-RPC `error` member under HTTP 200, so the reply body, not the status,
//! decides success.

use serde_json::{Value, json};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

use crate::services::machine_credentials::OdooMachineCredential;

#[derive(Debug)]
pub(super) struct OdooSession {
    http: reqwest::Client,
    url: String,
    db: String,
    uid: i64,
    api_key: String,
}

impl OdooSession {
    pub(super) async fn sign_in(credential: &OdooMachineCredential) -> AdminResult<Self> {
        let http = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(AdminError::internal)?;
        let args = json!([credential.db, credential.login, credential.api_key, {}]);
        let uid = call(&http, &credential.url, "common", "authenticate", args)
            .await?
            .as_i64()
            .filter(|uid| *uid > 0)
            .ok_or_else(|| {
                AdminError::Upstream("Odoo refused the farm bot's API key".to_owned())
            })?;
        Ok(Self {
            http,
            url: credential.url.trim_end_matches('/').to_owned(),
            db: credential.db.clone(),
            uid,
            api_key: credential.api_key.clone(),
        })
    }

    // JSON: Odoo JSON-RPC protocol boundary; records stay untyped here.
    pub(super) async fn kw(
        &self,
        model: &str,
        method: &str,
        args: Value,
        kwargs: Value,
    ) -> AdminResult<Value> {
        let call_args = json!([self.db, self.uid, self.api_key, model, method, args, kwargs]);
        call(&self.http, &self.url, "object", "execute_kw", call_args).await
    }

    pub(super) async fn search_one(&self, model: &str, domain: Value) -> AdminResult<Option<i64>> {
        let kwargs = json!({"limit": 1, "order": "id", "context": {"active_test": false}});
        let ids = self.kw(model, "search", json!([domain]), kwargs).await?;
        Ok(ids.get(0).and_then(Value::as_i64))
    }

    pub(super) async fn search_all(&self, model: &str, domain: Value) -> AdminResult<Vec<i64>> {
        let kwargs = json!({"context": {"active_test": false}});
        let ids = self.kw(model, "search", json!([domain]), kwargs).await?;
        Ok(ids
            .as_array()
            .map(|ids| ids.iter().filter_map(Value::as_i64).collect())
            .unwrap_or_default())
    }

    pub(super) async fn count(&self, model: &str, domain: Value) -> AdminResult<i64> {
        let n = self
            .kw(model, "search_count", json!([domain]), json!({}))
            .await?;
        Ok(n.as_i64().unwrap_or(0))
    }

    pub(super) async fn create(&self, model: &str, values: Value) -> AdminResult<i64> {
        self.kw(model, "create", json!([values]), json!({}))
            .await?
            .as_i64()
            .ok_or_else(|| AdminError::Upstream(format!("Odoo create on {model} returned no id")))
    }

    pub(super) async fn write(&self, model: &str, ids: &[i64], values: Value) -> AdminResult<()> {
        if ids.is_empty() {
            return Ok(());
        }
        self.kw(model, "write", json!([ids, values]), json!({}))
            .await?;
        Ok(())
    }
}

async fn call(
    http: &reqwest::Client,
    url: &str,
    service: &str,
    method: &str,
    args: Value,
) -> AdminResult<Value> {
    let body = json!({
        "jsonrpc": "2.0",
        "method": "call",
        "id": 1,
        "params": {"service": service, "method": method, "args": args},
    });
    let response = http
        .post(format!("{}/jsonrpc", url.trim_end_matches('/')))
        .json(&body)
        .send()
        .await
        .map_err(|error| {
            tracing::warn!(target: "odoo_demo", %error, "Odoo unreachable");
            AdminError::Upstream("Odoo could not be reached".to_owned())
        })?;
    if !response.status().is_success() {
        return Err(AdminError::Upstream(format!(
            "Odoo answered HTTP {}",
            response.status().as_u16()
        )));
    }
    let mut reply: Value = response.json().await.map_err(|_error| {
        AdminError::Upstream("Odoo answered something other than JSON-RPC".to_owned())
    })?;
    if let Some(error) = reply.get("error") {
        let message = error
            .pointer("/data/message")
            .or_else(|| error.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("unknown error");
        return Err(AdminError::Upstream(format!(
            "Odoo {service}.{method}: {message}"
        )));
    }
    Ok(reply.get_mut("result").map_or(Value::Null, Value::take))
}
