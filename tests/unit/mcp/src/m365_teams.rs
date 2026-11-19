//! The Teams and channel tools against a simulated Graph: joined teams read
//! without paging, channel and member lists with their Belgian `$select`,
//! messages paged with `top` and `skip`, message posts as text or HTML,
//! channel create and partial update, and ids refused before Graph.

use serde_json::json;
use systemprompt_mcp_microsoft_365::tools::teams::{
    get_channel, get_channel_message, list_channel_members, list_channel_message_replies,
    list_channel_messages, list_channels, list_team_members, list_teams,
};
use systemprompt_mcp_microsoft_365::tools::teams_write::{
    create_channel, reply_to_channel_message, send_channel_message, update_channel,
};

use crate::m365_pim_support::{args, assert_domain_registered, empty, graph, is_invalid};
use crate::m365_support::{Reply, Sim, collection};

const MEMBERS: &str = "id,displayName,roles,microsoft.graph.aadUserConversationMember/email";

async fn echo() -> Sim {
    Sim::start(|seen, _| match seen.method.as_str() {
        "PATCH" => empty(204),
        "POST" => Reply::json(201, &json!({"id": "new1"})),
        _ if seen.path.ends_with("/c1") || seen.path.ends_with("/m1") => {
            Reply::json(200, &json!({"id": "one"}))
        },
        _ => Reply::json(200, &collection(&[json!({"id": "x"})], None)),
    })
    .await
}

#[test]
fn m365_teams_all_twelve_belgian_tools_are_registered_as_pinned() {
    assert_domain_registered("teams", 12);
}

#[tokio::test]
async fn m365_teams_joined_teams_are_read_with_their_fields_and_no_paging() {
    let sim = Sim::start(|_, _| {
        Reply::json(
            200,
            &collection(&[json!({"id": "t1"})], Some("http://ignored/next")),
        )
    })
    .await;
    let reply = list_teams(&graph(&sim), args(json!({}))).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.path, "/v1.0/me/joinedTeams");
    assert_eq!(call.q("$select"), Some("id,displayName,description"));
    assert_eq!((call.q("$top"), call.q("$skip")), (None, None));
    assert_eq!(reply.data, vec![json!({"id": "t1"})]);
    assert_eq!(reply.has_more, Some(true));
}

#[tokio::test]
async fn m365_teams_channels_and_team_members_page_with_skip_and_select() {
    let sim = echo().await;
    let client = graph(&sim);
    list_channels(&client, args(json!({"team_id": "T1", "skip": 4})))
        .await
        .unwrap();
    list_team_members(&client, args(json!({"team_id": "T1", "skip": -2})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/teams/T1/channels");
    assert_eq!(
        calls[0].q("$select"),
        Some("id,displayName,description,membershipType")
    );
    assert_eq!((calls[0].q("$skip"), calls[0].q("$top")), (Some("4"), None));
    assert_eq!(calls[1].path, "/v1.0/teams/T1/members");
    assert_eq!(calls[1].q("$select"), Some(MEMBERS));
    assert_eq!(calls[1].q("$skip"), None);
}

#[tokio::test]
async fn m365_teams_channel_and_message_are_read_by_their_ids() {
    let sim = echo().await;
    let client = graph(&sim);
    let channel = get_channel(&client, args(json!({"team_id": "T1", "channel_id": "c1"})))
        .await
        .unwrap();
    let message = get_channel_message(
        &client,
        args(json!({"team_id": "T1", "channel_id": "19:a@thread.tacv2", "message_id": "m1"})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/teams/T1/channels/c1");
    assert_eq!(
        calls[1].path,
        "/v1.0/teams/T1/channels/19:a@thread.tacv2/messages/m1"
    );
    assert!(calls.iter().all(|c| c.q("$select").is_none()));
    assert_eq!(channel.data["id"], "one");
    assert_eq!(message.message, "success");
}

#[tokio::test]
async fn m365_teams_messages_replies_and_channel_members_page_as_belgium() {
    let sim = echo().await;
    let client = graph(&sim);
    let ids = json!({"team_id": "T1", "channel_id": "C1"});
    let messages = list_channel_messages(
        &client,
        args(json!({"team_id": "T1", "channel_id": "C1", "top": 99})),
    )
    .await
    .unwrap();
    list_channel_message_replies(
        &client,
        args(json!({"team_id": "T1", "channel_id": "C1", "message_id": "P1", "skip": 2})),
    )
    .await
    .unwrap();
    list_channel_members(&client, args(ids)).await.unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/teams/T1/channels/C1/messages");
    assert_eq!(
        (calls[0].q("$top"), calls[0].q("$skip")),
        (Some("50"), None)
    );
    assert!(calls[0].q("$select").is_none());
    assert_eq!(
        calls[1].path,
        "/v1.0/teams/T1/channels/C1/messages/P1/replies"
    );
    assert_eq!((calls[1].q("$skip"), calls[1].q("$top")), (Some("2"), None));
    assert_eq!(calls[2].path, "/v1.0/teams/T1/channels/C1/members");
    assert_eq!(calls[2].q("$select"), Some(MEMBERS));
    assert_eq!(messages.data.len(), 1);
}

#[tokio::test]
async fn m365_teams_messages_post_as_text_or_html() {
    let sim = echo().await;
    let client = graph(&sim);
    let sent = send_channel_message(
        &client,
        args(json!({"team_id": "T1", "channel_id": "C1", "body": "Hola"})),
    )
    .await
    .unwrap();
    reply_to_channel_message(
        &client,
        args(json!({
            "team_id": "T1", "channel_id": "C1", "message_id": "P1",
            "body": "<b>ok</b>", "is_html": true
        })),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].method, "POST");
    assert_eq!(calls[0].path, "/v1.0/teams/T1/channels/C1/messages");
    assert_eq!(
        calls[0].json(),
        json!({"body": {"content": "Hola", "contentType": "text"}})
    );
    assert_eq!(
        calls[1].path,
        "/v1.0/teams/T1/channels/C1/messages/P1/replies"
    );
    assert_eq!(
        calls[1].json(),
        json!({"body": {"content": "<b>ok</b>", "contentType": "html"}})
    );
    assert_eq!(sent.data["id"], "new1");
}

#[tokio::test]
async fn m365_teams_create_channel_defaults_to_standard_and_sends_description_only_if_given() {
    let sim = echo().await;
    let client = graph(&sim);
    create_channel(
        &client,
        args(json!({"team_id": "T1", "display_name": "Ops"})),
    )
    .await
    .unwrap();
    create_channel(
        &client,
        args(json!({
            "team_id": "T1", "display_name": "Board", "description": "Private",
            "membership_type": "private"
        })),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/teams/T1/channels");
    assert_eq!(
        calls[0].json(),
        json!({"displayName": "Ops", "membershipType": "standard"})
    );
    assert_eq!(
        calls[1].json(),
        json!({"displayName": "Board", "membershipType": "private", "description": "Private"})
    );
}

#[tokio::test]
async fn m365_teams_update_channel_patches_only_what_was_given() {
    let sim = echo().await;
    let client = graph(&sim);
    let reply = update_channel(
        &client,
        args(json!({"team_id": "T1", "channel_id": "C1", "description": ""})),
    )
    .await
    .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.method, "PATCH");
    assert_eq!(call.path, "/v1.0/teams/T1/channels/C1");
    assert_eq!(call.json(), json!({"description": ""}));
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": null, "message": "success"})
    );
}

#[tokio::test]
async fn m365_teams_ids_are_checked_before_graph() {
    let sim = echo().await;
    let client = graph(&sim);
    let errors = [
        list_channels(&client, args(json!({"team_id": "../x"})))
            .await
            .unwrap_err(),
        list_channel_messages(&client, args(json!({"team_id": "T1", "channel_id": "a/b"})))
            .await
            .unwrap_err(),
        send_channel_message(
            &client,
            args(json!({"team_id": "T1", "channel_id": "C1'", "body": "x"})),
        )
        .await
        .unwrap_err(),
        create_channel(&client, args(json!({"team_id": "", "display_name": "x"})))
            .await
            .unwrap_err(),
        update_channel(&client, args(json!({"team_id": "T1", "channel_id": "C 1"})))
            .await
            .unwrap_err(),
    ];

    assert!(errors.iter().all(is_invalid), "{errors:?}");
    assert!(sim.calls().is_empty());
}
