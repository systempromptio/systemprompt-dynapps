//! The viewer's consent per page and scope (`mcp:<server>`): read, granted,
//! denied and revoked only with the web login's own session. An agent's
//! bearer, an agent's token carried as the cookie, nobody, and a person who
//! cannot see the page are all refused.

use serde_json::{Value, json};
use systemprompt::identifiers::DocumentId;

use super::stack::{As, SERVER, Stack};

pub(super) fn scope() -> String {
    format!("mcp:{SERVER}")
}

fn path(document: &DocumentId) -> String {
    format!("/api/v1/artifacts/{}/permissions", document.as_str())
}

pub(super) async fn decide(
    stack: &Stack,
    who: As<'_>,
    document: &DocumentId,
    decision: &str,
) -> super::stack::Reply {
    stack
        .send(
            reqwest::Method::POST,
            &path(document),
            who,
            Some(&json!({ "scope": scope(), "decision": decision })),
        )
        .await
}

pub(super) async fn revoke(stack: &Stack, cookie: &str, document: &DocumentId) -> Value {
    let reply = stack
        .send(
            reqwest::Method::DELETE,
            &format!("{}/{}", path(document), scope()),
            As::Cookie(cookie),
            None,
        )
        .await;
    assert_eq!(reply.status, 200, "the viewer revokes: {}", reply.body);
    reply.json()
}

pub(super) struct Who<'a> {
    pub(super) owner: &'a str,
    pub(super) outsider: &'a str,
    pub(super) agent: &'a str,
}

pub(super) async fn only_the_viewer_decides(stack: &Stack, who: &Who<'_>, page: &DocumentId) {
    let before = super::manifest::state(stack, who.owner, page).await;
    assert_eq!(
        before["states"][scope()],
        "prompt",
        "nothing decided yet: {before}"
    );
    assert_eq!(before["states"]["mcp"], "prompt", "{before}");
    assert_eq!(before["states"]["artifact"], "granted", "{before}");

    let agent = decide(stack, As::Bearer(who.agent), page, "granted").await;
    assert_eq!(
        agent.status, 401,
        "an agent's bearer never grants: {}",
        agent.body
    );
    assert_eq!(agent.key(), "reauth");
    let agent_cookie = format!("access_token={}", who.agent);
    let smuggled = decide(stack, As::Cookie(&agent_cookie), page, "granted").await;
    assert_eq!(
        smuggled.status, 401,
        "an agent's token as the cookie: {}",
        smuggled.body
    );
    let read = stack
        .send(
            reqwest::Method::GET,
            &path(page),
            As::Bearer(who.agent),
            None,
        )
        .await;
    assert_eq!(
        read.status, 401,
        "an agent's bearer never reads the state: {}",
        read.body
    );
    let nobody = decide(stack, As::Nobody, page, "granted").await;
    assert_eq!(nobody.status, 401, "{}", nobody.body);
    let outsider = decide(stack, As::Cookie(who.outsider), page, "granted").await;
    assert_eq!(
        outsider.status, 404,
        "a person who cannot see the page: {}",
        outsider.body
    );
    assert_eq!(outsider.key(), "not_found");
    let undeclared = stack
        .send(
            reqwest::Method::POST,
            &path(page),
            As::Cookie(who.owner),
            Some(&json!({ "scope": "mcp:documents", "decision": "granted" })),
        )
        .await;
    assert_eq!(
        undeclared.status, 400,
        "a scope the page does not declare: {}",
        undeclared.body
    );

    let unchanged = super::manifest::state(stack, who.owner, page).await;
    assert_eq!(
        unchanged["states"][scope()],
        "prompt",
        "refused requests decided nothing"
    );
    let granted = decide(stack, As::Cookie(who.owner), page, "granted").await;
    assert_eq!(granted.status, 200, "the viewer grants: {}", granted.body);
    assert_eq!(
        granted.json()["states"][scope()],
        "granted",
        "{}",
        granted.body
    );
    assert_eq!(
        granted.json()["states"]["mcp"],
        "granted",
        "{}",
        granted.body
    );
}
