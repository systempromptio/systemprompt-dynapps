//! The broker as a native client uses it: `DynWork` desktop draws the page
//! itself and holds only the person's `ui` session, sent as the bearer with
//! no Origin and no cookie. With it the person reads the manifest (each
//! declared server with the name the viewer is shown for it), reads the page
//! with its runtime and its policy as a header, allows a server, runs a read
//! as themselves through core's proxy and revokes. The broker presents the
//! proxy a web session it mints for the `ui` session, one for as long as it
//! lasts, never one per call. Every other bearer is refused on every route,
//! and a person who cannot see a page gets no page.

use reqwest::header::{CONTENT_SECURITY_POLICY, CONTENT_TYPE};
use serde_json::json;
use systemprompt::identifiers::{DocumentId, UserId};

use super::fake::ECHO;
use super::stack::{As, Reply, SERVER, Stack};

pub(super) struct Who<'a> {
    pub(super) owner: &'a UserId,
    pub(super) owner_cookie: &'a str,
    pub(super) agent: &'a str,
}

// The broker's answer to a request as a native client makes it: the bearer,
// and neither an Origin nor a cookie.
async fn native(
    stack: &Stack,
    method: reqwest::Method,
    path: &str,
    bearer: &str,
    body: Option<&serde_json::Value>,
) -> reqwest::Response {
    let mut request = reqwest::Client::new()
        .request(method, format!("{}{path}", stack.api))
        .bearer_auth(bearer);
    if let Some(body) = body {
        request = request.json(body);
    }
    request.send().await.expect("reach the API")
}

async fn ask(
    stack: &Stack,
    method: reqwest::Method,
    path: &str,
    bearer: &str,
    body: Option<&serde_json::Value>,
) -> Reply {
    let response = native(stack, method, path, bearer, body).await;
    let status = response.status();
    let body = response.text().await.expect("read the API response");
    Reply { status, body }
}

fn header(response: &reqwest::Response, name: &str) -> String {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned()
}

async fn declare(pool: &sqlx::PgPool, page: &DocumentId, owner: &UserId) {
    sqlx::query(
        "INSERT INTO artifact_capabilities (document_id, capabilities, contract_version, declared_by)
         VALUES ($1::uuid, $2, '0.2.99', $3)",
    )
    .bind(page.as_str())
    .bind(json!({ "mcp": { "servers": [{ "server": SERVER, "tools": [ECHO] }] } }))
    .bind(owner.as_str())
    .execute(pool)
    .await
    .expect("declare the fake server's read on the page");
}

async fn web_sessions(pool: &sqlx::PgPool, owner: &UserId) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_sessions WHERE user_id = $1 AND session_source = 'oauth'",
    )
    .bind(owner.as_str())
    .fetch_one(pool)
    .await
    .expect("count the person's web sessions")
}

pub(super) async fn a_native_client_runs_the_page(
    stack: &Stack,
    pool: &sqlx::PgPool,
    who: &Who<'_>,
    remote: &DocumentId,
) -> DocumentId {
    let page =
        Box::pin(stack.named_document(who.owner, "dynapps-html", "artifacts-broker-native")).await;
    declare(pool, &page, who.owner).await;
    let base = format!("/api/v1/artifacts/{}", page.as_str());
    let ui = stack.ui_bearer(pool, who.owner).await;

    let manifest = ask(
        stack,
        reqwest::Method::GET,
        &format!("{base}/manifest"),
        &ui,
        None,
    )
    .await;
    assert_eq!(
        manifest.status, 200,
        "the ui session reads the manifest: {}",
        manifest.body
    );
    assert_eq!(
        manifest.json()["servers"],
        json!([{ "id": SERVER, "name": "Artifacts Fake" }]),
        "a server is named as the live-pages policy names it"
    );
    let unnamed = ask(
        stack,
        reqwest::Method::GET,
        &format!("/api/v1/artifacts/{}/manifest", remote.as_str()),
        &ui,
        None,
    )
    .await;
    assert_eq!(unnamed.status, 200, "{}", unnamed.body);
    assert_eq!(
        unnamed.json()["servers"],
        json!([{ "id": super::stack::REMOTE, "name": super::stack::REMOTE }]),
        "a server with no display name goes by its id"
    );

    let read = ask(
        stack,
        reqwest::Method::GET,
        &format!("{base}/permissions"),
        &ui,
        None,
    )
    .await;
    assert_eq!(read.status, 200, "{}", read.body);
    let revision = read.json()["revision"]
        .as_str()
        .expect("the page's revision")
        .to_owned();
    assert_eq!(
        read.json()["servers"][0]["name"],
        "Artifacts Fake",
        "the shell's state names the server too"
    );

    let allowed = ask(
        stack,
        reqwest::Method::POST,
        &format!("{base}/permissions"),
        &ui,
        Some(&json!({ "scope": super::consent::scope(), "decision": "granted" })),
    )
    .await;
    assert_eq!(
        allowed.status, 200,
        "the ui session allows the server: {}",
        allowed.body
    );
    assert_eq!(allowed.json()["states"][super::consent::scope()], "granted");

    let before = web_sessions(pool, who.owner).await;
    let call = json!({ "server": SERVER, "tool": ECHO, "input": { "text": "native" } });
    for round in ["first", "second"] {
        let ran = ask(
            stack,
            reqwest::Method::POST,
            &format!("{base}/mcp/call"),
            &ui,
            Some(&call),
        )
        .await;
        assert_eq!(ran.status, 200, "the {round} read runs: {}", ran.body);
        assert_eq!(
            ran.json()["structuredContent"]["caller"],
            who.owner.as_str(),
            "the server saw the person: {}",
            ran.body
        );
        assert_eq!(ran.json()["structuredContent"]["text"], "native");
    }
    let listed = ask(
        stack,
        reqwest::Method::POST,
        &format!("{base}/mcp/tools"),
        &ui,
        Some(&json!({})),
    )
    .await;
    assert_eq!(listed.status, 200, "{}", listed.body);
    assert_eq!(
        listed.json()["servers"][0]["authStatus"],
        "connected",
        "{}",
        listed.body
    );
    assert_eq!(
        web_sessions(pool, who.owner).await,
        before + 1,
        "one web session serves every proxied call of the same ui session"
    );

    let revoked = ask(
        stack,
        reqwest::Method::DELETE,
        &format!("{base}/permissions/{}", super::consent::scope()),
        &ui,
        None,
    )
    .await;
    assert_eq!(
        revoked.status, 200,
        "the ui session revokes: {}",
        revoked.body
    );
    let asked = ask(
        stack,
        reqwest::Method::POST,
        &format!("{base}/mcp/call"),
        &ui,
        Some(&call),
    )
    .await;
    assert_eq!(
        asked.status, 428,
        "after revoking, the next call asks again: {}",
        asked.body
    );

    the_page_with_its_runtime(stack, &ui, &page, &revision).await;
    page
}

async fn the_page_with_its_runtime(stack: &Stack, ui: &str, page: &DocumentId, revision: &str) {
    let path = format!("/api/v1/artifacts/{}/page", page.as_str());
    let served = native(stack, reqwest::Method::GET, &path, ui, None).await;
    assert_eq!(served.status(), 200, "the ui session reads the page");
    assert!(
        header(&served, CONTENT_TYPE.as_str()).starts_with("text/html"),
        "the page is HTML"
    );
    assert_eq!(
        header(&served, "x-revision"),
        revision,
        "the page names the revision it is"
    );
    let csp = header(&served, CONTENT_SECURITY_POLICY.as_str());
    for needed in [
        "sandbox allow-scripts;",
        "default-src 'none'",
        "connect-src 'none'",
        "form-action 'none'",
    ] {
        assert!(
            csp.contains(needed),
            "the page's policy has `{needed}`: {csp}"
        );
    }
    assert!(
        !csp.contains("allow-same-origin"),
        "the page never shares an origin: {csp}"
    );
    let nonce = csp
        .split("'nonce-")
        .nth(1)
        .and_then(|rest| rest.split('\'').next())
        .expect("with the instance's scripts off, only the runtime runs, by its nonce")
        .to_owned();
    let body = served.text().await.expect("read the page");
    assert!(
        body.contains(&format!("<script nonce=\"{nonce}\">")),
        "the runtime is inlined with the policy's nonce"
    );
    assert!(
        body.contains("governance-live/1"),
        "the runtime speaks the shell's protocol"
    );
    assert!(
        body.contains("\"origins\":[]"),
        "a native client's runtime trusts its parent window, not this site's origin"
    );

    let pinned = native(
        stack,
        reqwest::Method::GET,
        &format!("{path}?revision={revision}"),
        ui,
        None,
    )
    .await;
    assert_eq!(
        pinned.status(),
        200,
        "the page at the revision the client loaded"
    );
    let bad = native(
        stack,
        reqwest::Method::GET,
        &format!("{path}?revision=nope"),
        ui,
        None,
    )
    .await;
    assert_eq!(bad.status(), 404, "a revision that is not one is not found");
}

pub(super) async fn a_stranger_reads_no_page(
    stack: &Stack,
    pool: &sqlx::PgPool,
    stranger: &UserId,
    page: &DocumentId,
) {
    let ui = stack.ui_bearer(pool, stranger).await;
    let hidden = native(
        stack,
        reqwest::Method::GET,
        &format!("/api/v1/artifacts/{}/page", page.as_str()),
        &ui,
        None,
    )
    .await;
    assert_eq!(
        hidden.status(),
        404,
        "a person who cannot see the page gets no page"
    );
}

pub(super) async fn only_the_person_session_is_taken(
    stack: &Stack,
    pool: &sqlx::PgPool,
    who: &Who<'_>,
    page: &DocumentId,
) {
    let api_key = systemprompt_web_admin_core::repositories::bridge::issue_api_key(
        pool,
        who.owner,
        "native broker contract",
        None,
    )
    .await
    .expect("issue an API key");
    let device = systemprompt_web_admin_core::repositories::bridge::issue_api_key(
        pool,
        who.owner,
        "native broker contract device",
        None,
    )
    .await
    .expect("issue a key");
    sqlx::query("UPDATE user_api_keys SET kind = 'device' WHERE id = $1")
        .bind(&device.id)
        .execute(pool)
        .await
        .expect("make it a Device key");
    let web_token = who
        .owner_cookie
        .strip_prefix("access_token=")
        .expect("a cookie pair");
    let base = format!("/api/v1/artifacts/{}", page.as_str());
    let call = json!({ "server": SERVER, "tool": ECHO, "input": {} });
    let routes = [
        (reqwest::Method::GET, format!("{base}/manifest"), None),
        (reqwest::Method::GET, format!("{base}/page"), None),
        (reqwest::Method::GET, format!("{base}/permissions"), None),
        (
            reqwest::Method::POST,
            format!("{base}/permissions"),
            Some(json!({ "scope": super::consent::scope(), "decision": "granted" })),
        ),
        (
            reqwest::Method::DELETE,
            format!("{base}/permissions/{}", super::consent::scope()),
            None,
        ),
        (
            reqwest::Method::POST,
            format!("{base}/mcp/call"),
            Some(call.clone()),
        ),
        (
            reqwest::Method::POST,
            format!("{base}/mcp/approve"),
            Some(call),
        ),
        (
            reqwest::Method::POST,
            format!("{base}/mcp/tools"),
            Some(json!({})),
        ),
        (
            reqwest::Method::POST,
            format!("{base}/mcp/describe"),
            Some(json!({ "server": SERVER, "tool": ECHO })),
        ),
        (
            reqwest::Method::POST,
            format!("{base}/mcp/invalidate"),
            Some(json!({})),
        ),
        (
            reqwest::Method::POST,
            format!("{base}/artifact/publish"),
            Some(json!({ "html": "<!doctype html>", "base_revision": "x" })),
        ),
    ];
    let refused = [
        ("an agent's bearer", who.agent),
        ("an API key", api_key.secret.as_str()),
        ("a Device key", device.secret.as_str()),
        ("the web session as a bearer", web_token),
    ];
    for (method, path, body) in &routes {
        for (what, bearer) in refused {
            let answer = ask(stack, method.clone(), path, bearer, body.as_ref()).await;
            assert_eq!(
                answer.status, 401,
                "{what} on {method} {path}: {}",
                answer.body
            );
            assert_eq!(answer.key(), "reauth", "{what} on {method} {path}");
            let from_the_site = stack
                .send(method.clone(), path, As::Bearer(bearer), body.as_ref())
                .await;
            assert_eq!(
                from_the_site.status, 401,
                "{what} on {method} {path}, even from the site's own page: {}",
                from_the_site.body
            );
        }
    }
}
