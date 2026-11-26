//! The files of articles, end to end: upload, adoption, listing, `get` and
//! `export` with signed URLs, the download, core's file storage underneath,
//! and purging.

use axum::http::{StatusCode, header};
use dynbrain::access::{Level, Subject};
use dynbrain::brain::Brain;
use dynbrain::brain::args::GetArgs;
use dynbrain::brain::assets::RequestUploadArgs;
use dynbrain::brain::fault::BrainFault;
use dynbrain::brain::native::NativeBrain;

use crate::asset_support::{PNG, RAW, attach, file, guide, signed_url, upload};
use crate::fixture::user;
use crate::http_support::{App, BASE_URL, local};

#[tokio::test]
async fn the_full_cycle_uploads_attaches_and_serves_the_same_bytes() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, article) = guide(&app).await;
    let issued = crate::asset_support::ticket(&app, article.id, "alice").await;
    assert_eq!(issued.body["method"], "POST");
    assert_eq!(issued.body["expires_minutes"], 15);
    let url = issued.body["upload_url"].as_str().expect("an upload URL");
    assert!(url.starts_with(&format!("{BASE_URL}/api/dynbrain/v1/uploads/")));
    let posted = app.post_bytes(url, RAW, PNG).await;
    assert_eq!(posted.status, StatusCode::CREATED);
    assert_eq!(posted.json()["mimetype"], "image/png");
    assert_eq!(posted.json()["size_bytes"], PNG.len());
    let upload_id = posted.json()["upload_id"]
        .as_str()
        .expect("an id")
        .to_owned();
    let attached = attach(&app, article.id, "alice", &upload_id, "diagram.png").await;
    assert_eq!(attached.status, StatusCode::CREATED, "{:?}", attached.body);
    assert_eq!(attached.body["attached"], true);
    let id = attached.body["attachment_id"].as_i64().expect("an id");
    let reference = format!("asset://{id}");
    assert_eq!(attached.body["ref"], reference.as_str());
    assert_eq!(
        attached.body["markdown"],
        format!("![diagram.png]({reference})")
    );
    let body = format!("Copias:\n\n![Diagrama de copias]({reference})\n");
    app.fixture.write(&article, "es", "Guía", &body).await;

    let read = app.get(&format!("/articles/{}", article.id), "alice").await;
    assert_eq!(
        read.body["markdown"],
        body.as_str(),
        "the text keeps the reference"
    );
    let signed = read.body["asset_urls"][&reference]
        .as_str()
        .expect("a signed URL for the reference");
    let served = app.fetch(local(signed), None).await;
    assert_eq!(served.status, StatusCode::OK);
    assert_eq!(served.bytes, PNG, "the same bytes come back");
    assert_eq!(served.header(header::CONTENT_TYPE), "image/png");
    assert!(
        served
            .header(header::CONTENT_DISPOSITION)
            .starts_with("inline")
    );
    let own = app
        .fetch(&format!("/api/dynbrain/v1/assets/{id}"), Some("alice"))
        .await;
    assert_eq!(own.status, StatusCode::OK);
    assert_eq!(own.bytes, PNG);
    let listed = app
        .get(&format!("/articles/{}/assets", article.id), "alice")
        .await;
    assert_eq!(listed.body["total"], 1);
    assert_eq!(listed.body["files"][0]["attachment_id"], id);
    assert_eq!(listed.body["files"][0]["name"], "diagram.png");
    let exported = app
        .get(&format!("/spaces/{}/export", space.id), "alice")
        .await;
    assert!(exported.body["asset_urls"][&reference].is_string());
    app.finish().await;
}

#[tokio::test]
async fn the_bytes_are_written_and_read_through_core_file_storage() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (_, article) = guide(&app).await;
    let upload_id = upload(&app, article.id, "alice", PNG).await;
    let kept = app.root.join("dynbrain/uploads").join(&upload_id);
    assert_eq!(std::fs::read(&kept).expect("the stored bytes"), PNG);
    let attached = attach(&app, article.id, "alice", &upload_id, "a.png").await;
    let id = attached.body["attachment_id"].as_i64().expect("an id");
    std::fs::write(&kept, b"replaced in the store").expect("replace the bytes");
    let url = signed_url(&app, id, "alice").await;
    let served = app.fetch(local(&url), None).await;
    assert_eq!(
        served.bytes, b"replaced in the store",
        "a download reads the store, not a copy"
    );
    std::fs::remove_file(&kept).expect("remove the bytes");
    assert_eq!(
        app.fetch(local(&url), None).await.status,
        StatusCode::NOT_FOUND
    );
    app.finish().await;
}

#[tokio::test]
async fn purging_an_article_deletes_the_bytes_of_its_files() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (_, article) = guide(&app).await;
    file(&app, article.id, "alice", PNG, "a.png").await;
    file(&app, article.id, "alice", b"notes", "b.txt").await;
    let uploads = app.root.join("dynbrain/uploads");
    assert_eq!(std::fs::read_dir(&uploads).expect("the store").count(), 2);
    let purged = app.brain.purge_assets(article.id).await.expect("purge");
    assert_eq!(purged, 2);
    assert_eq!(std::fs::read_dir(&uploads).expect("the store").count(), 0);
    let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM dynbrain_asset")
        .fetch_one(app.fixture.pool())
        .await
        .expect("count the files");
    assert_eq!(left, 0);
    app.finish().await;
}

#[tokio::test]
async fn the_files_of_an_article_are_listed_by_language_and_paged() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (_, article) = guide(&app).await;
    let path = format!("/articles/{}/assets", article.id);
    for (lang, name) in [
        (Some("es"), "es.txt"),
        (Some("en"), "en.txt"),
        (None, "all.txt"),
    ] {
        let upload_id = upload(&app, article.id, "alice", name.as_bytes()).await;
        let body = serde_json::json!({"upload_id": upload_id, "filename": name, "lang": lang});
        let made = app.post(&path, "alice", &body).await;
        assert_eq!(made.status, StatusCode::CREATED, "{:?}", made.body);
    }
    let english = app.get(&format!("{path}?lang=en"), "alice").await;
    let names: Vec<&str> = english.body["files"]
        .as_array()
        .expect("files")
        .iter()
        .filter_map(|file| file["name"].as_str())
        .collect();
    assert_eq!(names, ["en.txt", "all.txt"]);
    let second = app.get(&format!("{path}?offset=1&limit=1"), "alice").await;
    assert_eq!(second.body["total"], 3);
    assert_eq!(second.body["shown"], 1);
    assert_eq!(second.body["files"][0]["name"], "en.txt");
    let upload_id = upload(&app, article.id, "alice", b"fr").await;
    let french = serde_json::json!({"upload_id": upload_id, "filename": "fr.txt", "lang": "fr"});
    let refused = app.post(&path, "alice", &french).await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    app.finish().await;
}

#[tokio::test]
async fn get_and_export_sign_only_the_files_the_person_reads() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (space, open) = guide(&app).await;
    let closed = app.fixture.article(space.id, None, "closed").await;
    app.fixture.write(&closed, "es", "Cerrado", "x").await;
    let shown = file(&app, open.id, "alice", PNG, "shown.png").await;
    let hidden = file(&app, closed.id, "alice", PNG, "hidden.png").await;
    let body = format!("![a](asset://{shown}) [b](asset://{hidden}) asset://999999");
    app.fixture.write(&open, "es", "Guía", &body).await;
    app.fixture
        .give(
            space.id,
            Some(open.id),
            Subject::user(&user("erin")),
            Level::Read,
        )
        .await;
    let as_erin = app.get(&format!("/articles/{}", open.id), "erin").await;
    let urls = as_erin.body["asset_urls"].as_object().expect("asset_urls");
    assert_eq!(urls.len(), 1, "{urls:?}");
    assert!(urls.contains_key(&format!("asset://{shown}")));
    assert!(
        !as_erin.body["markdown"]
            .as_str()
            .unwrap_or("")
            .contains("http")
    );
    let exported = app
        .get(&format!("/spaces/{}/export", space.id), "erin")
        .await;
    let exported_urls = exported.body["asset_urls"].as_object().expect("asset_urls");
    assert_eq!(exported_urls.len(), 1);
    let as_alice = app.get(&format!("/articles/{}", open.id), "alice").await;
    assert_eq!(
        as_alice.body["asset_urls"]
            .as_object()
            .map(serde_json::Map::len),
        Some(2)
    );
    app.finish().await;
}

#[tokio::test]
async fn without_a_file_store_the_file_operations_are_unavailable() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let (_, article) = guide(&app).await;
    let bare = NativeBrain::new(app.fixture.repo.clone());
    let alice = user("alice");
    let asked = RequestUploadArgs {
        article_id: Some(article.id),
        ..RequestUploadArgs::default()
    };
    assert!(matches!(
        bare.request_upload(&alice, &asked).await,
        Err(BrainFault::Unavailable(_))
    ));
    let read = bare
        .get(
            &alice,
            &GetArgs {
                article_id: Some(article.id),
                ..GetArgs::default()
            },
        )
        .await
        .expect("get works without files");
    assert!(read.asset_urls.is_empty());
    assert_eq!(bare.upload_limit(), 0);
    app.finish().await;
}
