//! The `artifact` capability: a page republishes itself as a new version of
//! its document, as the viewer and only if they may edit it, compare-and-set
//! on the version they loaded. The manifest stays with the document.

use serde_json::{Value, json};
use systemprompt::files::documents::agents::{DocumentActor, act_as};
use systemprompt::files::documents::models::GrantTarget;
use systemprompt::files::documents::service::ShareRequest;
use systemprompt::identifiers::{DocumentId, TraceId, UserId};
use systemprompt::models::auth::{AuthenticatedUser, Permission};
use systemprompt::models::documents::AccessLevel;

use super::calls::{call, tool};
use super::fake::ECHO;
use super::manifest::state;
use super::stack::{As, Reply, SERVER, Stack};

const PAGE: &str = "<!doctype html>\n<html><body><p>published from the page</p></body></html>\n";

async fn publish(
    stack: &Stack,
    who: As<'_>,
    document: &DocumentId,
    html: &str,
    base: &Value,
) -> Reply {
    stack
        .send(
            reqwest::Method::POST,
            &format!("/api/v1/artifacts/{}/artifact/publish", document.as_str()),
            who,
            Some(&json!({ "html": html, "base_revision": base })),
        )
        .await
}

async fn share_view(stack: &Stack, owner: &UserId, page: &DocumentId, viewer: &UserId) {
    let who = AuthenticatedUser::new(
        owner.clone(),
        owner.as_str().to_owned(),
        format!("{}@contract.test", owner.as_str()),
        vec![Permission::User],
    );
    let trace = TraceId::generate();
    let shared = stack.ctx.documents().service().share(
        &who,
        ShareRequest {
            target: GrantTarget::Document(page.clone()),
            subject_kind: "user".to_owned(),
            subject_value: viewer.as_str().to_owned(),
            subject_display: None,
            level: AccessLevel::View,
            valid_until: None,
            message: None,
            via_agent: None,
        },
        &trace,
    );
    Box::pin(act_as(DocumentActor::Human, shared))
        .await
        .expect("the owner shares the page to view");
}

pub(super) struct People<'a> {
    pub(super) owner: &'a UserId,
    pub(super) owner_cookie: &'a str,
    pub(super) reader: &'a UserId,
    pub(super) reader_cookie: &'a str,
}

pub(super) async fn the_page_republishes_itself(
    stack: &Stack,
    people: &People<'_>,
    page: &DocumentId,
    undeclared: &DocumentId,
) {
    let owner = As::Cookie(people.owner_cookie);
    let loaded = state(stack, people.owner_cookie, page).await;
    let base = loaded["revision"].clone();
    assert!(
        base.is_string(),
        "the state names the version the viewer loaded: {loaded}"
    );

    let stale = publish(
        stack,
        owner,
        page,
        PAGE,
        &json!(uuid::Uuid::new_v4().to_string()),
    )
    .await;
    assert_eq!(
        stale.status, 409,
        "a publish over a version someone else replaced: {}",
        stale.body
    );
    assert_eq!(stale.key(), "conflict");
    assert_eq!(
        stale.json()["live"],
        base,
        "the conflict names the live version: {}",
        stale.body
    );
    let fragment = publish(stack, owner, page, "<p>no doctype</p>", &base).await;
    assert_eq!(fragment.status, 400, "{}", fragment.body);
    assert_eq!(fragment.key(), "invalid_content");
    let nobody = publish(stack, As::Nobody, page, PAGE, &base).await;
    assert_eq!(nobody.status, 401, "{}", nobody.body);
    let plain = state(stack, people.owner_cookie, undeclared).await;
    let off = publish(stack, owner, undeclared, PAGE, &plain["revision"]).await;
    assert_eq!(
        off.status, 403,
        "a page that does not declare `artifact`: {}",
        off.body
    );
    assert_eq!(off.key(), "not_declared");

    share_view(stack, people.owner, page, people.reader).await;
    let reader = As::Cookie(people.reader_cookie);
    let read_only = publish(stack, reader, page, PAGE, &base).await;
    assert_eq!(
        read_only.status, 403,
        "a viewer who cannot edit: {}",
        read_only.body
    );
    assert_eq!(read_only.key(), "not_writer");
    let theirs = call(stack, reader, page, &tool(SERVER, ECHO, &json!({}))).await;
    assert_eq!(
        theirs.status, 428,
        "consent is each viewer's own: {}",
        theirs.body
    );

    let published = publish(stack, owner, page, PAGE, &base).await;
    assert_eq!(
        published.status, 200,
        "the owner's page republishes: {}",
        published.body
    );
    let version = published.json()["version"].clone();
    assert!(
        version.is_string() && version != base,
        "a new version: {}",
        published.body
    );
    let after = state(stack, people.owner_cookie, page).await;
    assert_eq!(
        after["revision"], version,
        "the document's head is the published page: {after}"
    );
    assert_eq!(
        after["capabilities"], loaded["capabilities"],
        "the manifest is kept: {after}"
    );
    let replay = publish(stack, owner, page, PAGE, &base).await;
    assert_eq!(
        replay.status, 409,
        "the old base is stale now: {}",
        replay.body
    );
}
