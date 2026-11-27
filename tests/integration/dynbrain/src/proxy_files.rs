//! The files of a proxy instance, against the fake source: `get` names the
//! source's files `asset://<id>` with the source's links in `asset_urls`,
//! `GET assets/{id}` redirects to the source's link (and is a 404 where the
//! source refuses), and the file writes go to the source as the person.

use axum::http::{Method, StatusCode};
use dynbrain::brain::Brain;
use dynbrain::brain::operation::Operation;
use serde_json::json;

use crate::http_support::desktop_token;
use crate::proxy_support::{Source, call, person, router, user};

#[tokio::test]
async fn get_names_the_source_files_as_asset_references() {
    let source = Source::start().await;
    let view = source
        .full()
        .run(
            &user("alice"),
            &person("alice", "jwt-alice"),
            Operation::Get,
            json!({"article_id": 7}),
        )
        .await
        .expect("the document");
    let markdown = view["markdown"].as_str().expect("the markdown");
    assert!(markdown.contains("![diagram](asset://41)"), "{markdown}");
    assert!(markdown.contains("[manual](asset://42)"), "{markdown}");
    assert!(
        markdown.contains("![red](asset://43)"),
        "a listed relative path: {markdown}"
    );
    assert!(markdown.contains("![hidden](asset://99)"), "{markdown}");
    assert!(
        !markdown.contains("source.test"),
        "no source URL is left: {markdown}"
    );
    assert_eq!(
        view["tool"], "docs_get",
        "the rest of the source's answer is kept"
    );
}

#[tokio::test]
async fn get_carries_a_source_link_for_each_file_the_person_may_read() {
    let source = Source::start().await;
    let view = source
        .full()
        .run(
            &user("alice"),
            &person("alice", "jwt-alice"),
            Operation::Get,
            json!({"article_id": 7}),
        )
        .await
        .expect("the document");
    let urls = view["asset_urls"].as_object().expect("asset_urls");
    assert_eq!(
        urls["asset://41"],
        "https://source.test/download/41?token=t"
    );
    assert_eq!(
        urls["asset://43"],
        "https://source.test/download/43?token=t"
    );
    assert!(
        urls.get("asset://99").is_none(),
        "a file the source refuses gets no link"
    );
    let links: Vec<_> = source
        .journal
        .all()
        .into_iter()
        .filter(|seen| seen.tool == "docs_asset_link")
        .collect();
    assert!(!links.is_empty());
    assert!(
        links
            .iter()
            .all(|seen| seen.bearer == "Bearer as-jwt-alice"
                && seen.arguments["article_id"] == json!(7)),
        "each link is asked as the person, for the document"
    );
}

#[tokio::test]
async fn the_asset_route_redirects_to_the_source_link() {
    let source = Source::start().await;
    let app = router(source.full());
    let token = desktop_token("alice");
    let answer = call(&app, Method::GET, "/assets/41", &token, None).await;
    assert_eq!(answer.status, StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(
        answer.location.as_deref(),
        Some("https://source.test/download/41?token=t")
    );
    let seen = source.journal.last();
    assert_eq!(seen.tool, "docs_asset_link");
    assert_eq!(seen.bearer, format!("Bearer as-{token}"));
}

#[tokio::test]
async fn the_asset_route_is_not_found_without_permission_at_the_source() {
    let source = Source::start().await;
    let app = router(source.full());
    let hidden = call(
        &app,
        Method::GET,
        "/assets/99",
        &desktop_token("alice"),
        None,
    )
    .await;
    assert_eq!(hidden.status, StatusCode::NOT_FOUND);
    assert_eq!(hidden.body["error"]["key"], "not_found");
    let refused = source
        .full()
        .asset_redirect(
            &user("carol"),
            &person("carol", "revoked"),
            dynbrain::ids::AssetId(41),
        )
        .await;
    assert!(
        matches!(refused, Err(dynbrain::brain::fault::BrainFault::NotFound)),
        "a refused token is the same not found: {refused:?}"
    );
    let unmapped = router(source.brain(&[Operation::Get]));
    let off = call(
        &unmapped,
        Method::GET,
        "/assets/41",
        &desktop_token("alice"),
        None,
    )
    .await;
    assert_eq!(off.status, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn file_writes_go_to_the_source_as_the_person() {
    let source = Source::start().await;
    let app = router(source.full());
    let token = desktop_token("alice");
    let ticket = call(
        &app,
        Method::POST,
        "/articles/7/assets/upload-url",
        &token,
        Some(&json!({"filename": "a.png"})),
    )
    .await;
    assert_eq!(ticket.status, StatusCode::CREATED);
    assert_eq!(
        ticket.body["upload_url"], "https://source.test/upload/abc",
        "the source's URL"
    );
    let seen = source.journal.last();
    assert_eq!(seen.tool, "docs_request_upload");
    assert_eq!(
        seen.arguments,
        json!({"article_id": 7, "filename": "a.png"})
    );
    assert_eq!(seen.bearer, format!("Bearer as-{token}"));
    let attached = call(
        &app,
        Method::POST,
        "/articles/7/assets",
        &token,
        Some(&json!({"upload_id": "abc", "filename": "a.png"})),
    )
    .await;
    assert_eq!(attached.status, StatusCode::CREATED);
    assert_eq!(source.journal.last().tool, "docs_asset_upload");
    let listed = call(&app, Method::GET, "/articles/7/assets", &token, None).await;
    assert_eq!(listed.body["files"][0]["attachment_id"], 43);
    let link = call(&app, Method::GET, "/assets/42/link", &token, None).await;
    assert_eq!(
        link.body["download_url"],
        "https://source.test/download/42?token=t"
    );
}

#[tokio::test]
async fn a_numeric_upload_id_reaches_the_source_as_a_number() {
    let source = Source::start().await;
    let app = router(source.full());
    let token = desktop_token("alice");
    let attached = call(
        &app,
        Method::POST,
        "/articles/7/assets",
        &token,
        Some(&json!({"upload_id": 886, "filename": "a.png", "path": "ignored.md"})),
    )
    .await;
    assert_eq!(attached.status, StatusCode::CREATED);
    let seen = source.journal.last();
    assert_eq!(seen.tool, "docs_asset_upload");
    assert_eq!(
        seen.arguments,
        json!({"article_id": 7, "upload_id": 886, "filename": "a.png"}),
        "the number is kept and the path in the URL wins"
    );
}

#[tokio::test]
async fn no_byte_of_a_source_file_is_kept_here() {
    let source = Source::start().await;
    let brain = source.full();
    assert_eq!(
        brain.upload_limit(),
        0,
        "a proxy Brain takes no upload bytes"
    );
    let opened = brain
        .open_asset(&user("alice"), dynbrain::ids::AssetId(41))
        .await;
    assert!(opened.is_err(), "it has no file to open");
    let app = router(brain);
    let answer = call(
        &app,
        Method::GET,
        "/assets/41",
        &desktop_token("alice"),
        None,
    )
    .await;
    assert_eq!(
        answer.status,
        StatusCode::TEMPORARY_REDIRECT,
        "the bytes come from the source"
    );
}
