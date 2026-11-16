//! The Teams chat tools against a simulated Graph: chats listed with their
//! members trimmed, messages and members with the Belgian `$select`, new
//! one-on-one and group chats that bind the caller from `/me`, pins bound
//! under the configured Graph root, and the null-data replies of unpin and
//! reactions.

use serde_json::{Value, json};
use systemprompt_mcp_microsoft_365::tools::chat::{
    get_chat, list_chat_members, list_chat_messages, list_chats,
};
use systemprompt_mcp_microsoft_365::tools::chat_write::{
    create_group_chat, create_one_on_one_chat, pin_chat_message, send_chat_message,
    set_chat_message_reaction, unpin_chat_message,
};

use crate::m365_pim_support::{args, assert_domain_registered, empty, graph, is_invalid};
use crate::m365_support::{Reply, Sim, collection};

const ME: &str = "me-0001";

async fn echo() -> Sim {
    Sim::start(|seen, _| match (seen.method.as_str(), seen.path.as_str()) {
        ("GET", "/v1.0/me") => Reply::json(200, &json!({"id": ME})),
        ("DELETE", _) => empty(204),
        ("POST", path) if path.ends_with("/setReaction") => empty(204),
        ("POST", _) => Reply::json(201, &json!({"id": "chat-new"})),
        ("GET", path) if path.ends_with("/c1") => Reply::json(200, &json!({"id": "c1"})),
        _ => Reply::json(200, &collection(&[json!({"id": "x"})], None)),
    })
    .await
}

fn bind(sim: &Sim, user: &str) -> Value {
    json!({
        "@odata.type": "#microsoft.graph.aadUserConversationMember",
        "roles": ["owner"],
        "user@odata.bind": format!("{}/users('{user}')", sim.graph_url()),
    })
}

#[test]
fn m365_teams_chat_all_ten_belgian_tools_are_registered_as_pinned() {
    assert_domain_registered("chat", 10);
}

#[tokio::test]
async fn m365_teams_chat_list_expands_members_and_trims_them() {
    let chat = json!({
        "id": "c1", "topic": "Ops",
        "members": [{
            "@odata.type": "#microsoft.graph.aadUserConversationMember",
            "id": "mm1", "displayName": "Ana", "roles": ["owner"], "email": "ana@x.test",
            "userId": "u1", "tenantId": "t1", "visibleHistoryStartDateTime": "0001-01-01"
        }]
    });
    let sim = Sim::start(move |_, _| Reply::json(200, &collection(&[chat.clone()], None))).await;
    let reply = list_chats(&graph(&sim), args(json!({"top": 0, "skip": 5})))
        .await
        .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.path, "/v1.0/me/chats");
    assert_eq!(
        call.q("$select"),
        Some("id,topic,chatType,createdDateTime,lastUpdatedDateTime")
    );
    assert_eq!(call.q("$expand"), Some("members"));
    assert_eq!((call.q("$top"), call.q("$skip")), (Some("1"), Some("5")));
    assert_eq!(
        reply.data,
        vec![json!({
            "id": "c1", "topic": "Ops",
            "members": [{"id": "mm1", "displayName": "Ana", "roles": ["owner"], "email": "ana@x.test"}]
        })]
    );
}

#[tokio::test]
async fn m365_teams_chat_get_messages_and_members_read_as_belgium() {
    let sim = echo().await;
    let client = graph(&sim);
    let chat = get_chat(&client, args(json!({"chat_id": "c1"})))
        .await
        .unwrap();
    list_chat_messages(&client, args(json!({"chat_id": "19:a_b@unq.gbl.spaces"})))
        .await
        .unwrap();
    list_chat_members(&client, args(json!({"chat_id": "C2", "skip": 1})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/chats/c1");
    assert_eq!(chat.data["id"], "c1");
    assert_eq!(
        calls[1].path,
        "/v1.0/me/chats/19:a_b@unq.gbl.spaces/messages"
    );
    assert_eq!(
        calls[1].q("$select"),
        Some("id,body,from,createdDateTime,importance,messageType")
    );
    assert_eq!(
        (calls[1].q("$top"), calls[1].q("$skip")),
        (Some("25"), None)
    );
    assert_eq!(calls[2].path, "/v1.0/me/chats/C2/members");
    assert_eq!(
        calls[2].q("$select"),
        Some("id,displayName,roles,microsoft.graph.aadUserConversationMember/email")
    );
    assert_eq!(calls[2].q("$skip"), Some("1"));
}

#[tokio::test]
async fn m365_teams_chat_send_posts_text_or_html() {
    let sim = echo().await;
    let client = graph(&sim);
    send_chat_message(&client, args(json!({"chat_id": "C1", "body": "hi"})))
        .await
        .unwrap();
    let sent = send_chat_message(
        &client,
        args(json!({"chat_id": "C1", "body": "<p>hi</p>", "is_html": true})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].method, "POST");
    assert_eq!(calls[0].path, "/v1.0/me/chats/C1/messages");
    assert_eq!(
        calls[0].json(),
        json!({"body": {"content": "hi", "contentType": "text"}})
    );
    assert_eq!(calls[1].json()["body"]["contentType"], "html");
    assert_eq!(sent.data["id"], "chat-new");
}

#[tokio::test]
async fn m365_teams_chat_one_on_one_binds_the_caller_and_the_other_user() {
    let sim = echo().await;
    let reply = create_one_on_one_chat(&graph(&sim), args(json!({"user_id": "u-2"})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me");
    assert_eq!(calls[0].q("$select"), Some("id"));
    assert_eq!(calls[1].method, "POST");
    assert_eq!(calls[1].path, "/v1.0/chats");
    assert_eq!(
        calls[1].json(),
        json!({"chatType": "oneOnOne", "members": [bind(&sim, ME), bind(&sim, "u-2")]})
    );
    assert_eq!(reply.data["id"], "chat-new");
}

#[tokio::test]
async fn m365_teams_chat_group_adds_the_caller_once_and_the_topic_if_given() {
    let sim = echo().await;
    let client = graph(&sim);
    create_group_chat(
        &client,
        args(json!({"member_ids": ["u-2", ME, "u-3"], "topic": "Launch"})),
    )
    .await
    .unwrap();
    create_group_chat(&client, args(json!({"member_ids": ["u-4"]})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(
        calls[1].json(),
        json!({
            "chatType": "group", "topic": "Launch",
            "members": [bind(&sim, ME), bind(&sim, "u-2"), bind(&sim, "u-3")]
        })
    );
    assert_eq!(
        calls[3].json(),
        json!({"chatType": "group", "members": [bind(&sim, ME), bind(&sim, "u-4")]})
    );
}

#[tokio::test]
async fn m365_teams_chat_without_a_caller_id_answers_like_belgium_and_posts_nothing() {
    let sim = Sim::start(|_, _| Reply::json(200, &json!({"displayName": "No id"}))).await;
    let client = graph(&sim);
    let one = create_one_on_one_chat(&client, args(json!({"user_id": "u-2"})))
        .await
        .unwrap();
    let group = create_group_chat(&client, args(json!({"member_ids": ["u-2"]})))
        .await
        .unwrap();

    let expected = json!({"data": null, "message": "Could not determine caller id from /me"});
    assert_eq!(serde_json::to_value(&one).unwrap(), expected);
    assert_eq!(serde_json::to_value(&group).unwrap(), expected);
    assert!(sim.calls().iter().all(|c| c.method == "GET"));
}

#[tokio::test]
async fn m365_teams_chat_pin_unpin_and_react() {
    let sim = echo().await;
    let client = graph(&sim);
    let pinned = pin_chat_message(&client, args(json!({"chat_id": "C1", "message_id": "M1"})))
        .await
        .unwrap();
    let unpinned = unpin_chat_message(
        &client,
        args(json!({"chat_id": "C1", "pinned_message_id": "P1"})),
    )
    .await
    .unwrap();
    let reacted = set_chat_message_reaction(
        &client,
        args(json!({"chat_id": "C1", "message_id": "M1", "reaction_type": "heart"})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/chats/C1/pinnedMessages");
    assert_eq!(
        calls[0].json(),
        json!({"message@odata.bind": format!("{}/chats/C1/messages/M1", sim.graph_url())})
    );
    assert_eq!(pinned.data["id"], "chat-new");
    assert_eq!(calls[1].method, "DELETE");
    assert_eq!(calls[1].path, "/v1.0/me/chats/C1/pinnedMessages/P1");
    assert_eq!(
        serde_json::to_value(&unpinned).unwrap(),
        json!({"data": null, "message": "Message unpinned"})
    );
    assert_eq!(calls[2].path, "/v1.0/me/chats/C1/messages/M1/setReaction");
    assert_eq!(calls[2].json(), json!({"reactionType": "heart"}));
    assert_eq!(
        serde_json::to_value(&reacted).unwrap(),
        json!({"data": null, "message": "Reaction 'heart' set"})
    );
}

#[tokio::test]
async fn m365_teams_chat_ids_are_checked_before_graph() {
    let sim = echo().await;
    let client = graph(&sim);
    let errors = [
        get_chat(&client, args(json!({"chat_id": "../me"})))
            .await
            .unwrap_err(),
        create_one_on_one_chat(&client, args(json!({"user_id": "u')/x"})))
            .await
            .unwrap_err(),
        pin_chat_message(&client, args(json!({"chat_id": "C1", "message_id": ""})))
            .await
            .unwrap_err(),
        unpin_chat_message(
            &client,
            args(json!({"chat_id": "C1", "pinned_message_id": "a\\b"})),
        )
        .await
        .unwrap_err(),
        set_chat_message_reaction(
            &client,
            args(json!({"chat_id": "C 1", "message_id": "M1", "reaction_type": "like"})),
        )
        .await
        .unwrap_err(),
    ];
    assert!(errors.iter().all(is_invalid), "{errors:?}");
    assert!(sim.calls().is_empty());

    let refused = create_group_chat(&client, args(json!({"member_ids": ["u-2", "bad'id"]})))
        .await
        .unwrap_err();
    assert!(is_invalid(&refused));
    assert!(sim.calls().iter().all(|c| c.method == "GET"));
}
