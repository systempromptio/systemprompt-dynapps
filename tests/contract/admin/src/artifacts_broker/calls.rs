//! A page's tool calls through the broker: what the manifest and the
//! viewer's consent let through, a write's one-shot approval, the cache of a
//! read, and what the server itself refuses.

use serde_json::{Value, json};
use systemprompt::identifiers::DocumentId;

use super::consent;
use super::fake::{ECHO, SECRET, WRITE, runs};
use super::stack::{As, Reply, SERVER, Stack};

fn path(document: &DocumentId, verb: &str) -> String {
    format!("/api/v1/artifacts/{}/mcp/{verb}", document.as_str())
}

pub(super) async fn call(stack: &Stack, who: As<'_>, document: &DocumentId, body: &Value) -> Reply {
    stack
        .send(
            reqwest::Method::POST,
            &path(document, "call"),
            who,
            Some(body),
        )
        .await
}

pub(super) fn tool(server: &str, name: &str, extra: &Value) -> Value {
    let mut body = json!({ "server": server, "tool": name, "input": { "text": "hola" } });
    if let (Some(body), Some(extra)) = (body.as_object_mut(), extra.as_object()) {
        body.extend(extra.clone());
    }
    body
}

fn refused(reply: &Reply, status: u16, key: &str, why: &str) {
    assert_eq!(reply.status, status, "{why}: {}", reply.body);
    assert_eq!(reply.key(), key, "{why}");
}

pub(super) async fn before_consent(stack: &Stack, owner: &str, page: &DocumentId) {
    let ask = call(
        stack,
        As::Cookie(owner),
        page,
        &tool(SERVER, ECHO, &json!({})),
    )
    .await;
    refused(
        &ask,
        428,
        "approval_required",
        "a declared server the viewer has not allowed yet",
    );
    assert_eq!(runs(ECHO), 0, "nothing ran before the viewer allowed it");
    let undeclared = call(
        stack,
        As::Cookie(owner),
        page,
        &tool(SERVER, SECRET, &json!({})),
    )
    .await;
    refused(
        &undeclared,
        403,
        "not_in_manifest",
        "a tool the page does not declare",
    );
    let unknown = call(
        stack,
        As::Cookie(owner),
        page,
        &tool("no-such-server", ECHO, &json!({})),
    )
    .await;
    refused(
        &unknown,
        400,
        "bad_request",
        "a server this instance does not have",
    );
}

pub(super) async fn reads_and_their_cache(
    stack: &Stack,
    owner: &str,
    owner_id: &str,
    page: &DocumentId,
) {
    let first = call(
        stack,
        As::Cookie(owner),
        page,
        &tool(SERVER, ECHO, &json!({})),
    )
    .await;
    assert_eq!(first.status, 200, "an allowed read runs: {}", first.body);
    let result = first.json();
    assert_eq!(
        result["structuredContent"]["caller"], owner_id,
        "the server saw the viewer: {result}"
    );
    assert_eq!(
        result["structuredContent"]["text"], "hola",
        "the input reached the tool"
    );
    assert!(
        result["content"].is_array(),
        "the result keeps core's shape: {result}"
    );
    assert!(
        result.get("cache").is_none(),
        "a fresh result carries no cache marker: {result}"
    );

    let fresh = json!({ "cache": { "staleTime": 60_000 } });
    let cached = call(stack, As::Cookie(owner), page, &tool(SERVER, ECHO, &fresh)).await;
    assert_eq!(cached.status, 200, "{}", cached.body);
    assert_eq!(
        runs(ECHO),
        1,
        "a read inside its staleTime is served, not run"
    );
    let marker = &cached.json()["cache"];
    assert!(
        marker["storedAt"].as_i64().is_some_and(|at| at > 0),
        "served from the cache: {marker}"
    );
    assert_eq!(marker["revalidating"], false, "{marker}");

    let refresh = json!({ "cache": { "staleTime": 60_000, "refresh": true } });
    let refreshed = call(
        stack,
        As::Cookie(owner),
        page,
        &tool(SERVER, ECHO, &refresh),
    )
    .await;
    assert_eq!(refreshed.status, 200, "{}", refreshed.body);
    assert_eq!(runs(ECHO), 2, "`refresh` runs the read again");
    assert!(
        refreshed.json().get("cache").is_none(),
        "{}",
        refreshed.body
    );
    let uncached = call(
        stack,
        As::Cookie(owner),
        page,
        &tool(SERVER, ECHO, &json!({ "cache": false })),
    )
    .await;
    assert_eq!(uncached.status, 200, "{}", uncached.body);
    assert_eq!(runs(ECHO), 3, "`cache: false` always runs");
}

async fn approve(stack: &Stack, who: As<'_>, page: &DocumentId, text: &str) -> Reply {
    let body = json!({ "server": SERVER, "tool": WRITE, "input": { "text": text } });
    stack
        .send(
            reqwest::Method::POST,
            &path(page, "approve"),
            who,
            Some(&body),
        )
        .await
}

async fn approval(stack: &Stack, owner: &str, page: &DocumentId) -> String {
    let reply = approve(stack, As::Cookie(owner), page, "hola").await;
    assert_eq!(
        reply.status, 200,
        "the viewer's Allow gives an approval: {}",
        reply.body
    );
    reply.json()["approval"]
        .as_str()
        .unwrap_or_else(|| panic!("an approval id: {}", reply.body))
        .to_owned()
}

pub(super) async fn writes_run_once_per_approval(
    stack: &Stack,
    owner: &str,
    agent: &str,
    page: &DocumentId,
) {
    let unapproved = call(
        stack,
        As::Cookie(owner),
        page,
        &tool(SERVER, WRITE, &json!({})),
    )
    .await;
    refused(
        &unapproved,
        428,
        "approval_required",
        "a write without the viewer's approval",
    );
    assert_eq!(runs(WRITE), 0, "an unapproved write never runs");
    let by_agent = approve(stack, As::Bearer(agent), page, "hola").await;
    assert_eq!(
        by_agent.status, 401,
        "an agent never approves: {}",
        by_agent.body
    );

    let id = approval(stack, owner, page).await;
    let approved = json!({ "approval": id });
    let ran = call(
        stack,
        As::Cookie(owner),
        page,
        &tool(SERVER, WRITE, &approved),
    )
    .await;
    assert_eq!(ran.status, 200, "an approved write runs: {}", ran.body);
    assert_eq!(runs(WRITE), 1);
    let again = call(
        stack,
        As::Cookie(owner),
        page,
        &tool(SERVER, WRITE, &approved),
    )
    .await;
    refused(
        &again,
        428,
        "approval_required",
        "an approval is spent by its call",
    );
    assert_eq!(runs(WRITE), 1, "a spent approval runs nothing");

    let other = approval(stack, owner, page).await;
    let mut changed = tool(SERVER, WRITE, &json!({ "approval": other }));
    changed["input"]["text"] = json!("adiós");
    let mismatch = call(stack, As::Cookie(owner), page, &changed).await;
    refused(
        &mismatch,
        428,
        "approval_required",
        "an approval covers only the input it was given for",
    );

    for expected in [2, 3] {
        let cached = json!({ "approval": approval(stack, owner, page).await, "cache": { "staleTime": 60_000 } });
        let write = call(
            stack,
            As::Cookie(owner),
            page,
            &tool(SERVER, WRITE, &cached),
        )
        .await;
        assert_eq!(write.status, 200, "{}", write.body);
        assert!(
            write.json().get("cache").is_none(),
            "a write is never served from the cache"
        );
        assert_eq!(
            runs(WRITE),
            expected,
            "a write runs every time, cache or not"
        );
    }
}

// Why: the fake's `secret` is the admins' only, so the owner's agent could
// not declare it; the row stands for a tool whose access was tightened after
// the page declared it.
pub(super) async fn the_server_still_decides(
    stack: &Stack,
    pool: &sqlx::PgPool,
    owner: &str,
    page: &DocumentId,
) {
    sqlx::query(
        "UPDATE artifact_capabilities
            SET capabilities = jsonb_set(capabilities, '{mcp,servers,0,tools}',
                                         capabilities #> '{mcp,servers,0,tools}' || $2::jsonb)
          WHERE document_id = $1::uuid",
    )
    .bind(page.as_str())
    .bind(json!([SECRET]))
    .execute(pool)
    .await
    .expect("declare the admins' tool on the page");
    let secret = call(
        stack,
        As::Cookie(owner),
        page,
        &tool(SERVER, SECRET, &json!({})),
    )
    .await;
    refused(
        &secret,
        403,
        "blocked_by_policy",
        "a declared tool the viewer's role lacks",
    );
    assert_eq!(runs(SECRET), 0);
}

pub(super) async fn list_and_describe(stack: &Stack, owner: &str, page: &DocumentId) {
    let listed = stack
        .send(
            reqwest::Method::POST,
            &path(page, "tools"),
            As::Cookie(owner),
            Some(&json!({})),
        )
        .await;
    assert_eq!(listed.status, 200, "{}", listed.body);
    let servers = listed.json()["servers"].clone();
    assert_eq!(servers[0]["server"], SERVER, "{servers}");
    assert_eq!(servers[0]["authStatus"], "connected", "{servers}");
    let names: Vec<&str> = servers[0]["tools"]
        .as_array()
        .map(|tools| {
            tools
                .iter()
                .filter_map(|tool| tool["name"].as_str())
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(
        names,
        [ECHO, WRITE],
        "the manifest as far as the viewer reaches: {servers}"
    );
    assert_eq!(
        servers[0]["tools"][0]["annotations"]["readOnlyHint"], true,
        "{servers}"
    );

    let described = stack
        .send(
            reqwest::Method::POST,
            &path(page, "describe"),
            As::Cookie(owner),
            Some(&json!({ "server": SERVER, "tool": ECHO })),
        )
        .await;
    assert_eq!(described.status, 200, "{}", described.body);
    assert_eq!(
        described.json()["inputSchema"]["properties"]["text"]["type"],
        "string",
        "{}",
        described.body
    );
    let outside = stack
        .send(
            reqwest::Method::POST,
            &path(page, "describe"),
            As::Cookie(owner),
            Some(&json!({ "server": SERVER, "tool": super::fake::OFF })),
        )
        .await;
    refused(
        &outside,
        403,
        "not_in_manifest",
        "describing a tool the page does not declare",
    );
}

pub(super) async fn consent_gates_the_cache(stack: &Stack, owner: &str, page: &DocumentId) {
    let fresh = json!({ "cache": { "staleTime": 60_000 } });
    let revoked = consent::revoke(stack, owner, page).await;
    assert_eq!(revoked["states"][consent::scope()], "prompt", "{revoked}");
    let asked = call(stack, As::Cookie(owner), page, &tool(SERVER, ECHO, &fresh)).await;
    refused(
        &asked,
        428,
        "approval_required",
        "a revoked server is never served, cached or not",
    );
    let regranted = consent::decide(stack, As::Cookie(owner), page, "granted").await;
    assert_eq!(regranted.status, 200, "{}", regranted.body);
    let before = runs(ECHO);
    let rerun = call(stack, As::Cookie(owner), page, &tool(SERVER, ECHO, &fresh)).await;
    assert_eq!(rerun.status, 200, "{}", rerun.body);
    assert_eq!(
        runs(ECHO),
        before + 1,
        "revoking dropped the viewer's cached results"
    );

    let denied = consent::decide(stack, As::Cookie(owner), page, "denied").await;
    assert_eq!(
        denied.json()["states"][consent::scope()],
        "denied",
        "{}",
        denied.body
    );
    let declined = call(stack, As::Cookie(owner), page, &tool(SERVER, ECHO, &fresh)).await;
    refused(
        &declined,
        403,
        "not_in_manifest",
        "a server the viewer declined",
    );
    let regranted = consent::decide(stack, As::Cookie(owner), page, "granted").await;
    assert_eq!(regranted.status, 200, "{}", regranted.body);
}
