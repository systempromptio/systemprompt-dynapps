//! Offset paging over a simulated Graph: `$skip`/`$top` handed to Graph
//! where it takes them, the `@odata.nextLink` walk where it refuses `$skip`,
//! the empty first page with a cursor, and cursors off the endpoint.

use serde_json::{Value, json};
use systemprompt_mcp_microsoft_365::graph::{GraphError, GraphQuery, Window};

use crate::m365_support::{FakeTokens, Reply, Sim, client, collection, fast_limits};

fn items(ids: &[&str]) -> Vec<Value> {
    ids.iter().map(|id| json!({ "id": id })).collect()
}

fn ids(values: &[Value]) -> Vec<String> {
    values
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn m365_core_paging_hands_skip_and_top_to_graph() {
    let sim = Sim::start(|seen, _| {
        let next = format!(
            "http://{}/v1.0/me/contacts?$skip=15",
            seen.header("host").unwrap()
        );
        Reply::json(200, &collection(&items(&["c11", "c12"]), Some(&next)))
    })
    .await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let window = Window {
        skip: 10,
        top: Some(5),
    };
    let page = graph
        .get_collection(
            "/me/contacts",
            &GraphQuery::new().with("$select", "id"),
            window,
        )
        .await
        .unwrap();
    assert_eq!(ids(&page.items), ["c11", "c12"]);
    assert!(page.has_more);
    let calls = sim.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].path, "/v1.0/me/contacts");
    assert_eq!(calls[0].q("$top"), Some("5"));
    assert_eq!(calls[0].q("$skip"), Some("10"));
    assert_eq!(calls[0].q("$select"), Some("id"));
}

#[tokio::test]
async fn m365_core_paging_walks_next_link_where_skip_is_refused() {
    let sim = Sim::start(|seen, _| {
        let host = seen.header("host").unwrap().to_owned();
        if seen.q("$skip").is_some() {
            return Reply::json(
                400,
                &json!({"error": {"code": "BadRequest", "message": "Query option 'Skip' is not allowed."}}),
            );
        }
        if seen.q("$skiptoken") == Some("p2") {
            return Reply::json(200, &collection(&items(&["m4", "m5", "m6"]), None));
        }
        let next = format!("http://{host}/v1.0/teams/T1/members?$skiptoken=p2");
        Reply::json(200, &collection(&items(&["m1", "m2", "m3"]), Some(&next)))
    })
    .await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let window = Window {
        skip: 2,
        top: Some(3),
    };
    let page = graph
        .get_collection("/teams/T1/members", &GraphQuery::new(), window)
        .await
        .unwrap();
    assert_eq!(ids(&page.items), ["m3", "m4", "m5"]);
    assert!(page.has_more, "m6 lies beyond the window");
    let calls = sim.calls();
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[1].q("$skip"), None);
    assert_eq!(calls[1].q("$top"), Some("3"));
    assert_eq!(calls[2].q("$skiptoken"), Some("p2"));
    assert!(
        calls
            .iter()
            .all(|c| c.header("authorization") == Some("Bearer t1"))
    );
}

#[tokio::test]
async fn m365_core_paging_follows_a_cursor_behind_an_empty_first_page() {
    let sim = Sim::start(|seen, _| {
        if seen.q("$skiptoken").is_some() {
            return Reply::json(200, &collection(&items(&["a", "b"]), None));
        }
        let next = format!(
            "http://{}/v1.0/chats/C1/members?$skiptoken=x",
            seen.header("host").unwrap()
        );
        Reply::json(200, &collection(&[], Some(&next)))
    })
    .await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let page = graph
        .get_collection(
            "/chats/C1/members",
            &GraphQuery::new(),
            Window {
                skip: 0,
                top: Some(10),
            },
        )
        .await
        .unwrap();
    assert_eq!(ids(&page.items), ["a", "b"]);
    assert!(!page.has_more);
    assert_eq!(sim.calls().len(), 2);
}

#[tokio::test]
async fn m365_core_paging_refuses_a_cursor_off_the_endpoint() {
    let sim = Sim::start(|_, _| {
        Reply::json(
            200,
            &collection(
                &[],
                Some("https://evil.example/v1.0/me/messages?$skiptoken=x"),
            ),
        )
    })
    .await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let error = graph
        .get_collection(
            "/me/messages",
            &GraphQuery::new(),
            Window {
                skip: 0,
                top: Some(10),
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(error, GraphError::OffEndpoint), "{error:?}");
    assert_eq!(
        sim.calls().len(),
        1,
        "the off-endpoint cursor is never requested"
    );
}

#[tokio::test]
async fn m365_core_paging_walk_stops_at_the_page_cap() {
    let sim = Sim::start(|seen, i| {
        if seen.q("$skip").is_some() {
            return Reply::json(
                400,
                &json!({"error": {"code": "BadRequest", "message": "$skip is not supported on this API."}}),
            );
        }
        let next = format!("http://{}/v1.0/sites?$skiptoken={i}", seen.header("host").unwrap());
        Reply::json(200, &collection(&items(&[&format!("s{i}")]), Some(&next)))
    })
    .await;
    let limits = systemprompt_mcp_microsoft_365::graph::GraphLimits {
        max_pages: 3,
        ..fast_limits()
    };
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), limits);
    let page = graph
        .get_collection("/sites", &GraphQuery::new(), Window { skip: 1, top: None })
        .await
        .unwrap();
    assert_eq!(ids(&page.items), ["s2", "s3"]);
    assert!(page.has_more, "a cursor was left unfollowed");
    assert_eq!(sim.calls().len(), 4, "one refused call, then three pages");
}
