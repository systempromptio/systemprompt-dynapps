//! The mail read tools against a simulated Graph: the path, `$select`,
//! paging and ordering each sends, the Belgian reply shapes, and ids or
//! folders that would leave the caller's mailbox refused before any call.

use serde_json::{Value, json};
use systemprompt_mcp_microsoft_365::graph::GraphError;
use systemprompt_mcp_microsoft_365::tools::mail::MAIL_LIST_FIELDS;
use systemprompt_mcp_microsoft_365::tools::mail_read::{
    ATTACHMENT_LIST_FIELDS, ListAttachmentsInput, MAIL_DETAIL_FIELDS, MAIL_FOLDER_FIELDS,
    MessageIdInput, SearchMailInput, SharedMailboxInput, SkipInput, get_mail,
    list_mail_attachments, list_mail_folders, list_mail_rules, list_shared_mailbox_messages,
    search_mail,
};
use systemprompt_mcp_microsoft_365::tools::reply::MORE_HINT;

use crate::m365_support::{FakeTokens, Reply, Sim, client, collection, fast_limits};

fn parse<T: serde::de::DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

async fn sim_with(body: Value) -> Sim {
    Sim::start(move |_, _| Reply::json(200, &body)).await
}

#[tokio::test]
async fn m365_mail_get_mail_selects_the_detail_fields_and_answers_data_and_message() {
    let sim = sim_with(json!({"id": "AAMk=", "subject": "Q3"})).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let reply = get_mail(&graph, parse(json!({"message_id": "AAMk="})))
        .await
        .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(
        (call.method.as_str(), call.path.as_str()),
        ("GET", "/v1.0/me/messages/AAMk=")
    );
    assert_eq!(call.q("$select"), Some(MAIL_DETAIL_FIELDS));
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": {"id": "AAMk=", "subject": "Q3"}, "message": "success"})
    );

    let bad: MessageIdInput = parse(json!({"message_id": "../users/ceo@contoso.com"}));
    let error = get_mail(&graph, bad).await.unwrap_err();
    assert!(matches!(error, GraphError::Invalid(_)), "{error:?}");
    assert_eq!(sim.calls().len(), 1, "the traversal never reached Graph");
}

#[tokio::test]
async fn m365_mail_search_mail_quotes_kql_over_every_folder_without_orderby() {
    let sim = sim_with(collection(&[json!({"id": "m1"})], None)).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let input: SearchMailInput =
        parse(json!({"query": "from:o\"brien budget", "top": 99, "skip": 5}));
    let reply = search_mail(&graph, input).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.path, "/v1.0/me/messages");
    assert_eq!(call.q("$search"), Some(r#""from:o\"brien budget""#));
    assert_eq!(call.q("$select"), Some(MAIL_LIST_FIELDS));
    assert_eq!(call.q("$top"), Some("50"));
    assert_eq!(call.q("$skip"), Some("5"));
    assert_eq!(call.q("$orderby"), None);
    assert_eq!(call.q("$filter"), None);
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": [{"id": "m1"}], "message": "success"})
    );
}

#[tokio::test]
async fn m365_mail_list_mail_folders_pages_a_hundred_with_the_folder_fields() {
    let sim = sim_with(collection(
        &[json!({"id": "f1", "displayName": "Inbox"})],
        None,
    ))
    .await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let skip: SkipInput = parse(json!({"skip": 100}));
    list_mail_folders(&graph, skip).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.path, "/v1.0/me/mailFolders");
    assert_eq!(call.q("$select"), Some(MAIL_FOLDER_FIELDS));
    assert_eq!(call.q("$top"), Some("100"));
    assert_eq!(call.q("$skip"), Some("100"));
}

#[tokio::test]
async fn m365_mail_list_mail_attachments_lists_metadata_only() {
    let sim = sim_with(collection(&[json!({"id": "att1", "name": "a.pdf"})], None)).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let input: ListAttachmentsInput = parse(json!({"message_id": "AAMk-1"}));
    let reply = list_mail_attachments(&graph, input).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.path, "/v1.0/me/messages/AAMk-1/attachments");
    assert_eq!(call.q("$select"), Some(ATTACHMENT_LIST_FIELDS));
    assert_eq!(call.q("$top"), None);
    assert_eq!(call.q("$skip"), None);
    assert_eq!(reply.data.len(), 1);

    let bad: ListAttachmentsInput = parse(json!({"message_id": "AAMk/../x"}));
    assert!(list_mail_attachments(&graph, bad).await.is_err());
    assert_eq!(sim.calls().len(), 1);
}

#[tokio::test]
async fn m365_mail_list_mail_rules_reads_the_inbox_rules_and_says_when_more_exist() {
    let sim = Sim::start(|seen, _| {
        let next = format!(
            "http://{}/v1.0/me/mailFolders/inbox/messageRules?$skiptoken=2",
            seen.header("host").unwrap()
        );
        Reply::json(
            200,
            &collection(&[json!({"id": "r1", "sequence": 1})], Some(&next)),
        )
    })
    .await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let reply = list_mail_rules(&graph, parse(json!({}))).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.path, "/v1.0/me/mailFolders/inbox/messageRules");
    assert_eq!(call.raw_query, None, "no paging options on the first page");
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({
            "data": [{"id": "r1", "sequence": 1}],
            "message": "success",
            "has_more": true,
            "total_returned": 1,
            "hint": MORE_HINT
        })
    );
}

#[tokio::test]
async fn m365_mail_list_shared_mailbox_messages_reads_the_named_mailbox_newest_first() {
    let sim = sim_with(collection(&[json!({"id": "s1"})], None)).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let input: SharedMailboxInput =
        parse(json!({"user_id": "shared@contoso.com", "folder": "sentitems", "top": 10}));
    list_shared_mailbox_messages(&graph, input).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(
        call.path,
        "/v1.0/users/shared@contoso.com/mailFolders/sentitems/messages"
    );
    assert_eq!(call.q("$select"), Some(MAIL_LIST_FIELDS));
    assert_eq!(call.q("$orderby"), Some("receivedDateTime desc"));
    assert_eq!(call.q("$top"), Some("10"));

    for bad in [
        json!({"user_id": "../me"}),
        json!({"user_id": "shared@contoso.com", "folder": "inbox/../../x"}),
        json!({"user_id": "a b@contoso.com"}),
    ] {
        let input: SharedMailboxInput = parse(bad.clone());
        let error = list_shared_mailbox_messages(&graph, input)
            .await
            .unwrap_err();
        assert!(matches!(error, GraphError::Invalid(_)), "{bad}: {error:?}");
    }
    assert_eq!(sim.calls().len(), 1);
}
