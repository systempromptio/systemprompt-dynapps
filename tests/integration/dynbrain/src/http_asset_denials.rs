//! What the file routes refuse.
//!
//! Uploading without `write`, an upload id used
//! twice or by another person, an expired upload link, a file over the size
//! limit, a link to a file of an article the person cannot read, a tampered
//! or expired download link, a file in the trash, and a form instead of the
//! raw bytes. SVG and HTML are always downloads, never pages.

use axum::http::{StatusCode, header};
use chrono::{Duration, Utc};
use dynbrain::access::{Level, Subject};
use dynbrain::assets::link::LinkPurpose;

use crate::asset_support::{HTML, PNG, RAW, SVG, attach, file, guide, signed_url, ticket, upload};
use crate::fixture::user;
use crate::http_support::{App, BASE_URL, MAX_UPLOAD, local, signer};

fn upload_url(upload_id: &str, minutes: i64) -> String {
    let link = signer()
        .sign(
            LinkPurpose::Upload,
            upload_id,
            Utc::now() + Duration::minutes(minutes),
        )
        .expect("sign");
    format!(
        "{BASE_URL}/api/dynbrain/v1/uploads/{upload_id}?{}",
        link.query()
    )
}

#[tokio::test]
async fn uploading_needs_write_and_a_stranger_is_told_nothing_exists() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, article) = guide(&app).await;
    app.fixture
        .give(
            space.id,
            Some(article.id),
            Subject::user(&user("bob")),
            Level::Read,
        )
        .await;
    let reader = ticket(&app, article.id, "bob").await;
    assert_eq!(reader.status, StatusCode::FORBIDDEN, "{:?}", reader.body);
    assert_eq!(reader.key(), "forbidden");
    let upload_id = upload(&app, article.id, "alice", PNG).await;
    let adopted = attach(&app, article.id, "bob", &upload_id, "a.png").await;
    assert_eq!(adopted.status, StatusCode::FORBIDDEN);
    let stranger = ticket(&app, article.id, "erin").await;
    assert_eq!(stranger.status, StatusCode::NOT_FOUND);
    let listed = app
        .get(&format!("/articles/{}/assets", article.id), "alice")
        .await;
    assert_eq!(listed.body["total"], 0);
    app.finish().await;
}

#[tokio::test]
async fn an_upload_id_is_adopted_only_once() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (_, article) = guide(&app).await;
    let upload_id = upload(&app, article.id, "alice", PNG).await;
    let first = attach(&app, article.id, "alice", &upload_id, "a.png").await;
    assert_eq!(first.status, StatusCode::CREATED);
    let second = attach(&app, article.id, "alice", &upload_id, "b.png").await;
    assert_eq!(second.status, StatusCode::BAD_REQUEST);
    assert_eq!(second.key(), "invalid");
    let issued = ticket(&app, article.id, "alice").await;
    let url = issued.body["upload_url"].as_str().expect("a URL");
    assert_eq!(
        app.post_bytes(url, RAW, PNG).await.status,
        StatusCode::CREATED
    );
    assert_eq!(
        app.post_bytes(url, RAW, PNG).await.status,
        StatusCode::NOT_FOUND,
        "an upload URL takes one POST"
    );
    let listed = app
        .get(&format!("/articles/{}/assets", article.id), "alice")
        .await;
    assert_eq!(listed.body["total"], 1);
    app.finish().await;
}

#[tokio::test]
async fn the_upload_id_of_another_person_is_refused() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, article) = guide(&app).await;
    app.fixture
        .give(space.id, None, Subject::user(&user("carol")), Level::Write)
        .await;
    let upload_id = upload(&app, article.id, "alice", PNG).await;
    let taken = attach(&app, article.id, "carol", &upload_id, "a.png").await;
    assert_eq!(taken.status, StatusCode::BAD_REQUEST, "{:?}", taken.body);
    assert_eq!(taken.key(), "invalid");
    let own = attach(&app, article.id, "alice", &upload_id, "a.png").await;
    assert_eq!(own.status, StatusCode::CREATED, "the owner still adopts it");
    app.finish().await;
}

#[tokio::test]
async fn an_expired_upload_link_is_refused() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (_, article) = guide(&app).await;
    let issued = ticket(&app, article.id, "alice").await;
    let upload_id = issued.body["upload_id"].as_str().expect("an id").to_owned();
    let late = app.post_bytes(&upload_url(&upload_id, -1), RAW, PNG).await;
    assert_eq!(late.status, StatusCode::NOT_FOUND, "a link past its expiry");
    sqlx::query(
        "UPDATE dynbrain_upload SET expires_at = CURRENT_TIMESTAMP - INTERVAL '1 minute' WHERE id = $1",
    )
    .bind(&upload_id)
    .execute(app.fixture.pool())
    .await
    .expect("expire the slot");
    let stale = app.post_bytes(&upload_url(&upload_id, 5), RAW, PNG).await;
    assert_eq!(
        stale.status,
        StatusCode::NOT_FOUND,
        "a slot past its expiry"
    );
    let adopted = attach(&app, article.id, "alice", &upload_id, "a.png").await;
    assert_eq!(
        adopted.status,
        StatusCode::BAD_REQUEST,
        "nothing was received"
    );
    app.finish().await;
}

#[tokio::test]
async fn a_file_over_the_limit_or_empty_is_refused_and_the_link_survives() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (_, article) = guide(&app).await;
    let issued = ticket(&app, article.id, "alice").await;
    assert_eq!(issued.body["max_bytes"], MAX_UPLOAD);
    let url = issued.body["upload_url"].as_str().expect("a URL");
    let big = vec![b'a'; usize::try_from(MAX_UPLOAD).expect("a size") + 1];
    let refused = app.post_bytes(url, RAW, &big).await;
    assert_eq!(refused.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(refused.json()["error"]["key"], "too_large");
    let empty = app.post_bytes(url, RAW, b"").await;
    assert_eq!(empty.status, StatusCode::BAD_REQUEST);
    let form = app
        .post_bytes(url, "multipart/form-data; boundary=x", PNG)
        .await;
    assert_eq!(form.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        app.post_bytes(url, RAW, PNG).await.status,
        StatusCode::CREATED
    );
    let stored = std::fs::read_dir(app.root.join("dynbrain/uploads")).expect("the store");
    assert_eq!(stored.count(), 1, "the refused bodies wrote nothing");
    app.finish().await;
}

#[tokio::test]
async fn a_link_to_a_file_the_person_cannot_read_is_the_same_not_found() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, article) = guide(&app).await;
    let other = app.fixture.article(space.id, None, "other").await;
    let id = file(&app, article.id, "alice", PNG, "a.png").await;
    let hidden = app.get(&format!("/assets/{id}/link"), "erin").await;
    let missing = app.get("/assets/999999/link", "erin").await;
    assert_eq!(hidden.status, StatusCode::NOT_FOUND);
    assert_eq!(hidden.body, missing.body, "unreadable reads as missing");
    let content = app
        .fetch(&format!("/api/dynbrain/v1/assets/{id}"), Some("erin"))
        .await;
    assert_eq!(content.status, StatusCode::NOT_FOUND);
    let elsewhere = app
        .get(
            &format!("/assets/{id}/link?article_id={}", other.id),
            "alice",
        )
        .await;
    assert_eq!(
        elsewhere.status,
        StatusCode::NOT_FOUND,
        "shown by another article"
    );
    let listed = app
        .get(&format!("/articles/{}/assets", article.id), "erin")
        .await;
    assert_eq!(listed.status, StatusCode::NOT_FOUND);
    app.finish().await;
}

#[tokio::test]
async fn a_tampered_or_expired_download_link_is_refused() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (_, article) = guide(&app).await;
    let id = file(&app, article.id, "alice", PNG, "a.png").await;
    let other = file(&app, article.id, "alice", b"other", "b.txt").await;
    let url = signed_url(&app, id, "alice").await;
    assert_eq!(app.fetch(local(&url), None).await.status, StatusCode::OK);
    let (path, query) = local(&url).split_once('?').expect("a query");
    let (exp, sig) = query.split_once("&sig=").expect("exp and sig");
    let exp: i64 = exp.trim_start_matches("exp=").parse().expect("exp");
    let flipped = format!(
        "{}{}",
        &sig[..sig.len() - 1],
        if sig.ends_with('0') { '1' } else { '0' }
    );
    let expired = signer()
        .sign(
            LinkPurpose::Download,
            &id.to_string(),
            Utc::now() - Duration::minutes(1),
        )
        .expect("sign");
    let upload_purpose = signer()
        .sign(
            LinkPurpose::Upload,
            &id.to_string(),
            Utc::now() + Duration::minutes(5),
        )
        .expect("sign");
    let other_path = path.replace(&format!("/assets/{id}/"), &format!("/assets/{other}/"));
    for refused in [
        format!("{path}?exp={exp}&sig={flipped}"),
        format!("{path}?exp={}&sig={sig}", exp + 60),
        format!("{path}?{}", expired.query()),
        format!("{path}?{}", upload_purpose.query()),
        format!("{other_path}?{query}"),
        path.to_owned(),
    ] {
        let answer = app.fetch(&refused, None).await;
        assert_eq!(answer.status, StatusCode::NOT_FOUND, "{refused}");
    }
    app.finish().await;
}

#[tokio::test]
async fn a_file_in_the_trash_is_not_served() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, article) = guide(&app).await;
    let id = file(&app, article.id, "alice", PNG, "a.png").await;
    let url = signed_url(&app, id, "alice").await;
    let trashed = app
        .post(
            &format!("/articles/{}/trash", article.id),
            "alice",
            &serde_json::json!({}),
        )
        .await;
    assert_eq!(trashed.status, StatusCode::OK);
    assert_eq!(
        app.fetch(local(&url), None).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        app.get(&format!("/assets/{id}/link"), "alice").await.status,
        StatusCode::NOT_FOUND
    );

    let kept = app.fixture.article(space.id, None, "kept").await;
    let alone = file(&app, kept.id, "alice", PNG, "b.png").await;
    let body = format!("![b](asset://{alone})");
    app.fixture.write(&kept, "es", "Kept", &body).await;
    let url = signed_url(&app, alone, "alice").await;
    sqlx::query("UPDATE dynbrain_asset SET trashed_at = CURRENT_TIMESTAMP WHERE id = $1")
        .bind(alone)
        .execute(app.fixture.pool())
        .await
        .expect("trash the file");
    assert_eq!(
        app.fetch(local(&url), None).await.status,
        StatusCode::NOT_FOUND
    );
    let listed = app
        .get(&format!("/articles/{}/assets", kept.id), "alice")
        .await;
    assert_eq!(listed.body["total"], 0);
    let read = app.get(&format!("/articles/{}", kept.id), "alice").await;
    assert_eq!(read.body["asset_urls"], serde_json::json!({}));
    app.finish().await;
}

#[tokio::test]
async fn svg_and_html_are_downloads_never_pages() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (_, article) = guide(&app).await;
    for (bytes, name, mimetype) in [
        (SVG, "diagram.png", "image/svg+xml"),
        (HTML, "page.jpg", "text/html"),
    ] {
        let id = file(&app, article.id, "alice", bytes, name).await;
        let listed = app
            .get(&format!("/articles/{}/assets", article.id), "alice")
            .await;
        let entry = listed.body["files"]
            .as_array()
            .expect("files")
            .iter()
            .find(|file| file["attachment_id"] == id)
            .cloned()
            .expect("the file");
        assert_eq!(entry["mimetype"], mimetype, "the type comes from the bytes");
        assert!(entry["markdown"].as_str().unwrap_or("").starts_with('['));
        let url = signed_url(&app, id, "alice").await;
        for served in [
            app.fetch(local(&url), None).await,
            app.fetch(&format!("/api/dynbrain/v1/assets/{id}"), Some("alice"))
                .await,
        ] {
            assert_eq!(served.status, StatusCode::OK);
            assert_eq!(served.header(header::CONTENT_TYPE), mimetype);
            assert!(
                served
                    .header(header::CONTENT_DISPOSITION)
                    .starts_with("attachment"),
                "{name}"
            );
            assert_eq!(served.header(header::X_CONTENT_TYPE_OPTIONS), "nosniff");
            assert_eq!(served.header(header::CONTENT_SECURITY_POLICY), "sandbox");
        }
    }
    let png = file(&app, article.id, "alice", PNG, "photo.png").await;
    let url = signed_url(&app, png, "alice").await;
    let served = app.fetch(local(&url), None).await;
    assert!(
        served
            .header(header::CONTENT_DISPOSITION)
            .starts_with("inline")
    );
    assert_eq!(served.header(header::X_CONTENT_TYPE_OPTIONS), "nosniff");
    app.finish().await;
}
