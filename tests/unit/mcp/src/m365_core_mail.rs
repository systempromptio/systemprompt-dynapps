//! `d365_list_mail` end to end against a simulated Graph: the request it
//! builds (folder, `$select`, `$filter` with escaped literals, `$search`
//! without `$orderby`), the Belgian reply shape, and refusals that never
//! reach Graph.

use serde_json::{Value, json};
use systemprompt_mcp_microsoft_365::graph::GraphError;
use systemprompt_mcp_microsoft_365::tools::mail::{ListMailInput, MAIL_LIST_FIELDS, list_mail};
use systemprompt_mcp_microsoft_365::tools::reply::{GraphList, MORE_HINT};

use crate::m365_support::{FakeTokens, Reply, Sim, client, collection, fast_limits};

fn input(value: Value) -> ListMailInput {
    serde_json::from_value(value).unwrap()
}

#[tokio::test]
async fn m365_core_list_mail_filters_and_pages_like_the_belgian_server() {
    let sim = Sim::start(|seen, _| {
        let next = format!(
            "http://{}/v1.0/me/mailFolders/Inbox/messages?$skip=50",
            seen.header("host").unwrap()
        );
        Reply::json(
            200,
            &collection(&[json!({"id": "m1", "subject": "Q3"})], Some(&next)),
        )
    })
    .await;
    let graph = client(
        &sim.graph_url(),
        FakeTokens::new(&["graph-token"]),
        fast_limits(),
    );
    let args = input(json!({
        "folder": "Inbox",
        "top": 500,
        "is_read": false,
        "from_address": "o'brien@contoso.com",
        "received_after": "2024-01-01T00:00:00Z",
        "search": "ignored when filtering"
    }));
    let reply = GraphList::from(list_mail(&graph, &args).await.unwrap());

    let calls = sim.calls();
    assert_eq!(calls.len(), 1);
    let call = &calls[0];
    assert_eq!(call.method, "GET");
    assert_eq!(call.path, "/v1.0/me/mailFolders/Inbox/messages");
    assert_eq!(call.header("authorization"), Some("Bearer graph-token"));
    assert_eq!(call.q("$top"), Some("50"));
    assert_eq!(call.q("$skip"), None);
    assert_eq!(call.q("$select"), Some(MAIL_LIST_FIELDS));
    assert_eq!(call.q("$orderby"), Some("receivedDateTime desc"));
    assert_eq!(
        call.q("$filter"),
        Some(
            "isRead eq false and from/emailAddress/address eq 'o''brien@contoso.com' and receivedDateTime ge 2024-01-01T00:00:00Z"
        )
    );
    assert_eq!(call.q("$search"), None);
    assert!(
        !call.raw_query.as_deref().unwrap().contains('+'),
        "spaces travel as %20"
    );

    let wire = serde_json::to_value(&reply).unwrap();
    assert_eq!(
        wire,
        json!({
            "data": [{"id": "m1", "subject": "Q3"}],
            "message": "success",
            "has_more": true,
            "total_returned": 1,
            "hint": MORE_HINT
        })
    );
}

#[tokio::test]
async fn m365_core_list_mail_search_drops_orderby_and_quotes_the_query() {
    let sim = Sim::start(|_, _| Reply::json(200, &collection(&[json!({"id": "m9"})], None))).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let args = input(json!({"search": "budget \"q3\"", "skip": 25, "top": 0}));
    let reply = GraphList::from(list_mail(&graph, &args).await.unwrap());

    let call = &sim.calls()[0];
    assert_eq!(call.path, "/v1.0/me/mailFolders/inbox/messages");
    assert_eq!(call.q("$search"), Some(r#""budget \"q3\"""#));
    assert_eq!(call.q("$orderby"), None);
    assert_eq!(call.q("$filter"), None);
    assert_eq!(call.q("$top"), Some("1"));
    assert_eq!(call.q("$skip"), Some("25"));
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": [{"id": "m9"}], "message": "success"})
    );
}

#[tokio::test]
async fn m365_core_list_mail_refuses_bad_input_before_calling_graph() {
    let sim = Sim::start(|_, _| Reply::json(200, &collection(&[], None))).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    for bad in [
        json!({"folder": "../../users/ceo@contoso.com/mailFolders/inbox"}),
        json!({"received_before": "yesterday"}),
        json!({"received_after": "2024-01-01T00:00:00Z) or (true"}),
    ] {
        let error = list_mail(&graph, &input(bad.clone())).await.unwrap_err();
        assert!(matches!(error, GraphError::Invalid(_)), "{bad}: {error:?}");
    }
    assert!(sim.calls().is_empty());
}
