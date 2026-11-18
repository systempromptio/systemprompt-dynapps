//! The group and user tools against a simulated Graph: the Belgian
//! `$select`, display-name searches as advanced queries
//! (`ConsistencyLevel: eventual`), and the signed-in user's own profile,
//! manager and direct reports.

use serde_json::json;
use systemprompt_mcp_microsoft_365::tools::groups::{
    get_group, list_group_conversations, list_group_members, list_group_owners, list_group_threads,
    list_groups,
};
use systemprompt_mcp_microsoft_365::tools::users::{
    get_my_manager, get_my_profile, get_user, list_direct_reports, search_users,
};

use crate::m365_pim_support::{args, assert_domain_registered, graph, is_invalid};
use crate::m365_support::{Reply, Sim, collection};

const USER_FIELDS: &str =
    "id,displayName,mail,userPrincipalName,jobTitle,department,officeLocation";
const DETAIL_FIELDS: &str = "id,displayName,mail,userPrincipalName,jobTitle,department,\
    officeLocation,businessPhones,mobilePhone,companyName";

fn listed() -> Reply {
    Reply::json(200, &collection(&[json!({"id": "x"})], None))
}

#[test]
fn m365_rest_directory_groups_and_users_are_registered_as_pinned() {
    assert_domain_registered("groups", 6);
    assert_domain_registered("users", 5);
}

#[tokio::test]
async fn m365_rest_directory_groups_are_listed_or_searched() {
    let sim = Sim::start(|_, _| listed()).await;
    let client = graph(&sim);
    list_groups(&client, args(json!({}))).await.unwrap();
    list_groups(
        &client,
        args(json!({"search": "Sales \"EU\"", "top": 5, "skip": 5})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/groups");
    assert_eq!(
        calls[0].q("$select"),
        Some("id,displayName,description,mail,groupTypes,visibility")
    );
    assert_eq!(calls[0].q("$top"), Some("25"));
    assert!(calls[0].q("$search").is_none());
    assert!(calls[0].header("consistencylevel").is_none());
    assert_eq!(calls[1].q("$search"), Some(r#""displayName:Sales \"EU\"""#));
    assert_eq!(calls[1].header("consistencylevel"), Some("eventual"));
    assert_eq!(
        (calls[1].q("$top"), calls[1].q("$skip")),
        (Some("5"), Some("5"))
    );
}

#[tokio::test]
async fn m365_rest_directory_a_group_and_its_people_and_conversations_are_read() {
    let sim = Sim::start(|seen, _| match seen.path.as_str() {
        "/v1.0/groups/g-1" => Reply::json(200, &json!({"id": "g-1"})),
        _ => listed(),
    })
    .await;
    let client = graph(&sim);
    let group = get_group(&client, args(json!({"group_id": "g-1"})))
        .await
        .unwrap();
    list_group_members(&client, args(json!({"group_id": "g-1", "skip": 1})))
        .await
        .unwrap();
    list_group_owners(&client, args(json!({"group_id": "g-1"})))
        .await
        .unwrap();
    list_group_conversations(&client, args(json!({"group_id": "g-1", "top": 10})))
        .await
        .unwrap();
    list_group_threads(
        &client,
        args(json!({"group_id": "g-1", "conversation_id": "c=1"})),
    )
    .await
    .unwrap();
    let bad = list_group_threads(
        &client,
        args(json!({"group_id": "g-1", "conversation_id": ""})),
    )
    .await
    .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 5);
    assert_eq!(group.data["id"], "g-1");
    assert!(calls[0].raw_query.is_none());
    assert_eq!(calls[1].path, "/v1.0/groups/g-1/members");
    assert_eq!(
        (calls[1].q("$select"), calls[1].q("$skip")),
        (Some(USER_FIELDS), Some("1"))
    );
    assert_eq!(calls[2].path, "/v1.0/groups/g-1/owners");
    assert_eq!(calls[2].q("$select"), Some(USER_FIELDS));
    assert_eq!(calls[3].path, "/v1.0/groups/g-1/conversations");
    assert_eq!(calls[3].q("$top"), Some("10"));
    assert_eq!(calls[4].path, "/v1.0/groups/g-1/conversations/c=1/threads");
    assert_eq!(calls[4].q("$top"), Some("25"));
    assert!(is_invalid(&bad));
}

#[tokio::test]
async fn m365_rest_directory_users_are_searched_and_read() {
    let sim = Sim::start(|seen, _| match seen.path.as_str() {
        "/v1.0/users" | "/v1.0/me/directReports" => listed(),
        _ => Reply::json(200, &json!({"id": "u"})),
    })
    .await;
    let client = graph(&sim);
    search_users(&client, args(json!({"query": "Ana"})))
        .await
        .unwrap();
    get_my_profile(&client, args(json!({}))).await.unwrap();
    let user = get_user(&client, args(json!({"user_id": "ana@contoso.com"})))
        .await
        .unwrap();
    get_my_manager(&client, args(json!({}))).await.unwrap();
    let bad = get_user(&client, args(json!({"user_id": "a/../b"})))
        .await
        .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 4);
    assert_eq!(calls[0].path, "/v1.0/users");
    assert_eq!(calls[0].q("$search"), Some("\"displayName:Ana\""));
    assert_eq!(calls[0].q("$select"), Some(USER_FIELDS));
    assert_eq!(calls[0].q("$top"), Some("10"));
    assert_eq!(calls[0].header("consistencylevel"), Some("eventual"));
    assert_eq!(calls[1].path, "/v1.0/me");
    assert_eq!(calls[1].q("$select"), Some(DETAIL_FIELDS));
    assert_eq!(calls[2].path, "/v1.0/users/ana@contoso.com");
    assert_eq!(calls[2].q("$select"), Some(DETAIL_FIELDS));
    assert_eq!(user.data["id"], "u");
    assert_eq!(calls[3].path, "/v1.0/me/manager");
    assert_eq!(calls[3].q("$select"), Some(USER_FIELDS));
    assert!(is_invalid(&bad));
}

#[tokio::test]
async fn m365_rest_directory_direct_reports_are_the_callers_or_a_users() {
    let sim = Sim::start(|_, _| listed()).await;
    let client = graph(&sim);
    list_direct_reports(&client, args(json!({}))).await.unwrap();
    list_direct_reports(&client, args(json!({"user_id": "u-2", "skip": 3})))
        .await
        .unwrap();
    list_direct_reports(&client, args(json!({"user_id": ""})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/directReports");
    assert_eq!(calls[0].q("$select"), Some(USER_FIELDS));
    assert_eq!(calls[1].path, "/v1.0/users/u-2/directReports");
    assert_eq!(calls[1].q("$skip"), Some("3"));
    assert_eq!(calls[2].path, "/v1.0/me/directReports");
}
