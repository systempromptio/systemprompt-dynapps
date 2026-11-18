//! The mail write tools against a simulated Graph: the exact body each
//! sends, the Belgian summary a 202/204 write answers with, and inputs that
//! never reach Graph.

use serde_json::{Value, json};
use systemprompt_mcp_microsoft_365::graph::GraphError;
use systemprompt_mcp_microsoft_365::tools::mail_send::{
    create_draft, delete_mail, forward_mail, move_mail, reply_mail, send_draft, send_mail,
    update_mail,
};

use crate::m365_support::{FakeTokens, Reply, Seen, Sim, client, fast_limits};

fn parse<T: serde::de::DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

fn accepted(_: &Seen, _: usize) -> Reply {
    Reply::bytes(202, b"")
}

fn wire<T: serde::Serialize>(reply: &T) -> Value {
    serde_json::to_value(reply).unwrap()
}

#[tokio::test]
async fn m365_mail_send_mail_posts_the_message_with_every_recipient_list() {
    let sim = Sim::start(accepted).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let input = parse(json!({
        "to": ["ana@contoso.com"], "subject": "Hola", "body": "<p>Hi</p>",
        "cc": ["cc@contoso.com"], "bcc": ["bcc@contoso.com"], "is_html": true, "importance": "high"
    }));
    let reply = send_mail(&graph, input).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(
        (call.method.as_str(), call.path.as_str()),
        ("POST", "/v1.0/me/sendMail")
    );
    assert_eq!(
        call.json(),
        json!({"message": {
            "subject": "Hola",
            "body": {"contentType": "HTML", "content": "<p>Hi</p>"},
            "toRecipients": [{"emailAddress": {"address": "ana@contoso.com"}}],
            "ccRecipients": [{"emailAddress": {"address": "cc@contoso.com"}}],
            "bccRecipients": [{"emailAddress": {"address": "bcc@contoso.com"}}],
            "importance": "high"
        }})
    );
    assert_eq!(
        wire(&reply),
        json!({"data": {"sent": true, "to": ["ana@contoso.com"], "subject": "Hola"}, "message": "success"})
    );
}

#[tokio::test]
async fn m365_mail_send_mail_defaults_to_text_normal_and_no_copies() {
    let sim = Sim::start(accepted).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let input =
        parse(json!({"to": ["a@contoso.com", "b@contoso.com"], "subject": "S", "body": "B"}));
    send_mail(&graph, input).await.unwrap();

    let message = sim.calls()[0].json()["message"].clone();
    assert_eq!(
        message["body"],
        json!({"contentType": "Text", "content": "B"})
    );
    assert_eq!(message["importance"], "normal");
    assert_eq!(message["toRecipients"].as_array().unwrap().len(), 2);
    assert!(message.get("ccRecipients").is_none());
    assert!(message.get("bccRecipients").is_none());
}

#[tokio::test]
async fn m365_mail_reply_mail_picks_reply_or_reply_all() {
    let sim = Sim::start(accepted).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let one = reply_mail(&graph, parse(json!({"message_id": "m1", "body": "ok"})))
        .await
        .unwrap();
    reply_mail(
        &graph,
        parse(json!({"message_id": "m1", "body": "all", "reply_all": true})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/messages/m1/reply");
    assert_eq!(calls[0].json(), json!({"comment": "ok"}));
    assert_eq!(calls[1].path, "/v1.0/me/messages/m1/replyAll");
    assert_eq!(
        wire(&one)["data"],
        json!({"replied": true, "message_id": "m1", "reply_all": false})
    );
}

#[tokio::test]
async fn m365_mail_forward_mail_adds_the_comment_only_when_given() {
    let sim = Sim::start(accepted).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    forward_mail(
        &graph,
        parse(json!({"message_id": "m1", "to": ["x@contoso.com"]})),
    )
    .await
    .unwrap();
    let reply = forward_mail(
        &graph,
        parse(json!({"message_id": "m1", "to": ["x@contoso.com"], "comment": "FYI"})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/messages/m1/forward");
    assert_eq!(
        calls[0].json(),
        json!({"toRecipients": [{"emailAddress": {"address": "x@contoso.com"}}]})
    );
    assert_eq!(calls[1].json()["comment"], "FYI");
    assert_eq!(
        wire(&reply)["data"],
        json!({"forwarded": true, "message_id": "m1", "to": ["x@contoso.com"]})
    );
}

#[tokio::test]
async fn m365_mail_create_draft_posts_to_messages_and_returns_the_draft() {
    let sim = Sim::start(|_, _| Reply::json(201, &json!({"id": "d1", "isDraft": true}))).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let input = parse(
        json!({"subject": "S", "body": "B", "to": ["a@contoso.com"], "cc": ["c@contoso.com"]}),
    );
    let reply = create_draft(&graph, input).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(
        (call.method.as_str(), call.path.as_str()),
        ("POST", "/v1.0/me/messages")
    );
    assert_eq!(
        call.json(),
        json!({
            "subject": "S",
            "body": {"contentType": "Text", "content": "B"},
            "toRecipients": [{"emailAddress": {"address": "a@contoso.com"}}],
            "ccRecipients": [{"emailAddress": {"address": "c@contoso.com"}}]
        })
    );
    assert_eq!(wire(&reply)["data"], json!({"id": "d1", "isDraft": true}));
}

#[tokio::test]
async fn m365_mail_send_draft_posts_an_empty_send_action() {
    let sim = Sim::start(accepted).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let reply = send_draft(&graph, parse(json!({"message_id": "d1"})))
        .await
        .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(
        (call.method.as_str(), call.path.as_str()),
        ("POST", "/v1.0/me/messages/d1/send")
    );
    assert!(call.body.is_empty());
    assert_eq!(
        call.header("content-length"),
        Some("0"),
        "Graph answers 411 without it"
    );
    assert_eq!(
        wire(&reply)["data"],
        json!({"sent": true, "message_id": "d1"})
    );
}

#[tokio::test]
async fn m365_mail_update_mail_patches_only_what_was_given() {
    let sim = Sim::start(|_, _| Reply::json(200, &json!({"id": "m1", "isRead": true}))).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let reply = update_mail(
        &graph,
        parse(json!({"message_id": "m1", "is_read": true, "categories": []})),
    )
    .await
    .unwrap();
    update_mail(
        &graph,
        parse(json!({"message_id": "m1", "importance": "low"})),
    )
    .await
    .unwrap();
    let nothing = update_mail(&graph, parse(json!({"message_id": "m1"})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(
        calls.len(),
        2,
        "an update with nothing to change never reaches Graph"
    );
    assert_eq!(
        (calls[0].method.as_str(), calls[0].path.as_str()),
        ("PATCH", "/v1.0/me/messages/m1")
    );
    assert_eq!(calls[0].json(), json!({"isRead": true, "categories": []}));
    assert_eq!(calls[1].json(), json!({"importance": "low"}));
    assert_eq!(wire(&reply)["data"], json!({"id": "m1", "isRead": true}));
    assert_eq!(
        wire(&nothing)["data"],
        json!({"updated": false, "reason": "No properties to update"})
    );
}

#[tokio::test]
async fn m365_mail_delete_mail_deletes_the_message() {
    let sim = Sim::start(|_, _| Reply::bytes(204, b"")).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let reply = delete_mail(&graph, parse(json!({"message_id": "m1"})))
        .await
        .unwrap();
    let error = delete_mail(&graph, parse(json!({"message_id": ".."})))
        .await
        .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        (calls[0].method.as_str(), calls[0].path.as_str()),
        ("DELETE", "/v1.0/me/messages/m1")
    );
    assert_eq!(
        wire(&reply)["data"],
        json!({"deleted": true, "message_id": "m1"})
    );
    assert!(matches!(error, GraphError::Invalid(_)), "{error:?}");
}

#[tokio::test]
async fn m365_mail_move_mail_sends_the_destination_and_refuses_a_bad_folder() {
    let sim =
        Sim::start(|_, _| Reply::json(201, &json!({"id": "m2", "parentFolderId": "archive"})))
            .await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let reply = move_mail(
        &graph,
        parse(json!({"message_id": "m1", "destination_folder": "archive"})),
    )
    .await
    .unwrap();
    let error = move_mail(
        &graph,
        parse(json!({"message_id": "m1", "destination_folder": "a/b"})),
    )
    .await
    .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].path, "/v1.0/me/messages/m1/move");
    assert_eq!(calls[0].json(), json!({"destinationId": "archive"}));
    assert_eq!(wire(&reply)["data"]["id"], "m2");
    assert!(matches!(error, GraphError::Invalid(_)), "{error:?}");
}
