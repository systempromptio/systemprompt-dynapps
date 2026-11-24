//! The session handoff: `DynWork` desktop holds the person's `ui` session,
//! which the broker never takes, and opens the viewer shell in a view of its
//! own. It asks `POST /api/v1/drive/handoff` for a one-shot code bound to the
//! person and a `/drive/` path, and the view loads the answered URL: the
//! redeem mints a real web session carrying the `ui` session's own sign-in
//! time, sets the web login's cookie and sends the view on to the path. The
//! cookie is then what the broker and the shell take. A code works once,
//! for 60 s; only the `ui` session issues one; every failed redeem is the
//! same trip to the web login, which comes back to the path.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use reqwest::header::{LOCATION, SET_COOKIE};
use serde_json::{Value, json};
use systemprompt::identifiers::{DocumentId, UserId};

use super::stack::{As, Stack};

const ISSUE: &str = "/api/v1/drive/handoff";

pub(super) struct Who<'a> {
    pub(super) owner: &'a UserId,
    pub(super) owner_cookie: &'a str,
    pub(super) agent: &'a str,
}

// The minted web cookie, `access_token=<jwt>`, from a redeem's answer.
fn session_cookie(response: &reqwest::Response) -> Option<String> {
    response
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find(|value| value.starts_with("access_token=") && !value.starts_with("access_token=;"))
        .map(str::to_owned)
}

fn location(response: &reqwest::Response) -> String {
    response
        .headers()
        .get(LOCATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned()
}

fn claims(cookie: &str) -> Value {
    let token = cookie
        .split(';')
        .next()
        .and_then(|pair| pair.strip_prefix("access_token="))
        .expect("the cookie carries the token");
    let payload = token.split('.').nth(1).expect("a JWT payload");
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(payload).expect("base64url payload"))
        .expect("JSON claims")
}

async fn issue(stack: &Stack, who: As<'_>, path: &str) -> super::stack::Reply {
    stack
        .send(
            reqwest::Method::POST,
            ISSUE,
            who,
            Some(&json!({ "path": path })),
        )
        .await
}

// The answered URL on this run's listener, as the view loads it.
fn redeem_path(stack: &Stack, url: &str) -> String {
    let site = stack.ctx.config().api_external_url.trim_end_matches('/');
    let path = url
        .strip_prefix(site)
        .unwrap_or_else(|| panic!("the handoff URL {url} is on this site ({site})"));
    assert!(
        path.starts_with("/drive/handoff?code="),
        "the URL redeems at /drive/handoff: {url}"
    );
    path.to_owned()
}

async fn issued(stack: &Stack, ui: &str, path: &str) -> String {
    let reply = issue(stack, As::Bearer(ui), path).await;
    assert_eq!(reply.status, 200, "a `ui` session issues: {}", reply.body);
    let url = reply.json()["url"]
        .as_str()
        .expect("the answer carries `url`")
        .to_owned();
    redeem_path(stack, &url)
}

async fn redeem(stack: &Stack, path: &str) -> reqwest::Response {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("build a client that does not follow redirects")
        .get(format!("{}{path}", stack.api))
        .header("accept", "text/html")
        .header("sec-fetch-mode", "navigate")
        .header("sec-fetch-site", "none")
        .send()
        .await
        .expect("reach the redeem")
}

fn encoded(path: &str) -> String {
    url::form_urlencoded::byte_serialize(path.as_bytes()).collect()
}

fn sign_in_again(target: &str) -> String {
    format!("/admin/login?redirect={}", encoded(target))
}

pub(super) async fn the_ui_session_opens_a_web_session(
    stack: &Stack,
    pool: &sqlx::PgPool,
    who: &Who<'_>,
    page: &DocumentId,
) {
    let target = format!("/drive/{}/live?embed=1", page.as_str());
    let signed_in = chrono::Utc::now().timestamp() - 7200;
    let ui = stack
        .ui_bearer_signed_in_at(pool, who.owner, signed_in)
        .await;

    let code = issued(stack, &ui, &target).await;
    let opened = redeem(stack, &code).await;
    assert_eq!(
        opened.status(),
        303,
        "a redeem sends the view on: {}",
        location(&opened)
    );
    assert_eq!(
        location(&opened),
        target,
        "to the path the code is bound to"
    );
    let cookie = session_cookie(&opened).expect("the redeem sets the web login's cookie");
    for attribute in ["; Path=/", "; HttpOnly", "; SameSite=Lax", "; Max-Age="] {
        assert!(
            cookie.contains(attribute),
            "the cookie is the web login's ({attribute}): {cookie}"
        );
    }
    let minted = claims(&cookie);
    assert_eq!(
        minted["sub"],
        who.owner.as_str(),
        "the session is the person's"
    );
    assert_eq!(
        minted["browser_session"], true,
        "a web session, as the web login mints"
    );
    assert_eq!(
        minted["auth_time"], signed_in,
        "the sign-in time is the `ui` session's own, never the redeem's"
    );
    let web = cookie
        .split(';')
        .next()
        .expect("the cookie pair")
        .to_owned();

    let tools = stack
        .send(
            reqwest::Method::POST,
            &format!("/api/v1/artifacts/{}/mcp/tools", page.as_str()),
            As::Cookie(&web),
            Some(&json!({})),
        )
        .await;
    assert_eq!(
        tools.status, 200,
        "the broker takes the handed-off session: {}",
        tools.body
    );
    assert!(
        tools.json()["servers"].is_array(),
        "the page's servers: {}",
        tools.body
    );
    let shell = stack.navigate(&target, Some(&web)).await;
    assert_eq!(shell.status(), 200, "the shell opens with it");

    let again = redeem(stack, &code).await;
    assert_eq!(again.status(), 303, "a spent code is a trip to sign in");
    assert_eq!(location(&again), sign_in_again(&target));
    assert!(
        session_cookie(&again).is_none(),
        "a spent code mints nothing"
    );

    let late = issued(stack, &ui, &target).await;
    sqlx::query("UPDATE drive_handoffs SET expires_at = NOW() - INTERVAL '1 second'")
        .execute(pool)
        .await
        .expect("let the code expire");
    let expired = redeem(stack, &late).await;
    assert_eq!(
        location(&expired),
        sign_in_again(&target),
        "an expired code fails"
    );
    assert!(
        session_cookie(&expired).is_none(),
        "an expired code mints nothing"
    );
    let left = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM drive_handoffs")
        .fetch_one(pool)
        .await
        .expect("count the codes");
    assert_eq!(left, 0, "a redeemed or expired code is not kept");

    let unknown = redeem(
        stack,
        &format!("/drive/handoff?code=not-a-code&path={}", encoded(&target)),
    )
    .await;
    assert_eq!(
        location(&unknown),
        sign_in_again(&target),
        "an unknown code fails the same way"
    );

    let logins = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_activity
          WHERE user_id = $1 AND category = 'login' AND metadata->>'via' = 'dynwork_handoff'",
    )
    .bind(who.owner.as_str())
    .fetch_one(pool)
    .await
    .expect("count the handoff logins");
    assert_eq!(logins, 1, "one login recorded, marked as a DynWork handoff");
}

pub(super) async fn only_a_ui_session_issues(
    stack: &Stack,
    pool: &sqlx::PgPool,
    who: &Who<'_>,
    page: &DocumentId,
) {
    let target = format!("/drive/{}/live?embed=1", page.as_str());
    let ui = stack.ui_bearer(pool, who.owner).await;
    for foreign in [
        "https://evil.example/drive/x",
        "//evil.example/drive/x",
        "/admin",
        "/drive",
        "/drive/../admin",
        "/drive/%2e%2e/admin",
        "/drive/x\\..\\admin",
        "drive/x",
        "",
    ] {
        let refused = issue(stack, As::Bearer(&ui), foreign).await;
        assert_eq!(
            refused.status, 400,
            "`{foreign}` is not a Drive path: {}",
            refused.body
        );
    }

    let web_token = who
        .owner_cookie
        .strip_prefix("access_token=")
        .expect("a cookie pair");
    let api_key = systemprompt_web_admin_core::repositories::bridge::issue_api_key(
        pool,
        who.owner,
        "handoff contract",
        None,
    )
    .await
    .expect("issue an API key");
    let device = systemprompt_web_admin_core::repositories::bridge::issue_api_key(
        pool,
        who.owner,
        "handoff contract device",
        None,
    )
    .await
    .expect("issue a key");
    sqlx::query("UPDATE user_api_keys SET kind = 'device' WHERE id = $1")
        .bind(&device.id)
        .execute(pool)
        .await
        .expect("make it a Device key");
    let refused = [
        ("no credential", As::Nobody),
        ("the web cookie", As::Cookie(who.owner_cookie)),
        ("the web session as a bearer", As::Bearer(web_token)),
        ("an agent's bearer", As::Bearer(who.agent)),
        ("an API key", As::Bearer(&api_key.secret)),
        ("a Device key", As::Bearer(&device.secret)),
    ];
    for (what, credential) in refused {
        let reply = issue(stack, credential, &target).await;
        assert_eq!(reply.status, 401, "{what} cannot issue: {}", reply.body);
    }

    let revoked = stack.ui_bearer(pool, who.owner).await;
    let session = claims(&format!("access_token={revoked}"))["session_id"]
        .as_str()
        .expect("a `ui` session names its session")
        .to_owned();
    sqlx::query("UPDATE user_sessions SET revoked_at = NOW() WHERE session_id = $1")
        .bind(&session)
        .execute(pool)
        .await
        .expect("revoke the session");
    let reply = issue(stack, As::Bearer(&revoked), &target).await;
    assert_eq!(
        reply.status, 401,
        "a revoked `ui` session cannot issue: {}",
        reply.body
    );
}
