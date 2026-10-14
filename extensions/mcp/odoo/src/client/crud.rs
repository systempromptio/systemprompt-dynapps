//! The model methods every tool actually calls: read, write, aggregate, post.
//!
//! Each is a thin, single-shape wrapper over [`OdooClient::execute_kw`] in
//! [`super`], which owns the transport, the credential and the stale-uid
//! recovery. Kept apart from it so that the one method with real behaviour is
//! not buried in a dozen that only name a `model` and a `method`.

use crate::client::{Credentials, GroupQuery, ModelCall, OdooClient, SearchOptions, unlink_args};
use crate::error::OdooError;

// Why: Odoo 17+ escapes a str body posted over RPC unless `body_is_html` is
// set, so every post goes as HTML and plain text is escaped into a paragraph.
#[must_use]
pub fn as_html(body: &str) -> String {
    let body = body.trim();
    if body.starts_with('<') {
        return body.to_owned();
    }
    let escaped = body
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\n', "<br>");
    format!("<p>{escaped}</p>")
}

// Why: Odoo names the bot login as the author of every post the farm makes,
// so the footer is what tells a reader which person or run acted.
#[must_use]
pub fn with_footer(body: &str, footer: Option<&str>) -> String {
    let body = as_html(body);
    match footer {
        Some(footer) => format!("{body}<p><em>{footer}</em></p>"),
        None => body,
    }
}

impl OdooClient {
    // JSON: Odoo JSON-RPC protocol boundary; records stay untyped until shaped.
    pub async fn search_read(
        &self,
        creds: &Credentials,
        model: &str,
        domain: serde_json::Value,
        options: &SearchOptions,
    ) -> Result<Vec<serde_json::Value>, OdooError> {
        let mut kwargs = serde_json::Map::new();
        kwargs.insert("fields".to_owned(), serde_json::json!(options.fields));
        kwargs.insert("limit".to_owned(), serde_json::json!(options.limit));
        if let Some(order) = &options.order {
            kwargs.insert("order".to_owned(), serde_json::json!(order));
        }
        let result = self
            .execute_kw(
                creds,
                ModelCall {
                    model,
                    method: "search_read",
                    args: serde_json::json!([domain]),
                    kwargs: serde_json::Value::Object(kwargs),
                },
            )
            .await?;
        Ok(result.as_array().cloned().unwrap_or_default())
    }

    // JSON: Odoo JSON-RPC protocol boundary; records stay untyped until shaped.
    pub async fn read(
        &self,
        creds: &Credentials,
        model: &str,
        ids: &[i64],
        fields: &[&str],
    ) -> Result<Vec<serde_json::Value>, OdooError> {
        let result = self
            .execute_kw(
                creds,
                ModelCall {
                    model,
                    method: "read",
                    args: serde_json::json!([ids]),
                    kwargs: serde_json::json!({ "fields": fields }),
                },
            )
            .await?;
        Ok(result.as_array().cloned().unwrap_or_default())
    }

    // JSON: Odoo JSON-RPC protocol boundary; records stay untyped until shaped.
    pub async fn read_group(
        &self,
        creds: &Credentials,
        model: &str,
        query: GroupQuery<'_>,
    ) -> Result<Vec<serde_json::Value>, OdooError> {
        let result = self
            .execute_kw(
                creds,
                ModelCall {
                    model,
                    method: "read_group",
                    args: serde_json::json!([query.domain, query.fields, query.group_by]),
                    kwargs: serde_json::json!({ "lazy": false }),
                },
            )
            .await?;
        Ok(result.as_array().cloned().unwrap_or_default())
    }

    pub async fn create(
        &self,
        creds: &Credentials,
        model: &str,
        values: serde_json::Value,
    ) -> Result<i64, OdooError> {
        let result = self
            .execute_kw(
                creds,
                ModelCall {
                    model,
                    method: "create",
                    args: serde_json::json!([values]),
                    kwargs: serde_json::json!({}),
                },
            )
            .await?;
        result
            .as_i64()
            .ok_or_else(|| OdooError::Odoo(format!("create on {model} returned no record id")))
    }

    // JSON: Odoo JSON-RPC protocol boundary; records stay untyped until shaped.
    pub async fn write(
        &self,
        creds: &Credentials,
        model: &str,
        id: i64,
        values: serde_json::Value,
    ) -> Result<bool, OdooError> {
        let result = self
            .execute_kw(
                creds,
                ModelCall {
                    model,
                    method: "write",
                    args: serde_json::json!([[id], values]),
                    kwargs: serde_json::json!({}),
                },
            )
            .await?;
        Ok(result.as_bool().unwrap_or(false))
    }

    pub async fn unlink(
        &self,
        creds: &Credentials,
        model: &str,
        ids: &[i64],
    ) -> Result<bool, OdooError> {
        let result = self
            .execute_kw(
                creds,
                ModelCall {
                    model,
                    method: "unlink",
                    args: unlink_args(ids),
                    kwargs: serde_json::json!({}),
                },
            )
            .await?;
        Ok(result.as_bool().unwrap_or(false))
    }

    pub async fn message_post(
        &self,
        creds: &Credentials,
        model: &str,
        res_id: i64,
        body: &str,
    ) -> Result<i64, OdooError> {
        // Why: Odoo's message_post is also its sending path — it mails
        // partner_ids and picks delivery from the subtype. Taking neither is
        // what stops a caller that already sent the message from making Odoo
        // deliver it a second time.
        let result = self
            .execute_kw(
                creds,
                ModelCall {
                    model,
                    method: "message_post",
                    args: serde_json::json!([[res_id]]),
                    kwargs: serde_json::json!({
                        "body": with_footer(body, creds.footer.as_deref()),
                        "message_type": "comment",
                        "body_is_html": true,
                    }),
                },
            )
            .await?;
        Ok(result.as_i64().unwrap_or_default())
    }
}
