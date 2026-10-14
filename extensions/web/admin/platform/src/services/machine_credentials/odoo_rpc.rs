//! Odoo's `common.authenticate` over JSON-RPC: the one call a link makes to
//! prove a login and API key work before they are stored, and to learn the
//! uid they act as.
//!
//! Odoo answers a uid for a good pair and `false` for a bad one, both with
//! HTTP 200, so the result is read as "a positive integer or nothing".

use serde::{Deserialize, Serialize};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

#[derive(Serialize)]
struct RpcCall<'a> {
    jsonrpc: &'static str,
    method: &'static str,
    id: u32,
    params: RpcParams<'a>,
}

type NoContext = std::collections::BTreeMap<&'static str, &'static str>;

#[derive(Serialize)]
struct RpcParams<'a> {
    service: &'static str,
    method: &'static str,
    args: (&'a str, &'a str, &'a str, NoContext),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Uid {
    Id(i64),
    Refused(serde::de::IgnoredAny),
}

#[derive(Deserialize)]
struct RpcReply {
    #[serde(default)]
    result: Option<Uid>,
    #[serde(default)]
    error: Option<RpcError>,
}

#[derive(Deserialize)]
struct RpcError {
    #[serde(default)]
    message: String,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct OdooLogin<'a> {
    pub url: &'a str,
    pub db: &'a str,
    pub login: &'a str,
    pub api_key: &'a str,
}

pub(super) async fn authenticate(
    http: &reqwest::Client,
    login: OdooLogin<'_>,
) -> AdminResult<Option<i64>> {
    let call = RpcCall {
        jsonrpc: "2.0",
        method: "call",
        id: 1,
        params: RpcParams {
            service: "common",
            method: "authenticate",
            args: (login.db, login.login, login.api_key, NoContext::new()),
        },
    };
    let response = http
        .post(format!("{}/jsonrpc", login.url))
        .json(&call)
        .send()
        .await
        .map_err(|error| {
            tracing::warn!(target: "connector_broker", %error, "Odoo authenticate unreachable");
            AdminError::Upstream("Odoo could not be reached".to_owned())
        })?;
    if !response.status().is_success() {
        return Err(AdminError::Upstream(format!(
            "Odoo answered HTTP {}",
            response.status().as_u16()
        )));
    }
    let reply: RpcReply = response.json().await.map_err(|_error| {
        AdminError::Upstream("Odoo answered something other than JSON-RPC".to_owned())
    })?;
    if let Some(error) = reply.error {
        tracing::warn!(target: "connector_broker", message = %error.message,
            "Odoo authenticate returned an error");
        return Err(AdminError::Upstream(
            "Odoo refused the authenticate call".to_owned(),
        ));
    }
    Ok(match reply.result {
        Some(Uid::Id(uid)) if uid > 0 => Some(uid),
        _ => None,
    })
}
