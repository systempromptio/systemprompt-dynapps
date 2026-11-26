//! The reader's access inside both arms of the search, in the keyword
//! fallback and in `answer`: what a person cannot read is never returned,
//! and the trash is out of every path.

use dynbrain::access::{Level, Subject};
use dynbrain::brain::Brain;
use dynbrain::brain::args::{AnswerArgs, SearchArgs};
use dynbrain::ids::{ChunkId, SpaceId};
use dynbrain::search::filter::PropertyFilter;
use dynbrain::search::query::literal;
use dynbrain::search::{RetrievalMode, RetrievalSettings, SearchScope};

use crate::fixture::{Fixture, user};
use crate::search_support::{ask, axis, brain, chunk, doc, hybrid, let_read, retriever};

fn scope(who: &str) -> SearchScope {
    SearchScope {
        reader: user(who),
        space: None,
        lang: None,
        filter: PropertyFilter::default(),
    }
}

// `open` is readable by bob, `closed` is not; both are alice's.
async fn corpus(f: &Fixture) -> (ChunkId, ChunkId) {
    let space = f.space("handbook").await;
    let open = doc(f, space.id, "open", &[("es", "Open", "open text")]).await;
    let closed = doc(f, space.id, "closed", &[("es", "Closed", "closed secret")]).await;
    let_read(f, &open, "bob").await;
    let open_chunk = chunk(
        f,
        &open,
        ("es", 0),
        "shared word open",
        Some(&axis(&[0, 1])),
    )
    .await;
    let closed_chunk = chunk(
        f,
        &closed,
        ("es", 0),
        "shared word secret",
        Some(&axis(&[0])),
    )
    .await;
    (open_chunk, closed_chunk)
}

#[tokio::test]
async fn the_vector_arm_returns_no_chunk_of_an_article_its_reader_cannot_read() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let (open, closed) = corpus(&f).await;
    let query = literal(&axis(&[0]));
    let bob = f
        .repo
        .list_vector_matches(&scope("bob"), &query, 100, 200)
        .await
        .expect("the vector arm");
    assert_eq!(bob, [open]);
    let alice = f
        .repo
        .list_vector_matches(&scope("alice"), &query, 100, 200)
        .await
        .expect("the vector arm");
    assert_eq!(alice, [closed, open], "the owner reads both, nearest first");
    f.finish().await;
}

#[tokio::test]
async fn the_text_arm_returns_no_chunk_of_an_article_its_reader_cannot_read() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let (open, closed) = corpus(&f).await;
    let bob = f
        .repo
        .list_text_matches(&scope("bob"), "shared word", 100)
        .await
        .expect("the text arm");
    assert_eq!(bob, [open]);
    let erin = f
        .repo
        .list_text_matches(&scope("erin"), "secret", 100)
        .await
        .expect("the text arm");
    assert!(erin.is_empty(), "no grant, no chunk");
    let alice = f
        .repo
        .list_text_matches(&scope("alice"), "secret", 100)
        .await
        .expect("the text arm");
    assert_eq!(alice, [closed]);
    f.finish().await;
}

#[tokio::test]
async fn the_arms_fill_their_pool_with_what_the_reader_may_read() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    for slug in ["closed-a", "closed-b", "closed-c"] {
        let closed = doc(&f, space.id, slug, &[("es", "Closed", "word")]).await;
        chunk(&f, &closed, ("es", 0), "word word", Some(&axis(&[0]))).await;
    }
    let open = doc(&f, space.id, "open", &[("es", "Open", "word")]).await;
    let_read(&f, &open, "bob").await;
    let readable = chunk(&f, &open, ("es", 0), "word", Some(&axis(&[0, 1]))).await;
    let query = literal(&axis(&[0]));
    let near = f
        .repo
        .list_vector_matches(&scope("bob"), &query, 2, 200)
        .await
        .expect("the vector arm");
    assert_eq!(
        near,
        [readable],
        "filtered inside the query, not after a cut of 2"
    );
    let words = f
        .repo
        .list_text_matches(&scope("bob"), "word", 2)
        .await
        .expect("the text arm");
    assert_eq!(words, [readable]);
    f.finish().await;
}

#[tokio::test]
async fn search_and_answer_never_show_an_article_the_reader_cannot_read() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    corpus(&f).await;
    let native = hybrid(&f);
    let found = native
        .search(&user("bob"), &ask("shared word"))
        .await
        .expect("search");
    assert_eq!(found.mode, RetrievalMode::Hybrid);
    let titles: Vec<&str> = found.results.iter().map(|hit| hit.title.as_str()).collect();
    assert_eq!(titles, ["Open"]);
    let answer = native
        .answer(
            &user("bob"),
            &AnswerArgs {
                question: "shared word".to_owned(),
                ..AnswerArgs::default()
            },
        )
        .await
        .expect("answer");
    assert!(answer.context.contains("open text"));
    assert!(!answer.context.contains("secret"), "{}", answer.context);
    assert!(answer.citations.iter().all(|cited| cited.title == "Open"));
    let stranger = native
        .search(&user("erin"), &ask("shared word"))
        .await
        .expect("search");
    assert!(stranger.results.is_empty());
    f.finish().await;
}

#[tokio::test]
async fn the_keyword_fallback_never_returns_an_article_its_reader_cannot_read() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    corpus(&f).await;
    let native = brain(&f, retriever(&f, None, None, RetrievalSettings::default()));
    let found = native
        .search(&user("bob"), &ask("text"))
        .await
        .expect("search");
    assert_eq!(found.mode, RetrievalMode::Keyword);
    let titles: Vec<&str> = found.results.iter().map(|hit| hit.title.as_str()).collect();
    assert_eq!(titles, ["Open"]);
    let secret = native
        .search(&user("bob"), &ask("secret"))
        .await
        .expect("search");
    assert!(secret.results.is_empty());
    let owner = native
        .search(&user("alice"), &ask("secret"))
        .await
        .expect("search");
    assert_eq!(owner.results.len(), 1);
    f.finish().await;
}

#[tokio::test]
async fn a_suspended_account_finds_nothing_even_with_a_grant() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let open = doc(&f, space.id, "open", &[("es", "Open", "open text")]).await;
    let_read(&f, &open, "dave").await;
    chunk(&f, &open, ("es", 0), "open text", Some(&axis(&[0]))).await;
    let native = hybrid(&f);
    let found = native
        .search(&user("dave"), &ask("open"))
        .await
        .expect("search");
    assert!(found.results.is_empty());
    f.finish().await;
}

#[tokio::test]
async fn a_trashed_article_is_found_by_neither_arm_nor_answer() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let gone = doc(&f, space.id, "gone", &[("es", "Gone", "gone text")]).await;
    chunk(&f, &gone, ("es", 0), "gone text", Some(&axis(&[0]))).await;
    f.repo.trash_article(gone.id).await.expect("trash");
    let query = literal(&axis(&[0]));
    let near = f
        .repo
        .list_vector_matches(&scope("alice"), &query, 100, 200)
        .await
        .expect("the vector arm");
    let words = f
        .repo
        .list_text_matches(&scope("alice"), "gone", 100)
        .await
        .expect("the text arm");
    assert!(near.is_empty() && words.is_empty());
    let keywords = brain(&f, retriever(&f, None, None, RetrievalSettings::default()));
    let found = keywords
        .search(&user("alice"), &ask("gone"))
        .await
        .expect("search");
    assert!(found.results.is_empty());
    let answer = hybrid(&f)
        .answer(
            &user("alice"),
            &AnswerArgs {
                question: "gone".to_owned(),
                ..AnswerArgs::default()
            },
        )
        .await
        .expect("answer");
    assert!(answer.context.is_empty() && answer.citations.is_empty());
    f.finish().await;
}

#[tokio::test]
async fn a_space_or_a_language_keeps_the_search_inside_it() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let handbook = f.space("handbook").await;
    let other = f.space("other").await;
    let here = doc(&f, handbook.id, "here", &[("es", "Here", "word")]).await;
    let there = doc(&f, other.id, "there", &[("es", "There", "word")]).await;
    chunk(&f, &here, ("es", 0), "word", Some(&axis(&[0]))).await;
    chunk(&f, &there, ("es", 0), "word", Some(&axis(&[0]))).await;
    f.give(other.id, None, Subject::user(&user("bob")), Level::Read)
        .await;
    let native = hybrid(&f);
    let scoped = SearchArgs {
        space: Some(handbook.id),
        ..ask("word")
    };
    let found = native
        .search(&user("alice"), &scoped)
        .await
        .expect("search");
    let titles: Vec<&str> = found.results.iter().map(|hit| hit.title.as_str()).collect();
    assert_eq!(titles, ["Here"]);
    let french = SearchArgs {
        lang: Some("fr".to_owned()),
        ..ask("word")
    };
    let none = native
        .search(&user("alice"), &french)
        .await
        .expect("search");
    assert!(none.results.is_empty(), "no space is written in French");
    let unknown_space = SearchArgs {
        space: Some(SpaceId(999)),
        ..ask("word")
    };
    let empty = native
        .search(&user("bob"), &unknown_space)
        .await
        .expect("search");
    assert!(empty.results.is_empty());
    f.finish().await;
}
