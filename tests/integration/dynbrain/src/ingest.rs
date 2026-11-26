//! The indexer against a throwaway database, with the models replaced by
//! deterministic doubles: the queue, eligibility, chunks, synopses,
//! embeddings and their fingerprint.

use dynbrain::ids::ArticleId;
use dynbrain::ingest::provider::Failure;
use dynbrain::model::{NewArticle, RagMode};

use crate::fixture::{Fixture, user};
use crate::ingest_support::{
    FakeEmbedder, FakeSummarizer, count, include_everything, indexer, plain_body, run,
};

const JOBS: &str = "SELECT COUNT(*) FROM dynbrain_index_job";
const CHUNKS: &str = "SELECT COUNT(*) FROM dynbrain_chunk";
const EMBEDDED: &str = "SELECT COUNT(*) FROM dynbrain_chunk WHERE embedding IS NOT NULL";

async fn article_with(f: &Fixture, slug: &str, mode: RagMode, body: &str) -> ArticleId {
    let space = match f.repo.list_spaces().await.expect("list").into_iter().next() {
        Some(space) => space,
        None => f.space("handbook").await,
    };
    let article = f
        .repo
        .insert_article(
            &NewArticle {
                space: space.id,
                parent: None,
                position: None,
                slug: slug.to_owned(),
                icon: None,
                rag_mode: mode,
            },
            &user("alice"),
        )
        .await
        .expect("create an article");
    f.write(&article, "es", "Zorros", body).await;
    article.id
}

#[tokio::test]
async fn a_write_queues_its_content_once_and_each_new_content_again() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let id = article_with(&f, "guide", RagMode::Default, "Hola.").await;
    assert_eq!(count(&f, JOBS).await, 1);
    f.write(&article_of(&f, id).await, "es", "Zorros", "Hola.")
        .await;
    assert_eq!(count(&f, JOBS).await, 1, "the same content queues nothing");
    f.write(&article_of(&f, id).await, "es", "Zorros", "Adios.")
        .await;
    assert_eq!(count(&f, JOBS).await, 2);
    assert_eq!(
        count(
            &f,
            "SELECT COUNT(*) FROM dynbrain_index_job WHERE state = 'pending'"
        )
        .await,
        2
    );
    f.finish().await;
}

async fn article_of(f: &Fixture, id: ArticleId) -> dynbrain::model::Article {
    f.repo
        .find_article(id)
        .await
        .expect("find")
        .expect("the article")
}

#[tokio::test]
async fn nothing_is_indexed_unless_a_rule_or_the_article_says_so() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    article_with(&f, "guide", RagMode::Default, &plain_body(5)).await;
    let (embedder, summarizer) = (FakeEmbedder::named("test:model"), FakeSummarizer::shared());
    let report = run(&indexer(&f.repo, &embedder, &summarizer)).await;
    assert_eq!((report.indexed, report.purged), (0, 1));
    assert_eq!(count(&f, CHUNKS).await, 0);
    assert_eq!(embedder.requests(), 0);
    assert_eq!(
        count(
            &f,
            "SELECT COUNT(*) FROM dynbrain_index_job WHERE state = 'done'"
        )
        .await,
        1
    );
    f.finish().await;
}

#[tokio::test]
async fn the_mode_of_an_article_beats_the_rules_and_a_rule_decides_the_rest() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    include_everything(&f).await;
    let never = article_with(&f, "never", RagMode::Never, &plain_body(5)).await;
    let normal = article_with(&f, "normal", RagMode::Default, &plain_body(5)).await;
    let (embedder, summarizer) = (FakeEmbedder::named("test:model"), FakeSummarizer::shared());
    run(&indexer(&f.repo, &embedder, &summarizer)).await;
    let of = |id: ArticleId| format!("SELECT COUNT(*) FROM dynbrain_chunk WHERE article_id = {id}");
    assert_eq!(count(&f, &of(never)).await, 0);
    assert_eq!(count(&f, &of(normal)).await, 1);
    f.finish().await;
}

#[tokio::test]
async fn an_included_variant_leaves_chunks_with_a_text_search_vector_and_an_embedding() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    include_everything(&f).await;
    let body = "# Cacería\n\nLos zorros cazan de noche y duermen de día en la madriguera.";
    article_with(&f, "guide", RagMode::Default, body).await;
    let (embedder, summarizer) = (FakeEmbedder::named("test:model"), FakeSummarizer::shared());
    let report = run(&indexer(&f.repo, &embedder, &summarizer)).await;
    assert_eq!((report.indexed, report.embedded, report.failed), (1, 1, 0));
    assert_eq!(count(&f, CHUNKS).await, 1);
    assert_eq!(
        count(
            &f,
            "SELECT COUNT(*) FROM dynbrain_chunk WHERE search_vector IS NOT NULL"
        )
        .await,
        1
    );
    assert_eq!(
        count(&f, "SELECT COUNT(*) FROM dynbrain_chunk WHERE search_vector @@ plainto_tsquery('spanish', 'zorro')").await,
        1,
        "stemmed in the language of the variant"
    );
    assert_eq!(count(&f, EMBEDDED).await, 1);
    let kept: (String, String, String) = sqlx::query_as(
        "SELECT heading_path, embedding_fingerprint, fts_config::text FROM dynbrain_chunk",
    )
    .fetch_one(f.pool())
    .await
    .expect("the chunk");
    assert_eq!(
        kept,
        (
            "Cacería".to_owned(),
            "test:model:1536".to_owned(),
            "spanish".to_owned()
        )
    );
    let sent = embedder.seen();
    assert_eq!(sent.len(), 1);
    assert!(
        sent[0].starts_with("Zorros\nCacería\n# Cacería"),
        "{}",
        sent[0]
    );
    let stored: String = sqlx::query_scalar("SELECT content FROM dynbrain_chunk")
        .fetch_one(f.pool())
        .await
        .expect("content");
    assert!(
        !stored.starts_with("Zorros"),
        "the prefix goes only into the embedding"
    );
    assert!(
        summarizer.prompts().is_empty(),
        "a chunk with a heading needs no synopsis"
    );
    f.finish().await;
}

#[tokio::test]
async fn the_same_content_is_not_indexed_or_embedded_twice() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    include_everything(&f).await;
    let id = article_with(&f, "guide", RagMode::Default, "# A\n\nUna frase de prueba.").await;
    let (embedder, summarizer) = (FakeEmbedder::named("test:model"), FakeSummarizer::shared());
    let worker = indexer(&f.repo, &embedder, &summarizer);
    run(&worker).await;
    let after_first = embedder.requests();
    let again = run(&worker).await;
    assert_eq!((again.indexed, again.embedded), (0, 0));
    assert_eq!(embedder.requests(), after_first);
    let article = article_of(&f, id).await;
    f.write(&article, "es", "Zorros", "# B\n\nOtra frase.")
        .await;
    f.write(&article, "es", "Zorros", "# A\n\nUna frase de prueba.")
        .await;
    let back = run(&worker).await;
    assert_eq!(
        back.indexed, 0,
        "the chunks already match the content written last"
    );
    assert_eq!(embedder.requests(), after_first);
    assert_eq!(
        count(
            &f,
            "SELECT COUNT(*) FROM dynbrain_index_job WHERE state = 'done'"
        )
        .await,
        1
    );
    f.finish().await;
}

#[tokio::test]
async fn a_changed_text_rebuilds_the_chunks_and_unchanged_ones_keep_their_synopsis() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    include_everything(&f).await;
    let id = article_with(&f, "guide", RagMode::Default, &plain_body(70)).await;
    let (embedder, summarizer) = (FakeEmbedder::named("test:model"), FakeSummarizer::shared());
    let worker = indexer(&f.repo, &embedder, &summarizer);
    let first = run(&worker).await;
    assert_eq!((first.synopses, first.embedded), (3, 3));
    assert_eq!(summarizer.prompts().len(), 3);
    let article = article_of(&f, id).await;
    f.write(
        &article,
        "es",
        "Zorros",
        &format!("{} Final{}.", plain_body(70), 1),
    )
    .await;
    let second = run(&worker).await;
    assert_eq!(second.indexed, 1);
    assert_eq!(
        summarizer.prompts().len(),
        4,
        "only the changed chunk pays for a synopsis"
    );
    assert_eq!(count(&f, "SELECT COUNT(*) FROM dynbrain_chunk WHERE synopsis IS NOT NULL AND synopsis_hash = content_hash").await, 3);
    assert_eq!(count(&f, EMBEDDED).await, 3);
    f.finish().await;
}

#[tokio::test]
async fn no_file_reference_reaches_a_chunk_a_synopsis_or_an_embedding() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    include_everything(&f).await;
    let body =
        "Intro ![Architecture diagram](asset://3) and [the spec](asset://4). See asset://5 too.";
    article_with(&f, "guide", RagMode::Default, body).await;
    let (embedder, summarizer) = (FakeEmbedder::named("test:model"), FakeSummarizer::shared());
    run(&indexer(&f.repo, &embedder, &summarizer)).await;
    let stored: String = sqlx::query_scalar("SELECT content FROM dynbrain_chunk")
        .fetch_one(f.pool())
        .await
        .expect("content");
    assert!(
        stored.contains("Architecture diagram") && stored.contains("the spec"),
        "{stored}"
    );
    assert!(!stored.contains("asset://"));
    assert!(
        embedder
            .seen()
            .iter()
            .all(|text| !text.contains("asset://"))
    );
    let prompts = summarizer.prompts();
    assert_eq!(prompts.len(), 1);
    assert!(
        prompts
            .iter()
            .all(|p| !p.system.contains("asset://") && !p.user.contains("asset://"))
    );
    assert!(prompts[0].user.contains("Architecture diagram"));
    f.finish().await;
}

#[tokio::test]
async fn without_a_key_the_chunks_wait_with_their_text_search_and_embed_later() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    include_everything(&f).await;
    article_with(&f, "guide", RagMode::Default, "# A\n\nLos zorros duermen.").await;
    let (embedder, summarizer) = (FakeEmbedder::named("test:model"), FakeSummarizer::shared());
    let worker = indexer(&f.repo, &embedder, &summarizer);
    embedder.fail_with(Some(Failure::NotConfigured("no key".to_owned())));
    let waiting = run(&worker).await;
    assert_eq!(
        waiting.stopped,
        Some(Failure::NotConfigured("no key".to_owned()))
    );
    assert_eq!((count(&f, CHUNKS).await, count(&f, EMBEDDED).await), (1, 0));
    assert_eq!(count(&f, "SELECT COUNT(*) FROM dynbrain_chunk WHERE search_vector IS NOT NULL AND has_error = false").await, 1);
    embedder.fail_with(None);
    let later = run(&worker).await;
    assert_eq!(later.embedded, 1);
    assert_eq!(count(&f, EMBEDDED).await, 1);
    f.finish().await;
}

#[tokio::test]
async fn a_different_fingerprint_is_detected_and_the_cost_of_each_batch_is_recorded() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    include_everything(&f).await;
    article_with(&f, "guide", RagMode::Default, "# A\n\nUna frase.").await;
    let summarizer = FakeSummarizer::shared();
    let old = FakeEmbedder::named("old:model");
    let report = run(&indexer(&f.repo, &old, &summarizer)).await;
    assert_eq!(
        (report.other_fingerprints, report.cost_microdollars),
        (0, 3)
    );
    let new = FakeEmbedder::named("new:model");
    let changed = run(&indexer(&f.repo, &new, &summarizer)).await;
    assert_eq!(changed.other_fingerprints, 1);
    let counts = f.repo.fingerprint_counts().await.expect("counts");
    assert_eq!(counts.len(), 1);
    assert_eq!(counts[0].fingerprint, "old:model:1536");
    let spent: (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*), COALESCE(SUM(cost_microdollars), 0)::int8 FROM dynbrain_embedding_usage",
    )
    .fetch_one(f.pool())
    .await
    .expect("usage");
    assert_eq!(spent, (1, 3));
    f.finish().await;
}

#[tokio::test]
async fn a_trashed_article_keeps_its_chunks_and_is_not_indexed_until_it_returns() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    include_everything(&f).await;
    let kept = article_with(&f, "kept", RagMode::Default, "# A\n\nUna frase.").await;
    let (embedder, summarizer) = (FakeEmbedder::named("test:model"), FakeSummarizer::shared());
    let worker = indexer(&f.repo, &embedder, &summarizer);
    run(&worker).await;
    f.repo.trash_article(kept).await.expect("trash");
    assert_eq!(count(&f, CHUNKS).await, 1, "the trash keeps the chunks");
    let waiting = article_with(&f, "waiting", RagMode::Default, "# B\n\nOtra frase.").await;
    f.repo.trash_article(waiting).await.expect("trash");
    run(&worker).await;
    assert_eq!(count(&f, CHUNKS).await, 1);
    f.repo.restore_article(waiting).await.expect("restore");
    run(&worker).await;
    assert_eq!(count(&f, CHUNKS).await, 2);
    f.finish().await;
}

#[tokio::test]
async fn a_rule_added_later_takes_effect_when_the_index_is_requeued() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    article_with(&f, "guide", RagMode::Default, "# A\n\nUna frase.").await;
    let (embedder, summarizer) = (FakeEmbedder::named("test:model"), FakeSummarizer::shared());
    let worker = indexer(&f.repo, &embedder, &summarizer);
    run(&worker).await;
    assert_eq!(count(&f, CHUNKS).await, 0);
    include_everything(&f).await;
    assert_eq!(f.repo.requeue_index().await.expect("requeue"), 1);
    run(&worker).await;
    assert_eq!(count(&f, CHUNKS).await, 1);
    f.finish().await;
}
