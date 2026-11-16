//! The Teams presence tools against a simulated Graph: the user's own
//! presence with the Belgian `$select`, another user's by id, and setting
//! the user's presence as a session of the Entra app, whose client id the
//! endpoint reads from the connector's `microsoft_365_client_id` secret.

use serde_json::json;
use systemprompt::traits::SecretLookup;
use systemprompt_mcp_microsoft_365::graph::config::APP_ID_SECRET;
use systemprompt_mcp_microsoft_365::graph::{
    GraphClient, GraphClientParts, GraphEndpoint, GraphError, http_client,
};
use systemprompt_mcp_microsoft_365::tools::presence::{
    get_my_presence, get_user_presence, set_my_presence,
};

use crate::m365_pim_support::{args, assert_domain_registered, empty, graph, is_invalid};
use crate::m365_support::{FakeTokens, Reply, Secrets, Sim, fast_limits};

fn graph_as_app(sim: &Sim, app_id: &str) -> GraphClient {
    let limits = fast_limits();
    GraphClient::new(GraphClientParts {
        endpoint: GraphEndpoint::parse(&sim.graph_url())
            .unwrap()
            .with_app_id(app_id),
        http: http_client(&limits).unwrap(),
        tokens: FakeTokens::new(&["graph-token"]),
        limits,
    })
}

#[test]
fn m365_teams_presence_all_three_belgian_tools_are_registered_as_pinned() {
    assert_domain_registered("presence", 3);
}

#[tokio::test]
async fn m365_teams_presence_own_and_other_users_are_read() {
    let sim = Sim::start(|_, _| Reply::json(200, &json!({"availability": "Busy"}))).await;
    let client = graph(&sim);
    let mine = get_my_presence(&client, args(json!({}))).await.unwrap();
    let theirs = get_user_presence(&client, args(json!({"user_id": "u-2"})))
        .await
        .unwrap();
    let refused = get_user_presence(&client, args(json!({"user_id": "u/2"})))
        .await
        .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].path, "/v1.0/me/presence");
    assert_eq!(
        calls[0].q("$select"),
        Some("id,availability,activity,statusMessage")
    );
    assert_eq!(calls[1].path, "/v1.0/users/u-2/presence");
    assert!(calls[1].q("$select").is_none());
    assert_eq!(mine.data["availability"], "Busy");
    assert_eq!(theirs.message, "success");
    assert!(is_invalid(&refused));
}

#[tokio::test]
async fn m365_teams_presence_is_set_as_a_session_of_the_entra_app() {
    let sim = Sim::start(|_, _| empty(200)).await;
    let reply = set_my_presence(
        &graph_as_app(&sim, "app-1234"),
        args(json!({"availability": "Busy", "activity": "InACall"})),
    )
    .await
    .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.method, "POST");
    assert_eq!(call.path, "/v1.0/me/presence/setPresence");
    assert_eq!(
        call.json(),
        json!({
            "sessionId": "app-1234", "availability": "Busy", "activity": "InACall",
            "expirationDuration": "PT1H"
        })
    );
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": null, "message": "Presence set to Busy/InACall"})
    );
}

#[tokio::test]
async fn m365_teams_presence_is_not_set_without_the_app_id() {
    let sim = Sim::start(|_, _| empty(200)).await;
    let blank = graph_as_app(&sim, "  ");
    let input =
        || args(json!({"availability": "Away", "activity": "Away", "expiration_duration": "PT5M"}));
    let unset = set_my_presence(&graph(&sim), input()).await.unwrap_err();
    let empty_id = set_my_presence(&blank, input()).await.unwrap_err();

    assert!(
        matches!(unset, GraphError::Config(ref m) if m.contains(APP_ID_SECRET)),
        "{unset}"
    );
    assert!(matches!(empty_id, GraphError::Config(_)), "{empty_id}");
    assert!(sim.calls().is_empty());
}

#[test]
fn m365_teams_presence_endpoint_reads_the_app_id_from_the_connector_secret() {
    let set = Secrets::with(&[(APP_ID_SECRET, " app-5678 ")]);
    let lookup: &dyn SecretLookup = set.as_ref();
    let endpoint = GraphEndpoint::from_lookup(lookup).unwrap();
    assert_eq!(endpoint.app_id(), Some("app-5678"));
    assert_eq!(APP_ID_SECRET, "microsoft_365_client_id");

    let none = Secrets::with(&[]);
    assert_eq!(
        GraphEndpoint::from_lookup(none.as_ref()).unwrap().app_id(),
        None
    );
}
