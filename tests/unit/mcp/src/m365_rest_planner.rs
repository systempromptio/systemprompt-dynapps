//! The Planner plan and bucket tools against a simulated Graph: the Belgian
//! `$select`, a container given as exactly one of group or roster, and every
//! update and delete sent with the caller's etag as `If-Match`.

use serde_json::json;
use systemprompt_mcp_microsoft_365::tools::planner::{
    create_planner_plan, delete_planner_plan, get_planner_plan, list_my_planner_tasks,
    list_planner_plans, update_planner_plan,
};
use systemprompt_mcp_microsoft_365::tools::planner_bucket::{
    create_planner_bucket, delete_planner_bucket, get_planner_bucket, list_planner_buckets,
    update_planner_bucket,
};

use crate::m365_pim_support::{args, assert_domain_registered, empty, graph, is_invalid};
use crate::m365_support::{Reply, Sim, collection};

const PLAN_FIELDS: &str = "id,title,createdDateTime,owner,createdBy,container";
const ETAG: &str = "W/\"JzEtUGxhbiAgQEBAQEBAQEBAQEBAQEBAWCc=\"";

fn listed() -> Reply {
    Reply::json(200, &collection(&[json!({"id": "p-1"})], None))
}

#[test]
fn m365_rest_planner_all_twenty_three_belgian_tools_are_registered_as_pinned() {
    assert_domain_registered("planner", 23);
}

#[tokio::test]
async fn m365_rest_planner_plans_are_listed_from_exactly_one_container() {
    let sim = Sim::start(|_, _| listed()).await;
    let client = graph(&sim);
    let by_group = list_planner_plans(&client, args(json!({"group_id": "g-1", "skip": 5})))
        .await
        .unwrap();
    list_planner_plans(&client, args(json!({"roster_id": "r-1"})))
        .await
        .unwrap();
    let both = list_planner_plans(&client, args(json!({"group_id": "g", "roster_id": "r"})))
        .await
        .unwrap_err();
    let neither = list_planner_plans(&client, args(json!({})))
        .await
        .unwrap_err();
    let bad = list_planner_plans(&client, args(json!({"group_id": "g/1"})))
        .await
        .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 2, "refusals never reach Graph");
    assert_eq!(calls[0].path, "/v1.0/groups/g-1/planner/plans");
    assert_eq!(calls[0].q("$select"), Some(PLAN_FIELDS));
    assert_eq!(calls[0].q("$skip"), Some("5"));
    assert_eq!(calls[1].path, "/v1.0/planner/rosters/r-1/plans");
    assert!(calls[1].q("$skip").is_none());
    assert_eq!(by_group.data, vec![json!({"id": "p-1"})]);
    assert!(is_invalid(&both) && is_invalid(&neither) && is_invalid(&bad));
    assert!(
        both.to_string()
            .contains("exactly one of group_id or roster_id")
    );
}

#[tokio::test]
async fn m365_rest_planner_my_tasks_and_a_plan_are_read_with_their_fields() {
    let sim = Sim::start(|seen, _| match seen.path.as_str() {
        "/v1.0/me/planner/tasks" => listed(),
        _ => Reply::json(200, &json!({"id": "p-1", "@odata.etag": "e"})),
    })
    .await;
    let client = graph(&sim);
    list_my_planner_tasks(&client, args(json!({"skip": 2})))
        .await
        .unwrap();
    let plan = get_planner_plan(&client, args(json!({"plan_id": "p-1"})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert!(
        calls[0]
            .q("$select")
            .unwrap()
            .starts_with("id,title,planId,bucketId,assignments")
    );
    assert_eq!(calls[0].q("$skip"), Some("2"));
    assert_eq!(calls[1].path, "/v1.0/planner/plans/p-1");
    assert_eq!(calls[1].q("$select"), Some(PLAN_FIELDS));
    assert_eq!(plan.data["@odata.etag"], "e");
}

#[tokio::test]
async fn m365_rest_planner_plan_is_created_in_its_container() {
    let sim = Sim::start(|_, _| Reply::json(201, &json!({"id": "p-9"}))).await;
    let client = graph(&sim);
    let created = create_planner_plan(&client, args(json!({"title": "Q1", "group_id": "g-1"})))
        .await
        .unwrap();
    create_planner_plan(&client, args(json!({"title": "Solo", "roster_id": "r-1"})))
        .await
        .unwrap();
    let refused = create_planner_plan(&client, args(json!({"title": "None"})))
        .await
        .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(
        (calls[0].method.as_str(), calls[0].path.as_str()),
        ("POST", "/v1.0/planner/plans")
    );
    assert_eq!(
        calls[0].json(),
        json!({"title": "Q1", "container": {"containerId": "g-1", "type": "group"}})
    );
    assert_eq!(
        calls[1].json()["container"],
        json!({"containerId": "r-1", "type": "roster"})
    );
    assert_eq!(created.data["id"], "p-9");
    assert!(is_invalid(&refused));
}

#[tokio::test]
async fn m365_rest_planner_plan_update_and_delete_carry_the_etag() {
    let sim = Sim::start(|seen, _| match seen.method.as_str() {
        "PATCH" => Reply::json(200, &json!({"id": "p-1", "title": "New"})),
        _ => empty(204),
    })
    .await;
    let client = graph(&sim);
    let updated = update_planner_plan(
        &client,
        args(json!({"plan_id": "p-1", "etag": ETAG, "title": "New"})),
    )
    .await
    .unwrap();
    let deleted = delete_planner_plan(&client, args(json!({"plan_id": "p-1", "etag": ETAG})))
        .await
        .unwrap();
    let nothing = update_planner_plan(&client, args(json!({"plan_id": "p-1", "etag": ETAG})))
        .await
        .unwrap_err();
    let no_tag = delete_planner_plan(&client, args(json!({"plan_id": "p-1", "etag": " "})))
        .await
        .unwrap_err();
    let forged = delete_planner_plan(
        &client,
        args(json!({"plan_id": "p-1", "etag": "a\r\nX: y"})),
    )
    .await
    .unwrap_err();

    let calls = sim.calls();
    assert_eq!(
        calls.len(),
        2,
        "no Graph call without a field or a valid etag"
    );
    assert_eq!(calls[0].method, "PATCH");
    assert_eq!(calls[0].path, "/v1.0/planner/plans/p-1");
    assert_eq!(calls[0].header("if-match"), Some(ETAG));
    assert_eq!(calls[0].header("prefer"), Some("return=representation"));
    assert_eq!(calls[0].json(), json!({"title": "New"}));
    assert_eq!(calls[1].method, "DELETE");
    assert_eq!(calls[1].header("if-match"), Some(ETAG));
    assert_eq!(updated.data["title"], "New");
    assert_eq!(
        serde_json::to_value(&deleted).unwrap(),
        json!({"data": {"deleted": true, "plan_id": "p-1"}, "message": "success"})
    );
    assert!(nothing.to_string().contains("No fields to update"));
    assert!(is_invalid(&no_tag) && is_invalid(&forged));
}

#[tokio::test]
async fn m365_rest_planner_buckets_are_listed_read_created_updated_and_deleted() {
    let sim = Sim::start(|seen, _| match seen.method.as_str() {
        "GET" if seen.path.ends_with("/buckets") => listed(),
        "DELETE" => empty(204),
        _ => Reply::json(200, &json!({"id": "b-1"})),
    })
    .await;
    let client = graph(&sim);
    list_planner_buckets(&client, args(json!({"plan_id": "p-1", "skip": 3})))
        .await
        .unwrap();
    get_planner_bucket(&client, args(json!({"bucket_id": "b-1"})))
        .await
        .unwrap();
    create_planner_bucket(&client, args(json!({"plan_id": "p-1", "name": "To do"})))
        .await
        .unwrap();
    update_planner_bucket(
        &client,
        args(json!({"bucket_id": "b-1", "etag": "e-1", "name": "Doing", "order_hint": "8585"})),
    )
    .await
    .unwrap();
    let deleted = delete_planner_bucket(&client, args(json!({"bucket_id": "b-1", "etag": "e-2"})))
        .await
        .unwrap();
    let bad = get_planner_bucket(&client, args(json!({"bucket_id": "../x"})))
        .await
        .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 5);
    assert_eq!(calls[0].path, "/v1.0/planner/plans/p-1/buckets");
    assert_eq!(calls[0].q("$select"), Some("id,name,planId,orderHint"));
    assert_eq!(calls[0].q("$skip"), Some("3"));
    assert_eq!(calls[1].path, "/v1.0/planner/buckets/b-1");
    assert_eq!(calls[1].q("$select"), Some("id,name,planId,orderHint"));
    assert_eq!(calls[2].path, "/v1.0/planner/buckets");
    assert_eq!(
        calls[2].json(),
        json!({"name": "To do", "planId": "p-1", "orderHint": " !"})
    );
    assert_eq!(calls[3].method, "PATCH");
    assert_eq!(calls[3].header("if-match"), Some("e-1"));
    assert_eq!(
        calls[3].json(),
        json!({"name": "Doing", "orderHint": "8585"})
    );
    assert_eq!(
        (calls[4].method.as_str(), calls[4].header("if-match")),
        ("DELETE", Some("e-2"))
    );
    assert_eq!(deleted.data, json!({"deleted": true, "bucket_id": "b-1"}));
    assert!(is_invalid(&bad));
}
