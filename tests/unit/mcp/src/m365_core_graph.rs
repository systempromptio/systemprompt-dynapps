//! The Graph client against a simulated Graph: 429 retried after
//! `Retry-After` within bounds, 401 answered with one renewed token,
//! redirects followed without the bearer leaving Graph, and the response
//! size limit.


use std::sync::Arc;

use axum::http::Method;
use serde_json::json;
use systemprompt_mcp_microsoft_365::graph::{GraphError, GraphLimits, GraphQuery, GraphRequest};

use crate::m365_support::{FakeTokens, Reply, Sim, client, fast_limits};

fn ok_body() -> Reply {
    Reply::json(200, &json!({"id": "me"}))
}

#[tokio::test]
async fn m365_core_throttle_is_retried_after_retry_after() {
    let sim = Sim::start(|_, i| {
        if i == 0 {
            Reply::json(
                429,
                &json!({"error": {"code": "TooManyRequests", "message": "slow down"}}),
            )
            .with_header("retry-after", "0")
        } else {
            ok_body()
        }
    })
    .await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let body = graph.get_json("/me", &GraphQuery::new()).await.unwrap();
    assert_eq!(body["id"], "me");
    assert_eq!(sim.calls().len(), 2);
}

#[tokio::test]
async fn m365_core_throttle_beyond_the_wait_bound_is_returned_not_slept() {
    let sim =
        Sim::start(|_, _| Reply::json(429, &json!({})).with_header("retry-after", "120")).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let error = graph.get_json("/me", &GraphQuery::new()).await.unwrap_err();
    assert!(
        matches!(
            error,
            GraphError::Throttled {
                retry_after_secs: 120
            }
        ),
        "{error:?}"
    );
    assert_eq!(sim.calls().len(), 1);
}

#[tokio::test]
async fn m365_core_persistent_throttle_stops_after_the_retry_budget() {
    let sim = Sim::start(|_, _| Reply::json(429, &json!({})).with_header("retry-after", "0")).await;
    let limits = fast_limits();
    let budget = limits.max_throttle_retries as usize;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), limits);
    let error = graph.get_json("/me", &GraphQuery::new()).await.unwrap_err();
    assert!(matches!(error, GraphError::Throttled { .. }), "{error:?}");
    assert_eq!(sim.calls().len(), budget + 1);
}

#[tokio::test]
async fn m365_core_unauthorized_renews_the_token_once_and_retries() {
    let sim = Sim::start(|seen, _| match seen.header("authorization") {
        Some("Bearer fresh") => ok_body(),
        _ => Reply::json(
            401,
            &json!({"error": {"code": "InvalidAuthenticationToken", "message": "expired"}}),
        ),
    })
    .await;
    let tokens = FakeTokens::new(&["stale", "fresh"]);
    let graph = client(
        &sim.graph_url(),
        Arc::<FakeTokens>::clone(&tokens),
        fast_limits(),
    );
    assert_eq!(
        graph.get_json("/me", &GraphQuery::new()).await.unwrap()["id"],
        "me"
    );
    assert_eq!(tokens.asked(), vec![None, Some("stale".to_owned())]);
    let calls = sim.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].header("authorization"), Some("Bearer stale"));
    assert_eq!(calls[1].header("authorization"), Some("Bearer fresh"));

    // The renewed token is kept for the rest of the call.
    assert_eq!(
        graph.get_json("/me", &GraphQuery::new()).await.unwrap()["id"],
        "me"
    );
    assert_eq!(tokens.asked().len(), 2);
}

#[tokio::test]
async fn m365_core_unauthorized_with_no_new_token_asks_to_reconnect() {
    let sim = Sim::start(|_, _| Reply::json(401, &json!({}))).await;
    let same = FakeTokens::new(&["same"]);
    let graph = client(
        &sim.graph_url(),
        Arc::<FakeTokens>::clone(&same),
        fast_limits(),
    );
    let error = graph.get_json("/me", &GraphQuery::new()).await.unwrap_err();
    assert!(matches!(error, GraphError::Unauthorized), "{error:?}");
    assert_eq!(sim.calls().len(), 1, "an unchanged token is not re-sent");

    let rotating = FakeTokens::new(&["a", "b", "c"]);
    let graph = client(
        &sim.graph_url(),
        Arc::<FakeTokens>::clone(&rotating),
        fast_limits(),
    );
    let error = graph.get_json("/me", &GraphQuery::new()).await.unwrap_err();
    assert!(matches!(error, GraphError::Unauthorized), "{error:?}");
    assert_eq!(rotating.asked().len(), 2, "one renewal, then stop");
    assert_eq!(sim.calls().len(), 3);
}

#[tokio::test]
async fn m365_core_redirects_are_followed_without_the_bearer() {
    let download = Sim::start(|_, _| Reply::bytes(200, b"file-bytes")).await;
    let target = format!("{}/download/abc?sig=1", download.base);
    let graph_sim =
        Sim::start(move |_, _| Reply::bytes(302, b"").with_header("location", target.clone()))
            .await;
    let graph = client(
        &graph_sim.graph_url(),
        FakeTokens::new(&["t1"]),
        fast_limits(),
    );

    let response = graph
        .send(&GraphRequest::new(
            Method::GET,
            "/me/drive/items/01ABC/content",
        ))
        .await
        .unwrap();
    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"file-bytes");
    assert_eq!(
        graph_sim.calls()[0].header("authorization"),
        Some("Bearer t1")
    );
    let fetched = download.calls();
    assert_eq!(fetched.len(), 1);
    assert_eq!(fetched[0].path, "/download/abc");
    assert_eq!(
        fetched[0].header("authorization"),
        None,
        "the bearer must not follow a cross-host redirect"
    );
}

#[tokio::test]
async fn m365_core_responses_over_the_limit_are_refused() {
    let sim = Sim::start(|_, _| Reply::bytes(200, &[b'x'; 4096])).await;
    let limits = GraphLimits {
        max_response_bytes: 1024,
        ..fast_limits()
    };
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), limits);
    let error = graph
        .send(&GraphRequest::new(Method::GET, "/me/photo/$value"))
        .await
        .unwrap_err();
    assert!(
        matches!(error, GraphError::TooLarge { limit: 1024 }),
        "{error:?}"
    );
}

#[tokio::test]
async fn m365_core_graph_errors_carry_status_and_message() {
    let sim = Sim::start(|_, _| {
        Reply::json(404, &json!({"error": {"code": "ErrorItemNotFound", "message": "The specified object was not found in the store."}}))
    })
    .await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let error = graph
        .get_json("/me/messages/AAA=", &GraphQuery::new())
        .await
        .unwrap_err();
    match &error {
        GraphError::Api {
            status,
            code,
            message,
        } => {
            assert_eq!(*status, 404);
            assert_eq!(code.as_deref(), Some("ErrorItemNotFound"));
            assert!(message.contains("not found"));
        },
        other => panic!("expected an API error, got {other:?}"),
    }
    let mcp = rmcp::ErrorData::from(error);
    assert_eq!(mcp.code, rmcp::model::ErrorCode::RESOURCE_NOT_FOUND);
    assert!(
        !mcp.message.contains("127.0.0.1"),
        "no URL in the message: {}",
        mcp.message
    );
}
