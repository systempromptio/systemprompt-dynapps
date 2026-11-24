//! A connector that must be signed in again is not an outage. The page's
//! call goes through core's real proxy to an external server whose accessor
//! answers as the extension's connector accessor does: 404 for a grant the
//! provider refused to refresh (core: not connected, 409
//! `provider_not_connected`), which the broker answers `needs_reauth`, and
//! 503 for an accessor that is down (core: 503), which stays
//! `server_unavailable`. The listing says the same as `authStatus`. That the
//! accessor answers 404 for a refused refresh is the fake portal's case
//! (`tests/integration/admin-config/src/portal_e2e.rs`).

use std::sync::atomic::Ordering;

use serde_json::json;
use systemprompt::identifiers::{DocumentId, UserId};

use super::stack::{As, REMOTE, REMOTE_TOOL, Stack};

async fn declare(pool: &sqlx::PgPool, page: &DocumentId, owner: &UserId) {
    sqlx::query(
        "INSERT INTO artifact_capabilities (document_id, capabilities, contract_version, declared_by)
         VALUES ($1::uuid, $2, '0.2.99', $3)",
    )
    .bind(page.as_str())
    .bind(json!({ "mcp": { "servers": [{ "server": REMOTE, "tools": [REMOTE_TOOL] }] } }))
    .bind(owner.as_str())
    .execute(pool)
    .await
    .expect("declare the remote server's tool on the page");
}

async fn call(stack: &Stack, cookie: &str, page: &DocumentId) -> super::stack::Reply {
    stack
        .send(
            reqwest::Method::POST,
            &format!("/api/v1/artifacts/{}/mcp/call", page.as_str()),
            As::Cookie(cookie),
            Some(&json!({ "server": REMOTE, "tool": REMOTE_TOOL, "input": {} })),
        )
        .await
}

async fn listed(stack: &Stack, cookie: &str, page: &DocumentId) -> serde_json::Value {
    let reply = stack
        .send(
            reqwest::Method::POST,
            &format!("/api/v1/artifacts/{}/mcp/tools", page.as_str()),
            As::Cookie(cookie),
            Some(&json!({ "server": REMOTE })),
        )
        .await;
    assert_eq!(
        reply.status, 200,
        "the page lists its servers: {}",
        reply.body
    );
    reply.json()["servers"].clone()
}

pub(super) async fn a_refused_grant_asks_to_reconnect(
    stack: &Stack,
    pool: &sqlx::PgPool,
    owner: &UserId,
    cookie: &str,
) -> DocumentId {
    let page =
        Box::pin(stack.named_document(owner, "dynapps-html", "artifacts-broker-remote")).await;
    declare(pool, &page, owner).await;
    let granted = stack
        .send(
            reqwest::Method::POST,
            &format!("/api/v1/artifacts/{}/permissions", page.as_str()),
            As::Cookie(cookie),
            Some(&json!({ "scope": format!("mcp:{REMOTE}"), "decision": "granted" })),
        )
        .await;
    assert_eq!(
        granted.status, 200,
        "the viewer allows the server: {}",
        granted.body
    );

    stack.accessor.store(404, Ordering::SeqCst);
    let rejected = call(stack, cookie, &page).await;
    assert_eq!(
        rejected.status, 409,
        "a grant to sign in again: {}",
        rejected.body
    );
    assert_eq!(
        rejected.key(),
        "needs_reauth",
        "the page is asked to reconnect"
    );
    assert_eq!(
        listed(stack, cookie, &page).await,
        json!([{ "server": REMOTE, "authStatus": "needs_reauth", "tools": [] }]),
        "the listing says the same"
    );

    stack.accessor.store(503, Ordering::SeqCst);
    let down = call(stack, cookie, &page).await;
    assert_eq!(down.status, 503, "an accessor that is down: {}", down.body);
    assert_eq!(
        down.key(),
        "server_unavailable",
        "a real outage stays an outage"
    );
    page
}
