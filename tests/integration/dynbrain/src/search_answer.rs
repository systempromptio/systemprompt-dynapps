//! `answer` hands back the assembled context and its citations: numbered
//! blocks, whole documents when they fit and their best chunks otherwise,
//! with no model called.

use dynbrain::brain::Brain;
use dynbrain::brain::args::AnswerArgs;
use dynbrain::search::RetrievalSettings;
use dynbrain::search::confidence::MatchConfidence;
use dynbrain::search::context::GAP_MARK;

use crate::fixture::{Fixture, user};
use crate::search_support::{QueryEmbedder, axis, brain, chunk, doc, hybrid, retriever};

fn question(text: &str) -> AnswerArgs {
    AnswerArgs {
        question: text.to_owned(),
        ..AnswerArgs::default()
    }
}

#[tokio::test]
async fn answer_numbers_its_sources_and_cites_them_in_order() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let first = doc(&f, space.id, "first", &[("es", "First", "The first body.")]).await;
    let second = doc(
        &f,
        space.id,
        "second",
        &[("es", "Second", "The second body.")],
    )
    .await;
    chunk(&f, &first, ("es", 0), "alpha passage", Some(&axis(&[0]))).await;
    chunk(&f, &second, ("es", 0), "beta passage", Some(&axis(&[0, 1]))).await;
    let answer = hybrid(&f)
        .answer(&user("alice"), &question("passage"))
        .await
        .expect("answer");
    assert_eq!(
        answer.context,
        "[1] First (handbook/first, lang=es)\nThe first body.\n\n---\n\n[2] Second (handbook/second, lang=es)\nThe second body."
    );
    let cited: Vec<(&str, &str, &str)> = answer
        .citations
        .iter()
        .map(|cited| {
            (
                cited.path.as_str(),
                cited.title.as_str(),
                cited.snippet.as_str(),
            )
        })
        .collect();
    assert_eq!(
        cited,
        [
            ("handbook/first", "First", "alpha passage"),
            ("handbook/second", "Second", "beta passage")
        ]
    );
    let wire = serde_json::to_value(&answer).expect("serialize");
    assert_eq!(wire["mode"], "context");
    assert!(wire.get("answer").is_none(), "no model wrote anything");
    assert_eq!(wire["confidence"], "unrated");
    assert!(wire["confidence_note"].is_string());
    assert_eq!(wire["citations"][0]["lang"], "es");
    f.finish().await;
}

#[tokio::test]
async fn a_long_document_gives_its_best_chunks_whole_in_reading_order() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    let parts = [
        "Alpha one two three four.",
        "Bravo one two three four.",
        "Charlie one two three fou.",
        "Delta one two three four.",
    ];
    let long = doc(&f, space.id, "long", &[("es", "Long", &parts.join("\n\n"))]).await;
    chunk(&f, &long, ("es", 0), parts[0], Some(&axis(&[0, 1]))).await;
    chunk(&f, &long, ("es", 1), parts[1], Some(&axis(&[1]))).await;
    chunk(&f, &long, ("es", 2), parts[2], Some(&axis(&[0]))).await;
    chunk(&f, &long, ("es", 3), parts[3], Some(&axis(&[1]))).await;
    let tight = RetrievalSettings {
        context_quota_chars: 60,
        ..RetrievalSettings::default()
    };
    let native = brain(
        &f,
        retriever(&f, Some(QueryEmbedder::on(&[0])), None, tight),
    );
    let answer = native
        .answer(&user("alice"), &question("zzz"))
        .await
        .expect("answer");
    assert_eq!(
        answer.context,
        format!(
            "[1] Long (handbook/long, lang=es)\n{}\n{GAP_MARK}\n{}",
            parts[0], parts[2]
        ),
        "the two most relevant chunks that fit, whole, in reading order"
    );
    f.finish().await;
}

#[tokio::test]
async fn without_a_query_vector_the_answer_is_built_from_keyword_matches() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let space = f.space("handbook").await;
    doc(
        &f,
        space.id,
        "restore",
        &[("es", "Restore", "Restore from the nightly copy.")],
    )
    .await;
    let native = brain(&f, retriever(&f, None, None, RetrievalSettings::default()));
    let answer = native
        .answer(&user("alice"), &question("nightly"))
        .await
        .expect("answer");
    assert_eq!(
        answer.context,
        "[1] Restore (handbook/restore, lang=es)\nRestore from the nightly copy."
    );
    assert_eq!(answer.verdict.confidence, MatchConfidence::Unrated);
    f.finish().await;
}
