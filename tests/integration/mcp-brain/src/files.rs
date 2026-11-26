use serde_json::json;

use crate::support::Brain;

#[tokio::test]
async fn the_first_call_of_asset_upload_is_the_request_for_the_url() {
    let db = dynapps_test_common::db_or_skip!();
    let brain = Brain::new(db, true).await;
    let space = brain.space("handbook").await;
    let article = brain.article(space.id, None, "leave", ("Vacaciones", "texto")).await;

    let ticket = brain
        .call(
            "alice",
            "asset_upload",
            json!({"article_id": article.id.0, "filename": "plano.png"}),
        )
        .await
        .expect("the first call gives the ticket");
    assert_eq!(ticket["method"], "POST", "{ticket}");
    assert!(ticket["upload_url"].as_str().is_some_and(|url| url.contains("/uploads/")));
    assert!(ticket["upload_id"].is_string());
    let direct = brain
        .call(
            "alice",
            "request_upload",
            json!({"article_id": article.id.0, "filename": "plano.png"}),
        )
        .await
        .expect("request_upload gives one too");
    assert_eq!(direct["method"], "POST");
    brain.finish().await;
}

#[tokio::test]
async fn an_unknown_upload_id_is_refused_and_a_reader_gets_no_url() {
    let db = dynapps_test_common::db_or_skip!();
    let brain = Brain::new(db, true).await;
    let space = brain.space("handbook").await;
    let article = brain.article(space.id, None, "leave", ("Vacaciones", "texto")).await;
    brain.give(space.id, None, "bob", dynbrain::access::Level::Read).await;

    let bogus = brain
        .call(
            "alice",
            "asset_upload",
            json!({"article_id": article.id.0, "filename": "plano.png", "upload_id": "00000000000000000000000000000000"}),
        )
        .await;
    assert!(bogus.is_err(), "{bogus:?}");
    let reader = brain
        .call(
            "bob",
            "request_upload",
            json!({"article_id": article.id.0, "filename": "plano.png"}),
        )
        .await
        .expect_err("a reader cannot upload");
    assert!(reader.message.starts_with("Forbidden"), "{reader:?}");
    let files = brain
        .call("bob", "asset_list", json!({"article_id": article.id.0}))
        .await
        .expect("a reader lists the files");
    assert_eq!(files["total"], 0, "{files}");
    brain.finish().await;
}

#[tokio::test]
async fn without_file_storage_the_file_tools_say_so() {
    let db = dynapps_test_common::db_or_skip!();
    let brain = Brain::new(db, false).await;
    let space = brain.space("handbook").await;
    let article = brain.article(space.id, None, "leave", ("Vacaciones", "texto")).await;

    let refused = brain
        .call(
            "alice",
            "request_upload",
            json!({"article_id": article.id.0, "filename": "plano.png"}),
        )
        .await
        .expect_err("no file storage");
    assert!(refused.message.contains("files are not available"), "{refused:?}");
    brain.finish().await;
}
