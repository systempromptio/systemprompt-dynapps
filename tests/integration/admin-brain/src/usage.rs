//! Usage: what searches returned and cited, and what found nothing.

use dynbrain::brain::Brain;
use dynbrain::brain::args::{AnswerArgs, SearchArgs};

use crate::env::{Env, user};

#[tokio::test]
async fn usage_counts_the_documents_searches_return_and_answers_cite() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let ask = SearchArgs {
        query: "apple".to_owned(),
        ..SearchArgs::default()
    };
    env.native.search(&user("root"), &ask).await.expect("search");
    env.native.search(&user("root"), &ask).await.expect("search");
    let answer = AnswerArgs {
        question: "apple".to_owned(),
        ..AnswerArgs::default()
    };
    env.native
        .answer(&user("root"), &answer)
        .await
        .expect("answer");
    let page = env.get("/brain/usage", true).await;
    assert!(page.has("Invoices"), "the article most returned");
    let counts = env
        .sql_for(
            "SELECT count(*) FILTER (WHERE kind = 'search') || ':' || count(*) FILTER (WHERE kind = 'answer')
               FROM dynbrain_search_log WHERE $1 = ANY(article_ids)",
            env.invoicing.id.0,
        )
        .await;
    assert_eq!(counts, "2:1");
    env.finish().await;
}

#[tokio::test]
async fn a_search_that_finds_nothing_is_listed_with_its_count() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let ask = SearchArgs {
        query: "nothing carol may read".to_owned(),
        ..SearchArgs::default()
    };
    for _ in 0..2 {
        env.native.search(&user("carol"), &ask).await.expect("search");
    }
    let usage = env.get("/brain/usage", true).await;
    assert!(usage.has("nothing carol may read"));
    let overview = env.get("/brain", true).await;
    assert!(overview.has("nothing carol may read"));
    let times = env
        .sql_text("SELECT count(*)::text FROM dynbrain_search_log WHERE cardinality(article_ids) = 0")
        .await;
    assert_eq!(times, "2");
    env.finish().await;
}

#[tokio::test]
async fn the_log_keeps_no_person_and_no_place() {
    let db = dynapps_test_common::db_or_skip!();
    let env = Env::new(db).await;
    let ask = SearchArgs {
        query: "apple".to_owned(),
        ..SearchArgs::default()
    };
    env.native.search(&user("root"), &ask).await.expect("search");
    let columns = env
        .sql_text(
            "SELECT string_agg(column_name, ',' ORDER BY column_name) FROM information_schema.columns
              WHERE table_name = 'dynbrain_search_log'",
        )
        .await;
    assert_eq!(columns, "article_ids,created_at,id,kind,query_text,space_id");
    env.finish().await;
}
