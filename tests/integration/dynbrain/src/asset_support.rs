//! Steps the file suites share: a space with one article, a ticket, an
//! upload, an adoption and a signed link, each through the routes.

use axum::http::StatusCode;
use dynbrain::ids::ArticleId;
use dynbrain::model::{Article, Space};
use serde_json::json;

use crate::http_support::{Answer, App};

pub(crate) const PNG: &[u8] =
    b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01";
pub(crate) const SVG: &[u8] =
    b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script>alert(1)</script></svg>";
pub(crate) const HTML: &[u8] =
    b"<!DOCTYPE html><html><body><script>alert(1)</script></body></html>";
pub(crate) const RAW: &str = "application/octet-stream";

// The `handbook` space with one article, `guide`, written in Spanish.
pub(crate) async fn guide(app: &App) -> (Space, Article) {
    let space = app.fixture.space("handbook").await;
    let article = app.fixture.article(space.id, None, "guide").await;
    app.fixture.write(&article, "es", "Guía", "Hola").await;
    (space, article)
}

pub(crate) async fn ticket(app: &App, article: ArticleId, who: &str) -> Answer {
    app.post(
        &format!("/articles/{article}/assets/upload-url"),
        who,
        &json!({"filename": "file"}),
    )
    .await
}

// A ticket of `who` for `article`, with `bytes` POSTed to it: the upload id.
pub(crate) async fn upload(app: &App, article: ArticleId, who: &str, bytes: &[u8]) -> String {
    let issued = ticket(app, article, who).await;
    assert_eq!(issued.status, StatusCode::CREATED, "{:?}", issued.body);
    let url = issued.body["upload_url"].as_str().expect("an upload URL");
    let posted = app.post_bytes(url, RAW, bytes).await;
    assert_eq!(posted.status, StatusCode::CREATED, "{:?}", posted.json());
    posted.json()["upload_id"]
        .as_str()
        .expect("an upload id")
        .to_owned()
}

pub(crate) async fn attach(
    app: &App,
    article: ArticleId,
    who: &str,
    upload: &str,
    filename: &str,
) -> Answer {
    app.post(
        &format!("/articles/{article}/assets"),
        who,
        &json!({"upload_id": upload, "filename": filename}),
    )
    .await
}

// `bytes` uploaded and adopted by `who` as a file of `article`: its id.
pub(crate) async fn file(
    app: &App,
    article: ArticleId,
    who: &str,
    bytes: &[u8],
    filename: &str,
) -> i64 {
    let upload = upload(app, article, who, bytes).await;
    let attached = attach(app, article, who, &upload, filename).await;
    assert_eq!(attached.status, StatusCode::CREATED, "{:?}", attached.body);
    attached.body["attachment_id"]
        .as_i64()
        .expect("an attachment id")
}

pub(crate) async fn signed_url(app: &App, asset: i64, who: &str) -> String {
    let link = app.get(&format!("/assets/{asset}/link"), who).await;
    assert_eq!(link.status, StatusCode::OK, "{:?}", link.body);
    link.body["download_url"]
        .as_str()
        .expect("a download URL")
        .to_owned()
}
