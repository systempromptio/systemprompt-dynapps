//! Offset paging over a simulated Graph: `$skip`/`$top` handed to Graph
//! where it takes them, the `@odata.nextLink` walk where it refuses `$skip`
//! (in large pages, stepping down when a page size is refused), a deep
//! `skip` reached or refused with an error, the empty first page with a
//! cursor, and cursors off the endpoint.

use serde_json::{Value, json};
use systemprompt_mcp_microsoft_365::graph::{GraphError, GraphLimits, GraphQuery, Window};

use crate::m365_support::{FakeTokens, Reply, Seen, Sim, client, collection, fast_limits};

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
    assert_eq!(
        calls[1].q("$top"),
        Some("5"),
        "one page as large as skip + top is asked for"
    );
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
    let limits = GraphLimits {
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

fn skip_refused() -> Reply {
    Reply::json(
        400,
        &json!({"error": {"code": "BadRequest", "message": "'$skip' is not supported by the service."}}),
    )
}

const fn walk_window(skip: u32, top: u32) -> Window {
    Window {
        skip,
        top: Some(top),
    }
}

fn numbered(from: usize, to: usize) -> Vec<String> {
    (from..to).map(|i| format!("i{i}")).collect()
}

// A collection of `len` items (`i0`, `i1`, …) that refuses `$skip` and pages
// through `$skiptoken`: `default` items a page unless `$top` asks otherwise
// (the cursor then carries it), and a `$top` over `cap` refused with a 400,
// as Teams refuses one over 50.
fn cursor_only(
    len: usize,
    default: usize,
    cap: usize,
) -> impl Fn(&Seen, usize) -> Reply + Send + Sync + 'static {
    move |seen, _| {
        if seen.q("$skip").is_some() {
            return skip_refused();
        }
        let size = seen.q("$top").map_or(default, |top| top.parse().unwrap());
        if size > cap {
            return Reply::json(
                400,
                &json!({"error": {"code": "BadRequest", "message": format!("Invalid page size {size}; the maximum is {cap}.")}}),
            );
        }
        let from: usize = seen
            .q("$skiptoken")
            .map_or(0, |token| token.parse().unwrap());
        let to = (from + size).min(len);
        let page: Vec<Value> = (from..to)
            .map(|i| json!({ "id": format!("i{i}") }))
            .collect();
        let next = (to < len).then(|| {
            let host = seen.header("host").unwrap();
            let top = seen
                .q("$top")
                .map(|top| format!("$top={top}&"))
                .unwrap_or_default();
            format!("http://{host}{}?{top}$skiptoken={to}", seen.path)
        });
        Reply::json(200, &collection(&page, next.as_deref()))
    }
}

// Five items a page whatever `$top` says, and always a cursor to more.
fn five_a_page_forever(seen: &Seen, _: usize) -> Reply {
    if seen.q("$skip").is_some() {
        return skip_refused();
    }
    let from: usize = seen
        .q("$skiptoken")
        .map_or(0, |token| token.parse().unwrap());
    let page: Vec<Value> = (from..from + 5)
        .map(|i| json!({ "id": format!("i{i}") }))
        .collect();
    let host = seen.header("host").unwrap();
    let next = format!("http://{host}{}?$skiptoken={}", seen.path, from + 5);
    Reply::json(200, &collection(&page, Some(&next)))
}

#[tokio::test]
async fn m365_core_paging_a_deep_skip_is_reached_where_skip_is_refused() {
    let sim = Sim::start(cursor_only(1000, 20, 999)).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let page = graph
        .get_collection("/users", &GraphQuery::new(), walk_window(200, 10))
        .await
        .unwrap();
    assert_eq!(ids(&page.items), numbered(200, 210));
    assert!(page.has_more);
    let calls = sim.calls();
    assert_eq!(calls.len(), 2, "one refused call, then one page of 210");
    assert_eq!(calls[1].q("$top"), Some("210"));
    assert_eq!(calls[1].q("$skip"), None);
}

#[tokio::test]
async fn m365_core_paging_a_refused_page_size_steps_down_to_fifty() {
    let sim = Sim::start(cursor_only(1000, 20, 50)).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let page = graph
        .get_collection(
            "/chats/C1/messages",
            &GraphQuery::new(),
            walk_window(200, 10),
        )
        .await
        .unwrap();
    assert_eq!(ids(&page.items), numbered(200, 210));
    assert!(page.has_more);
    let tops: Vec<Option<String>> = sim
        .calls()
        .iter()
        .map(|call| call.q("$top").map(str::to_owned))
        .collect();
    assert_eq!(
        tops[..3],
        [
            Some("10".to_owned()),
            Some("210".to_owned()),
            Some("50".to_owned())
        ],
        "the fast path, the refused 210, then pages of 50"
    );
    assert_eq!(tops.len(), 7, "three calls, then four more pages of 50");
}

#[tokio::test]
async fn m365_core_paging_an_unreachable_skip_is_an_error_not_an_empty_page() {
    let sim = Sim::start(five_a_page_forever).await;
    let limits = GraphLimits {
        max_pages: 3,
        ..fast_limits()
    };
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), limits);
    let result = graph
        .get_collection("/sites", &GraphQuery::new(), walk_window(200, 10))
        .await;
    let message = match &result {
        Err(GraphError::Invalid(message)) => message.clone(),
        other => panic!("an unreachable skip must be an error, got {other:?}"),
    };
    assert!(message.contains("skip=200"), "{message}");
    assert!(message.contains("narrow"), "{message}");
    assert_eq!(sim.calls().len(), 4, "one refused call, then the page cap");
    let mcp = rmcp::ErrorData::from(result.unwrap_err());
    assert_eq!(mcp.code, rmcp::model::ErrorCode::INVALID_PARAMS);
}

#[tokio::test]
async fn m365_core_paging_a_window_cut_by_the_page_cap_keeps_what_it_reached() {
    let sim = Sim::start(five_a_page_forever).await;
    let limits = GraphLimits {
        max_pages: 3,
        ..fast_limits()
    };
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), limits);
    let page = graph
        .get_collection("/sites", &GraphQuery::new(), walk_window(12, 10))
        .await
        .unwrap();
    assert_eq!(ids(&page.items), numbered(12, 15));
    assert!(page.has_more, "the cursor was left unfollowed at the cap");
}

#[tokio::test]
async fn m365_core_paging_a_collection_that_refuses_top_is_walked_without_it() {
    let sim = Sim::start(|seen: &Seen, _| {
        if seen.q("$skip").is_some() {
            return skip_refused();
        }
        if seen.q("$top").is_some() {
            return Reply::json(
                400,
                &json!({"error": {"code": "BadRequest", "message": "Query option 'Top' is not allowed."}}),
            );
        }
        cursor_only(10, 3, usize::MAX)(seen, 0)
    })
    .await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let page = graph
        .get_collection(
            "/planner/plans/P1/tasks",
            &GraphQuery::new(),
            walk_window(4, 3),
        )
        .await
        .unwrap();
    assert_eq!(ids(&page.items), numbered(4, 7));
    assert!(page.has_more);
    let calls = sim.calls();
    assert_eq!(
        calls.len(),
        5,
        "skip refused, top refused, then three pages"
    );
    assert_eq!(calls[1].q("$top"), Some("7"));
    assert!(calls[2..].iter().all(|call| call.q("$top").is_none()));
}
