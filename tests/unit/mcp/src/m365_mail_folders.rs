//! Mail folders, inbox rules and Outlook categories against a simulated
//! Graph: where each is created, the rule's default sequence, and colours
//! or sequences Graph would refuse caught before any call.

use serde_json::{Value, json};
use systemprompt_mcp_microsoft_365::graph::GraphError;
use systemprompt_mcp_microsoft_365::tools::mail_folders::{
    create_mail_folder, create_mail_rule, create_outlook_category, delete_mail_folder,
};

use crate::m365_support::{FakeTokens, Reply, Sim, client, collection, fast_limits};

fn parse<T: serde::de::DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

fn echo_created() -> impl Fn(&crate::m365_support::Seen, usize) -> Reply {
    |seen, _| match seen.method.as_str() {
        "GET" => Reply::json(
            200,
            &collection(
                &[
                    json!({"id": "r1", "sequence": 2}),
                    json!({"id": "r2", "sequence": 7}),
                    json!({"id": "r3"}),
                ],
                None,
            ),
        ),
        "DELETE" => Reply::bytes(204, b""),
        _ => Reply::json(201, &json!({"id": "new"})),
    }
}

#[tokio::test]
async fn m365_mail_create_mail_folder_at_the_top_or_under_a_parent() {
    let sim = Sim::start(echo_created()).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let top = create_mail_folder(&graph, parse(json!({"display_name": "Clients"})))
        .await
        .unwrap();
    create_mail_folder(
        &graph,
        parse(json!({"display_name": "Acme", "parent_folder_id": "AAMkF="})),
    )
    .await
    .unwrap();
    let error = create_mail_folder(
        &graph,
        parse(json!({"display_name": "x", "parent_folder_id": "a/../b"})),
    )
    .await
    .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].path, "/v1.0/me/mailFolders");
    assert_eq!(calls[0].json(), json!({"displayName": "Clients"}));
    assert_eq!(calls[1].path, "/v1.0/me/mailFolders/AAMkF=/childFolders");
    assert_eq!(serde_json::to_value(&top).unwrap()["data"]["id"], "new");
    assert!(matches!(error, GraphError::Invalid(_)), "{error:?}");
}

#[tokio::test]
async fn m365_mail_delete_mail_folder_deletes_by_id() {
    let sim = Sim::start(echo_created()).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let reply = delete_mail_folder(&graph, parse(json!({"folder_id": "AAMkF="})))
        .await
        .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(
        (call.method.as_str(), call.path.as_str()),
        ("DELETE", "/v1.0/me/mailFolders/AAMkF=")
    );
    assert_eq!(
        serde_json::to_value(&reply).unwrap()["data"],
        json!({"deleted": true, "folder_id": "AAMkF="})
    );
}

#[tokio::test]
async fn m365_mail_create_mail_rule_runs_last_unless_a_sequence_is_given() {
    let sim = Sim::start(echo_created()).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let rule = json!({
        "display_name": "Invoices",
        "conditions": {"subjectContains": ["invoice"]},
        "actions": {"moveToFolder": "AAMkF=", "markAsRead": true}
    });
    create_mail_rule(&graph, parse(rule.clone())).await.unwrap();

    let calls = sim.calls();
    assert_eq!(
        (calls[0].method.as_str(), calls[0].path.as_str()),
        ("GET", "/v1.0/me/mailFolders/inbox/messageRules")
    );
    assert_eq!(calls[1].method, "POST");
    assert_eq!(
        calls[1].json(),
        json!({
            "displayName": "Invoices",
            "sequence": 8,
            "conditions": {"subjectContains": ["invoice"]},
            "actions": {"moveToFolder": "AAMkF=", "markAsRead": true},
            "isEnabled": true
        })
    );

    let mut explicit = rule.clone();
    explicit["sequence"] = json!(3);
    explicit["is_enabled"] = json!(false);
    create_mail_rule(&graph, parse(explicit)).await.unwrap();
    let calls = sim.calls();
    assert_eq!(calls.len(), 3, "an explicit sequence needs no lookup");
    assert_eq!(calls[2].json()["sequence"], 3);
    assert_eq!(calls[2].json()["isEnabled"], false);

    let mut zero = rule;
    zero["sequence"] = json!(0);
    let error = create_mail_rule(&graph, parse(zero)).await.unwrap_err();
    assert!(matches!(error, GraphError::Invalid(_)), "{error:?}");
    assert_eq!(sim.calls().len(), 3);
}

#[tokio::test]
async fn m365_mail_create_outlook_category_admits_only_graph_colour_presets() {
    let sim = Sim::start(echo_created()).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    for color in ["none", "preset0", "preset9", "preset19", "preset24"] {
        create_outlook_category(
            &graph,
            parse(json!({"display_name": "Projects", "color": color})),
        )
        .await
        .unwrap();
    }
    create_outlook_category(&graph, parse(json!({"display_name": "Default"})))
        .await
        .unwrap();
    for color in [
        "preset25", "preset01", "preset+5", "preset", "red", "Preset1",
    ] {
        let error =
            create_outlook_category(&graph, parse(json!({"display_name": "X", "color": color})))
                .await
                .unwrap_err();
        assert!(
            matches!(error, GraphError::Invalid(_)),
            "{color}: {error:?}"
        );
    }

    let calls = sim.calls();
    assert_eq!(calls.len(), 6);
    assert_eq!(calls[0].path, "/v1.0/me/outlook/masterCategories");
    assert_eq!(
        calls[0].json(),
        json!({"displayName": "Projects", "color": "none"})
    );
    assert_eq!(
        calls[5].json(),
        json!({"displayName": "Default", "color": "preset0"})
    );
}
