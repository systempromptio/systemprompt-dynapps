//! Microsoft Search and `d365_batch` against a simulated Graph: the hits of
//! the first container (or none), and a batch normalised as the Belgian
//! server did — default ids and methods, JSON bodies typed, the 20-request
//! cap and refusals answered as a message without calling Graph.

use serde_json::json;
use systemprompt_mcp_microsoft_365::tools::search::{batch, search, search_messages};

use crate::m365_pim_support::{args, assert_domain_registered, graph};
use crate::m365_support::{Reply, Sim};

#[test]
fn m365_rest_search_and_batch_are_registered_as_pinned() {
    assert_domain_registered("search", 2);
    assert_domain_registered("batch", 1);
}

#[tokio::test]
async fn m365_rest_search_returns_the_hits_of_the_first_container() {
    let hits = json!([{"hitId": "h-1", "summary": "Budget"}]);
    let reply = json!({"value": [{"hitsContainers": [{"hits": hits.clone(), "total": 1}]}]});
    let sim = Sim::start(
        move |seen, _| match seen.json()["requests"][0]["size"].as_u64() {
            Some(10) => Reply::json(200, &reply),
            _ => Reply::json(200, &json!({"value": []})),
        },
    )
    .await;
    let client = graph(&sim);
    let found = search(&client, args(json!({"query": "budget 'Q1'"})))
        .await
        .unwrap();
    let narrowed = search(
        &client,
        args(json!({"query": "x", "entity_types": ["driveItem"], "top": 3})),
    )
    .await
    .unwrap();
    let mail = search_messages(&client, args(json!({"query": "invoice"})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(
        (calls[0].method.as_str(), calls[0].path.as_str()),
        ("POST", "/v1.0/search/query")
    );
    assert_eq!(
        calls[0].json(),
        json!({"requests": [{
            "entityTypes": ["message", "driveItem", "listItem", "site"],
            "query": {"queryString": "budget ''Q1''"}, "from": 0, "size": 10
        }]})
    );
    assert_eq!(found.data, hits);
    assert_eq!(
        calls[1].json()["requests"][0]["entityTypes"],
        json!(["driveItem"])
    );
    assert_eq!(calls[1].json()["requests"][0]["size"], 3);
    assert_eq!(narrowed.data, json!([]), "no container means no hits");
    assert_eq!(
        calls[2].json()["requests"][0]["entityTypes"],
        json!(["message"])
    );
    assert_eq!(calls[2].json()["requests"][0]["size"], 25);
    assert_eq!(mail.message, "success");
}

#[tokio::test]
async fn m365_rest_search_batch_is_normalised_and_sorted_by_id() {
    let sim = Sim::start(|_, _| {
        Reply::json(
            200,
            &json!({"responses": [
                {"id": "req1", "status": 201, "body": {}},
                {"id": "a", "status": 200, "body": {"id": "me"}}
            ]}),
        )
    })
    .await;
    let reply = batch(
        &graph(&sim),
        args(json!({"requests": [
            {"id": "a", "method": "get", "url": "/me", "headers": {"Prefer": "x"}},
            {"method": "POST", "url": "/me/todo/lists", "body": {"displayName": "L"}},
            {"id": 7, "method": "PATCH", "url": "/me/events/e", "body": {"subject": "s"},
             "headers": {"Content-Type": "application/merge-patch+json"}}
        ]})),
    )
    .await
    .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(
        (call.method.as_str(), call.path.as_str()),
        ("POST", "/v1.0/$batch")
    );
    assert_eq!(
        call.json(),
        json!({"requests": [
            {"id": "a", "method": "GET", "url": "/me", "headers": {"Prefer": "x"}},
            {"id": "req1", "method": "POST", "url": "/me/todo/lists",
             "body": {"displayName": "L"}, "headers": {"Content-Type": "application/json"}},
            {"id": "7", "method": "PATCH", "url": "/me/events/e", "body": {"subject": "s"},
             "headers": {"Content-Type": "application/merge-patch+json"}}
        ]})
    );
    assert_eq!(reply.data[0]["id"], "a");
    assert_eq!(reply.data[1]["id"], "req1");
}

#[tokio::test]
async fn m365_rest_search_batch_refusals_are_messages_and_reach_no_graph() {
    let sim = Sim::start(|_, _| Reply::json(200, &json!({"responses": []}))).await;
    let client = graph(&sim);
    let many: Vec<_> = (0..21).map(|i| json!({"id": i, "url": "/me"})).collect();
    let cases = [
        (json!([]), "Provide a non-empty list of request objects"),
        (json!(many), "Too many requests: 21. Graph batch max is 20."),
        (
            json!([{"id": "x", "method": "GET"}]),
            "Request x has no 'url'",
        ),
        (
            json!([{"url": "/me", "method": "HEAD"}]),
            "Request req0 method 'HEAD' not allowed",
        ),
    ];
    for (requests, message) in cases {
        let reply = batch(&client, args(json!({"requests": requests})))
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(&reply).unwrap(),
            json!({"data": null, "message": message})
        );
    }
    assert!(sim.calls().is_empty());
}
