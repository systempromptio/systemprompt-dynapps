//! The JSON-RPC transport: one POST to `{url}/jsonrpc`, one envelope in,
//! one envelope out.
//!
//! Odoo's external API is JSON-RPC 2.0 with two services. `common` handles
//! `authenticate`; `object` handles `execute_kw`, which is the door to every
//! model method. Keeping the envelope handling here means the model-level code
//! in the parent module reads as Odoo calls, not as HTTP.

use serde::Deserialize;

use crate::error::OdooError;

/// Where Odoo lives, as named by the caller's machine credential.
#[derive(Debug, Clone)]
pub struct OdooConnection {
    pub url: String,
    pub db: String,
}

impl OdooConnection {
    pub fn new(url: &str, db: &str) -> Result<Self, OdooError> {
        let url = url.trim().trim_end_matches('/').to_owned();
        let db = db.trim().to_owned();
        if url.is_empty() || db.is_empty() {
            return Err(OdooError::NotConfigured(
                "the Odoo machine credential must name both a URL and a database".to_owned(),
            ));
        }
        Ok(Self { url, db })
    }

    #[must_use]
    pub fn endpoint(&self) -> String {
        format!("{}/jsonrpc", self.url)
    }

    // Why: the `/web#` fragment form opens a record on every Odoo version this
    // server has been pointed at; the newer `/odoo/<model>/<id>` route does
    // not exist before 17.
    #[must_use]
    pub fn record_url(&self, model: &str, id: i64) -> String {
        format!("{}/web#id={id}&model={model}&view_type=form", self.url)
    }
}

#[derive(Deserialize)]
struct Envelope {
    // JSON: protocol boundary — `result` is whatever the called method
    // returns, so it stays untyped until the caller shapes it.
    result: Option<serde_json::Value>,
    error: Option<Fault>,
}

#[derive(Deserialize)]
struct Fault {
    message: String,
    #[serde(default)]
    data: Option<FaultData>,
}

#[derive(Deserialize)]
struct FaultData {
    #[serde(default)]
    message: Option<String>,
}

#[doc(hidden)]
#[must_use]
pub fn build_request(service: &str, method: &str, args: &[serde_json::Value]) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "method": "call",
        "id": 1,
        "params": { "service": service, "method": method, "args": args },
    })
}

// JSON: Odoo JSON-RPC protocol boundary; records stay untyped until shaped.
#[doc(hidden)]
pub fn parse_response(body: &str) -> Result<serde_json::Value, OdooError> {
    let envelope: Envelope =
        serde_json::from_str(body).map_err(|e| OdooError::Transport(e.to_string()))?;
    if let Some(fault) = envelope.error {
        let detail = fault
            .data
            .and_then(|d| d.message)
            .map(|m| m.trim().to_owned())
            .filter(|m| !m.is_empty())
            .unwrap_or(fault.message);
        return Err(OdooError::Odoo(detail));
    }
    Ok(envelope.result.unwrap_or(serde_json::Value::Null))
}

pub async fn call(
    http: &reqwest::Client,
    conn: &OdooConnection,
    service: &str,
    method: &str,
    args: &[serde_json::Value],
) -> Result<serde_json::Value, OdooError> {
    let request = build_request(service, method, args);
    let resp = http
        .post(conn.endpoint())
        .json(&request)
        .send()
        .await
        .map_err(|e| OdooError::Transport(e.to_string()))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| OdooError::Transport(e.to_string()))?;

    if !status.is_success() {
        // Why: Odoo answers an application fault with HTTP 200 and an `error`
        // member, so a non-2xx is the proxy, the URL, or the server being
        // down — never a model-level refusal, and the body is not an envelope.
        return Err(OdooError::Transport(format!("Odoo returned HTTP {status}")));
    }

    parse_response(&body)
}
