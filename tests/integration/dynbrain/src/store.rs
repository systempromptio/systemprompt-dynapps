//! Spaces, the article tree, variants and revisions.

use dynbrain::BrainError;
use dynbrain::model::{NewSpace, SpaceEdit};
use dynbrain::properties::{PropertyError, PropertySchema};
use serde_json::json;

use crate::fixture::{Fixture, content, user};

#[tokio::test]
async fn spaces_are_created_read_listed_and_edited() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let handbook = f.space("handbook").await;
    let manuals = f.space("manuals").await;
    assert_eq!(handbook.created_by, Some(user("alice")));
    assert_eq!(handbook.langs, ["es", "en"]);
    assert_eq!(
        f.repo.find_space(handbook.id).await.expect("find"),
        Some(handbook.clone())
    );
    let listed: Vec<String> = f
        .repo
        .list_spaces()
        .await
        .expect("list")
        .into_iter()
        .map(|space| space.slug)
        .collect();
    assert_eq!(listed, ["handbook", "manuals"]);
    let edited = f
        .repo
        .update_space(
            manuals.id,
            &SpaceEdit {
                name: "Manuals".to_owned(),
                properties: PropertySchema::default(),
            },
        )
        .await
        .expect("edit")
        .expect("the space exists");
    assert_eq!(edited.name, "Manuals");
    assert!(edited.properties.0.is_empty());
    f.finish().await;
}

#[tokio::test]
async fn a_space_refuses_languages_it_cannot_hold() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    for (default_lang, langs) in [("fr", vec!["es"]), ("es", vec!["es", "Spanish"])] {
        let refused = f
            .repo
            .insert_space(&NewSpace {
                slug: "wrong".to_owned(),
                name: "Wrong".to_owned(),
                default_lang: default_lang.to_owned(),
                langs: langs.iter().map(|lang| (*lang).to_owned()).collect(),
                properties: PropertySchema::default(),
                created_by: user("alice"),
            })
            .await;
        assert!(matches!(refused, Err(BrainError::Invalid(_))), "{langs:?}");
    }
    assert!(f.repo.list_spaces().await.expect("list").is_empty());
    f.finish().await;
}

#[tokio::test]
async fn articles_form_a_tree_listed_in_order() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let root = f.article(space.id, None, "root").await;
    let first = f.article(space.id, Some(root.id), "first").await;
    let second = f.article(space.id, Some(root.id), "second").await;
    assert_eq!((first.position, second.position), (0, 1));
    assert_eq!(first.parent, Some(root.id));
    let children: Vec<String> = f
        .repo
        .list_children(space.id, Some(root.id))
        .await
        .expect("children")
        .into_iter()
        .map(|article| article.slug)
        .collect();
    assert_eq!(children, ["first", "second"]);
    let top: Vec<String> = f
        .repo
        .list_children(space.id, None)
        .await
        .expect("top level")
        .into_iter()
        .map(|article| article.slug)
        .collect();
    assert_eq!(top, ["root"]);
    assert_eq!(f.repo.list_articles(space.id).await.expect("all").len(), 3);
    assert_eq!(
        f.repo.find_article(second.id).await.expect("find"),
        Some(second)
    );
    f.finish().await;
}

#[tokio::test]
async fn every_variant_write_bumps_the_revision_and_appends_it() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let article = f.article(space.id, None, "policy").await;
    let mut written = content("Policy", "First text");
    written.properties = json!({"status": "draft"})
        .as_object()
        .expect("an object")
        .clone();
    let first = f
        .repo
        .write_variant(article.id, "es", &written, &user("alice"))
        .await
        .expect("first write");
    assert_eq!(first.revision, 1);
    let second = f
        .repo
        .write_variant(
            article.id,
            "es",
            &content("Policy", "Second text"),
            &user("bob"),
        )
        .await
        .expect("second write");
    assert_eq!(second.revision, 2);
    assert_ne!(first.content_hash, second.content_hash);
    assert_eq!(second.updated_by, Some(user("bob")));
    f.repo
        .write_variant(
            article.id,
            "en",
            &content("Policy", "English"),
            &user("alice"),
        )
        .await
        .expect("another language");
    let revisions = f
        .repo
        .list_revisions(article.id, "es")
        .await
        .expect("history");
    let bodies: Vec<(i32, String)> = revisions
        .iter()
        .map(|revision| (revision.number, revision.body.clone()))
        .collect();
    assert_eq!(
        bodies,
        [(1, "First text".to_owned()), (2, "Second text".to_owned())]
    );
    assert_eq!(revisions[0].author, Some(user("alice")));
    assert_eq!(revisions[0].properties.get("status"), Some(&json!("draft")));
    let languages: Vec<String> = f
        .repo
        .list_variants(article.id)
        .await
        .expect("variants")
        .into_iter()
        .map(|variant| variant.lang)
        .collect();
    assert_eq!(languages, ["en", "es"]);
    assert_eq!(
        f.repo
            .find_variant(article.id, "es")
            .await
            .expect("find")
            .map(|variant| variant.body),
        Some("Second text".to_owned())
    );
    f.finish().await;
}

#[tokio::test]
async fn a_refused_variant_write_leaves_nothing_behind() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let article = f.article(space.id, None, "policy").await;
    let foreign = f
        .repo
        .write_variant(
            article.id,
            "fr",
            &content("Politique", "Texte"),
            &user("alice"),
        )
        .await;
    assert!(matches!(
        foreign,
        Err(BrainError::LanguageNotInSpace { .. })
    ));
    let mut wrong = content("Policy", "Text");
    wrong.properties = json!({"status": "archived"})
        .as_object()
        .expect("an object")
        .clone();
    let refused = f
        .repo
        .write_variant(article.id, "es", &wrong, &user("alice"))
        .await;
    assert!(matches!(
        refused,
        Err(BrainError::Property(PropertyError::WrongType { .. }))
    ));
    assert!(
        f.repo
            .list_variants(article.id)
            .await
            .expect("variants")
            .is_empty()
    );
    assert!(
        f.repo
            .list_revisions(article.id, "es")
            .await
            .expect("history")
            .is_empty()
    );
    f.finish().await;
}

#[tokio::test]
async fn revisions_refuse_updates() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let article = f.article(space.id, None, "policy").await;
    f.repo
        .write_variant(article.id, "es", &content("Policy", "Text"), &user("alice"))
        .await
        .expect("write");
    let rewritten = sqlx::query("UPDATE dynbrain_article_revision SET body = 'forged'")
        .execute(f.pool())
        .await;
    assert!(rewritten.is_err(), "the history is append-only");
    f.finish().await;
}

#[tokio::test]
async fn trash_takes_the_subtree_out_and_restore_brings_it_back() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let root = f.article(space.id, None, "root").await;
    let child = f.article(space.id, Some(root.id), "child").await;
    let trashed = f.repo.trash_article(root.id).await.expect("trash");
    assert!(trashed.trashed_at.is_some());
    let child_now = f
        .repo
        .find_article(child.id)
        .await
        .expect("find")
        .expect("kept");
    assert_eq!(child_now.trashed_at, trashed.trashed_at);
    assert!(
        f.repo
            .list_children(space.id, None)
            .await
            .expect("top")
            .is_empty()
    );
    assert!(matches!(
        f.repo
            .write_variant(child.id, "es", &content("Child", "Text"), &user("alice"))
            .await,
        Err(BrainError::ArticleNotFound(_))
    ));
    let restored = f.repo.restore_article(root.id).await.expect("restore");
    assert!(restored.trashed_at.is_none());
    let child_back = f
        .repo
        .find_article(child.id)
        .await
        .expect("find")
        .expect("kept");
    assert!(child_back.trashed_at.is_none());
    f.finish().await;
}
