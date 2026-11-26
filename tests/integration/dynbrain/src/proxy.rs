//! Proxy mode against a fake source over real MCP: every operation reaches
//! the tool its mapping names, with the arguments renamed and the constants
//! added, as the person who asks, with that person's token; the source's
//! refusals, an unmapped operation, the grants, and what the REST routes and
//! the `brain` tools answer.

use axum::http::{Method, StatusCode};
use dynbrain::brain::Brain;
use dynbrain::brain::args::{GrantInput, GrantTarget, InheritArgs};
use dynbrain::brain::fault::{BrainFault, SourceFault};
use dynbrain::brain::operation::Operation;
use dynbrain::ids::{ArticleId, SpaceId};
use serde_json::json;

use crate::http_support::desktop_token;
use crate::proxy_support::{Source, call, person, router, user};

#[tokio::test]
async fn every_operation_reaches_the_tool_its_mapping_names() {
    let source = Source::start().await;
    let brain = source.full();
    let alice = person("alice", "jwt-alice");
    for operation in Operation::ALL {
        let reply = brain
            .run(
                &user("alice"),
                &alice,
                operation,
                json!({"article_id": 7, "lang": "es"}),
            )
            .await
            .expect("the source answers");
        let tool = format!("docs_{}", operation.name());
        let reached = source.journal.all().iter().any(|seen| {
            seen.tool == tool
                && seen.arguments["article_id"] == json!(7)
                && seen.arguments["lang"] == json!("es")
        });
        assert!(reached, "{operation:?} reached {tool}");
        assert_eq!(
            reply["tool"],
            json!(tool),
            "the source's answer comes back as it came"
        );
    }
    assert!(source.journal.all().len() >= 18);
}

#[tokio::test]
async fn arguments_are_renamed_and_constants_added() {
    let source = Source::start().await;
    let brain = source.full();
    let alice = person("alice", "jwt-alice");
    brain
        .run(
            &user("alice"),
            &alice,
            Operation::Search,
            json!({"query": "backups", "lang": null, "limit": 5}),
        )
        .await
        .expect("search");
    let seen = source.journal.last();
    assert_eq!(seen.arguments, json!({"q": "backups", "limit": 5}));
    brain
        .run(
            &user("alice"),
            &alice,
            Operation::Answer,
            json!({"question": "how?"}),
        )
        .await
        .expect("answer");
    let seen = source.journal.last();
    assert_eq!(
        seen.arguments,
        json!({"question": "how?", "synthesize": false})
    );
}

#[tokio::test]
async fn the_call_carries_the_token_of_the_person_who_asks() {
    let source = Source::start().await;
    let brain = source.full();
    brain
        .run(
            &user("alice"),
            &person("alice", "jwt-alice"),
            Operation::ListSpaces,
            json!({}),
        )
        .await
        .expect("alice lists");
    let alice = source.journal.last();
    brain
        .run(
            &user("bob"),
            &person("bob", "jwt-bob"),
            Operation::ListSpaces,
            json!({}),
        )
        .await
        .expect("bob lists");
    let bob = source.journal.last();
    assert_eq!(alice.bearer, "Bearer as-jwt-alice");
    assert_eq!(bob.bearer, "Bearer as-jwt-bob");
    assert_ne!(alice.bearer, bob.bearer, "another person, another token");
}

#[tokio::test]
async fn a_401_of_the_source_is_forbidden() {
    let source = Source::start().await;
    let brain = source.full();
    let refused = brain
        .run(
            &user("carol"),
            &person("carol", "jwt-revoked"),
            Operation::Get,
            json!({"article_id": 7}),
        )
        .await;
    assert!(
        matches!(refused, Err(BrainFault::Source(SourceFault::Refused(_)))),
        "{refused:?}"
    );
    assert!(
        source.journal.all().is_empty(),
        "no tool ran for a refused token"
    );
}

#[tokio::test]
async fn a_call_without_the_person_token_never_reaches_the_source() {
    let source = Source::start().await;
    let brain = source.full();
    let context = person("alice", "jwt-alice");
    let bare = systemprompt::models::execution::context::RequestContext::new(
        context.session_id().clone(),
        context.trace_id().clone(),
        context.context_id().clone(),
        context.agent_name().clone(),
        context.actor().clone(),
    );
    let refused = brain
        .run(
            &user("alice"),
            &bare,
            Operation::Search,
            json!({"query": "x"}),
        )
        .await;
    assert!(
        matches!(refused, Err(BrainFault::Source(SourceFault::Refused(_)))),
        "{refused:?}"
    );
    assert!(source.journal.all().is_empty());
}

#[tokio::test]
async fn an_operation_without_a_tool_is_unavailable() {
    let source = Source::start().await;
    let brain = source.brain(&[Operation::Search]);
    let refused = brain
        .run(
            &user("alice"),
            &person("alice", "jwt-alice"),
            Operation::Get,
            json!({"article_id": 7}),
        )
        .await;
    assert!(
        matches!(refused, Err(BrainFault::Unavailable(_))),
        "{refused:?}"
    );
    assert!(source.journal.all().is_empty(), "nothing was called");
    let app = router(brain);
    let answer = call(
        &app,
        Method::GET,
        "/articles/7",
        &desktop_token("alice"),
        None,
    )
    .await;
    assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(answer.body["error"]["key"], "unavailable");
}

#[tokio::test]
async fn grants_are_managed_at_the_source() {
    let source = Source::start().await;
    let brain = source.full();
    let alice = user("alice");
    let grants: Vec<GrantInput> = Vec::new();
    let listed = brain
        .list_grants(&alice, GrantTarget::Space(SpaceId(1)))
        .await;
    assert!(
        matches!(listed, Err(BrainFault::ManagedAtSource)),
        "{listed:?}"
    );
    let set = brain
        .set_grants(&alice, GrantTarget::Article(ArticleId(7)), &grants)
        .await;
    assert!(matches!(set, Err(BrainFault::ManagedAtSource)), "{set:?}");
    let inherit = brain
        .set_inherit(&alice, ArticleId(7), InheritArgs::default())
        .await;
    assert!(
        matches!(inherit, Err(BrainFault::ManagedAtSource)),
        "{inherit:?}"
    );
    let app = router(brain);
    let token = desktop_token("alice");
    let put = call(
        &app,
        Method::PUT,
        "/spaces/1/grants",
        &token,
        Some(&json!({"grants": []})),
    )
    .await;
    assert_eq!(put.status, StatusCode::CONFLICT);
    assert_eq!(put.body["error"]["key"], "managed_at_source");
    let get = call(&app, Method::GET, "/articles/7/grants", &token, None).await;
    assert_eq!(get.status, StatusCode::CONFLICT);
    assert!(
        source.journal.all().is_empty(),
        "grants never reach the source"
    );
}

#[tokio::test]
async fn the_rest_routes_answer_what_the_source_answers() {
    let source = Source::start().await;
    let app = router(source.full());
    let token = desktop_token("alice");
    let spaces = call(&app, Method::GET, "/spaces?limit=5", &token, None).await;
    assert_eq!(spaces.status, StatusCode::OK);
    assert_eq!(
        spaces.body,
        json!({"tool": "docs_list_spaces", "arguments": {"limit": 5}})
    );
    let seen = source.journal.last();
    assert_eq!(
        seen.bearer,
        format!("Bearer as-{token}"),
        "the desktop JWT is exchanged"
    );
    let created = call(
        &app,
        Method::POST,
        "/articles",
        &token,
        Some(&json!({"space": 3, "title": "New"})),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED);
    assert_eq!(created.body["tool"], "docs_create");
    assert_eq!(created.body["arguments"]["title"], "New");
    let moved = call(
        &app,
        Method::POST,
        "/articles/7/move",
        &token,
        Some(&json!({"new_parent_id": 8})),
    )
    .await;
    assert_eq!(
        moved.body["arguments"],
        json!({"article_id": 7, "new_parent_id": 8})
    );
}

#[tokio::test]
async fn the_rest_routes_turn_a_refusal_of_the_source_into_forbidden() {
    let source = Source::start().await;
    let app = router(source.full());
    let refused = call(&app, Method::GET, "/articles/7", "revoked-token", None).await;
    assert_eq!(
        refused.status,
        StatusCode::UNAUTHORIZED,
        "a bad JWT never reaches the source"
    );
    let token = desktop_token("carol");
    let brain = source.full();
    let direct = brain
        .run(
            &user("carol"),
            &person("carol", "revoked"),
            Operation::Trash,
            json!({"article_id": 7}),
        )
        .await
        .expect_err("refused");
    assert_eq!(
        dynbrain::router::error::HttpFault::from(direct).status(),
        StatusCode::FORBIDDEN
    );
    let fine = call(&app, Method::POST, "/articles/7/trash", &token, None).await;
    assert_eq!(fine.status, StatusCode::OK);
    assert_eq!(fine.body["tool"], "docs_trash");
}

#[tokio::test]
async fn the_brain_tools_answer_what_the_source_answers() {
    let source = Source::start().await;
    let brain = source.full();
    let alice = person("alice", "jwt-alice");
    let listed =
        systemprompt_mcp_brain::tools::run(&brain, &alice, "list_spaces", json!({"limit": 2}))
            .await
            .expect("the tool answers");
    assert_eq!(
        listed,
        json!({"tool": "docs_list_spaces", "arguments": {"limit": 2}})
    );
    assert_eq!(source.journal.last().bearer, "Bearer as-jwt-alice");
    let refused = systemprompt_mcp_brain::tools::run(
        &brain,
        &person("carol", "revoked"),
        "get",
        json!({"article_id": 7}),
    )
    .await
    .expect_err("a refused token");
    assert!(refused.message.starts_with("Forbidden"), "{refused:?}");
    let first = systemprompt_mcp_brain::tools::run(
        &brain,
        &alice,
        "asset_upload",
        json!({"article_id": 7, "filename": "a.png"}),
    )
    .await
    .expect("the first upload call");
    assert_eq!(
        first["tool"], "docs_request_upload",
        "the first call asks for the URL"
    );
}

#[tokio::test]
async fn this_instance_takes_no_upload_bytes_and_serves_no_signed_file() {
    let source = Source::start().await;
    let app = router(source.full());
    let upload = axum::http::Request::builder()
        .method(Method::POST)
        .uri(format!(
            "{}/uploads/abc?exp=1&sig=00",
            dynbrain::router::EXTENSION_BASE_PATH
        ))
        .body(axum::body::Body::from(vec![1_u8, 2, 3]))
        .expect("an upload");
    let answer = tower::ServiceExt::oneshot(app.clone(), upload)
        .await
        .expect("answers");
    assert_eq!(answer.status(), StatusCode::NOT_FOUND);
    let raw = call(&app, Method::GET, "/assets/41/raw?exp=1&sig=00", "", None).await;
    assert_eq!(raw.status, StatusCode::NOT_FOUND);
    assert!(source.journal.all().is_empty());
}
