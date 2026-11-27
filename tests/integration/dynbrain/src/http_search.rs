//! The search and answer routes, paths that hide unreadable ancestors, the
//! list's `where` rule, and the Cohere client against a local server.

use std::net::SocketAddr;

use axum::http::{HeaderMap, Method, StatusCode};
use axum::routing::post;
use axum::{Json, Router};
use dynbrain::access::{Level, Subject};
use dynbrain::brain::Brain;
use dynbrain::brain::args::ListArgs;
use dynbrain::brain::fault::BrainFault;
use dynbrain::search::cohere::{CohereReranker, MODEL};
use dynbrain::search::rerank::{RerankFailure, Reranker};
use serde_json::{Value, json};

use crate::fixture::{content, user};
use crate::http_support::App;

#[tokio::test]
async fn the_search_and_answer_routes_answer_for_the_person_of_the_token() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let article = app.fixture.article(space.id, None, "backups").await;
    app.fixture
        .write(&article, "es", "Backups", "Nightly copies of everything.")
        .await;
    let found = app
        .post("/search", "alice", &json!({"query": "nightly"}))
        .await;
    assert_eq!(found.status, StatusCode::OK, "{}", found.body);
    assert_eq!(found.body["mode"], "keyword", "no embedder in this suite");
    assert_eq!(found.body["results"][0]["title"], "Backups");
    assert_eq!(found.body["results"][0]["path"], "handbook/backups");
    assert_eq!(found.body["confidence"], "unrated");
    let answer = app
        .post("/answer", "alice", &json!({"question": "nightly"}))
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{}", answer.body);
    assert_eq!(answer.body["mode"], "context");
    assert_eq!(
        answer.body["context"],
        "[1] Backups (handbook/backups, lang=es)\nNightly copies of everything."
    );
    let stranger = app
        .post("/search", "bob", &json!({"query": "nightly"}))
        .await;
    assert_eq!(stranger.body["results"], json!([]));
    let anonymous = app
        .request(Method::POST, "/search", None, Some(&json!({"query": "x"})))
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    let undeclared = app
        .post(
            "/search",
            "alice",
            &json!({"query": "x", "where": {"owner": "ana"}}),
        )
        .await;
    assert_eq!(undeclared.status, StatusCode::BAD_REQUEST);
    app.finish().await;
}

#[tokio::test]
async fn a_path_hides_the_ancestors_its_reader_cannot_read() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let f = &app.fixture;
    let space = f.space("handbook").await;
    let top = f.article(space.id, None, "board").await;
    let middle = f.article(space.id, Some(top.id), "salaries").await;
    let leaf = f.article(space.id, Some(middle.id), "holidays").await;
    for (article, title) in [(&top, "Board"), (&middle, "Salaries"), (&leaf, "Holidays")] {
        f.write(article, "es", title, "Holiday calendar.").await;
    }
    f.give(
        space.id,
        Some(leaf.id),
        Subject::user(&user("bob")),
        Level::Read,
    )
    .await;
    let bob = app.get(&format!("/articles/{}", leaf.id.0), "bob").await;
    assert_eq!(bob.status, StatusCode::OK, "{}", bob.body);
    assert_eq!(bob.body["path"], "handbook/…/holidays");
    let alice = app.get(&format!("/articles/{}", leaf.id.0), "alice").await;
    assert_eq!(alice.body["path"], "handbook/board/salaries/holidays");
    let found = app
        .post("/search", "bob", &json!({"query": "holiday"}))
        .await;
    assert_eq!(found.body["results"][0]["path"], "handbook/…/holidays");
    assert!(!found.body.to_string().contains("salaries"));
    app.finish().await;
}

#[tokio::test]
async fn a_list_where_names_declared_properties_and_false_means_not_set() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let f = &app.fixture;
    let space = f.space("handbook").await;
    let decided = f.article(space.id, None, "decided").await;
    let bare = f.article(space.id, None, "bare").await;
    let mut write = content("Decided", "x");
    write
        .properties
        .insert("status".to_owned(), json!("decided"));
    f.repo
        .write_variant(decided.id, "es", &write, &user("alice"))
        .await
        .expect("write a variant");
    f.write(&bare, "es", "Bare", "x").await;
    let list = |filter: Value| ListArgs {
        space: space.id,
        filter: filter.as_object().cloned(),
        ..ListArgs::default()
    };
    let unset = app
        .brain
        .list(&user("alice"), &list(json!({"status": false})))
        .await
        .expect("list");
    let titles: Vec<&str> = unset
        .documents
        .iter()
        .map(|doc| doc.title.as_str())
        .collect();
    assert_eq!(titles, ["Bare"]);
    let unknown = app
        .brain
        .list(&user("alice"), &list(json!({"owner": "ana"})))
        .await;
    assert!(matches!(unknown, Err(BrainFault::Property(_))));
    app.finish().await;
}

async fn rerank_server() -> SocketAddr {
    let routes = Router::new()
        .route(
            "/v2/rerank",
            post(|headers: HeaderMap, Json(body): Json<Value>| async move {
                let keyed = headers
                    .get("authorization")
                    .and_then(|value| value.to_str().ok())
                    == Some("Bearer test-key");
                if !keyed {
                    return (StatusCode::UNAUTHORIZED, Json(json!({})));
                }
                let asked = body["model"] == MODEL && body["top_n"] == 2 && body["query"] == "q";
                if !asked {
                    return (StatusCode::BAD_REQUEST, Json(json!({})));
                }
                (
                    StatusCode::OK,
                    Json(json!({"results": [
                        {"index": 1, "relevance_score": 0.91},
                        {"index": 0, "relevance_score": 0.12}
                    ]})),
                )
            }),
        )
        .route(
            "/down",
            post(|| async { (StatusCode::SERVICE_UNAVAILABLE, "down") }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind a local port");
    let address = listener.local_addr().expect("the local address");
    tokio::spawn(async move { axum::serve(listener, routes).await });
    address
}

// Why: reqwest's rustls has no default provider; core installs ring at
// server start, which this suite never runs.
fn install_tls() {
    if rustls::crypto::ring::default_provider()
        .install_default()
        .is_err()
    {
        // already installed by an earlier test in this process
    }
}

#[tokio::test]
async fn the_cohere_client_sends_its_key_and_reads_the_scores() {
    install_tls();
    let address = rerank_server().await;
    let endpoint = format!("http://{address}/v2/rerank");
    let documents = ["first".to_owned(), "second".to_owned()];
    let cohere = CohereReranker::new("test-key".to_owned(), &endpoint, MODEL).expect("a client");
    assert_eq!(cohere.provider(), "cohere");
    let scores = cohere.rerank("q", &documents).await.expect("scores");
    assert_eq!(scores, [(1, 0.91), (0, 0.12)]);
    let wrong = CohereReranker::new("other".to_owned(), &endpoint, MODEL).expect("a client");
    let refused = wrong.rerank("q", &documents).await.expect_err("refused");
    assert!(matches!(refused, RerankFailure::Status(401)));
    assert!(!refused.opens_breaker(), "a rejected key is not an outage");
}

#[tokio::test]
async fn the_cohere_client_reports_an_outage_that_opens_the_breaker() {
    install_tls();
    let address = rerank_server().await;
    let down = CohereReranker::new(
        "test-key".to_owned(),
        &format!("http://{address}/down"),
        MODEL,
    )
    .expect("a client");
    let failure = down.rerank("q", &["a".to_owned()]).await.expect_err("down");
    assert!(matches!(failure, RerankFailure::Status(503)));
    assert!(failure.opens_breaker());
    let nowhere = CohereReranker::new("test-key".to_owned(), "http://127.0.0.1:9/v2/rerank", MODEL)
        .expect("a client");
    let unreachable = nowhere
        .rerank("q", &["a".to_owned()])
        .await
        .expect_err("unreachable");
    assert!(unreachable.opens_breaker());
    assert!(CohereReranker::new("k".to_owned(), "http://10.0.0.1/rerank", MODEL).is_err());
}
