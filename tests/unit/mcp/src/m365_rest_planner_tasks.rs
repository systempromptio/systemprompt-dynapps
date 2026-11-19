//! The Planner task, task details and roster tools against a simulated
//! Graph: assignments in Planner's keyed format (null unassigns), only the
//! given fields patched, the details' own etag, and rosters with their
//! members.

use serde_json::json;
use systemprompt_mcp_microsoft_365::tools::planner_details::{
    get_planner_task_details, update_planner_task_details,
};
use systemprompt_mcp_microsoft_365::tools::planner_roster::{
    add_planner_roster_member, create_planner_roster, delete_planner_roster,
    delete_planner_roster_member, list_planner_roster_members,
};
use systemprompt_mcp_microsoft_365::tools::planner_task::{
    create_planner_task, delete_planner_task, get_planner_task, list_planner_tasks,
    update_planner_task,
};

use crate::m365_pim_support::{args, empty, graph, is_invalid};
use crate::m365_support::{Reply, Sim, collection};

fn assigned() -> serde_json::Value {
    json!({"@odata.type": "#microsoft.planner.assignment", "orderHint": " !"})
}

#[tokio::test]
async fn m365_rest_planner_tasks_are_listed_from_a_plan_or_a_bucket() {
    let sim = Sim::start(|_, _| Reply::json(200, &collection(&[json!({"id": "t"})], None))).await;
    let client = graph(&sim);
    list_planner_tasks(
        &client,
        args(json!({"plan_id": "p-1", "top": 99, "skip": 4})),
    )
    .await
    .unwrap();
    list_planner_tasks(&client, args(json!({"bucket_id": "b-1"})))
        .await
        .unwrap();
    let both = list_planner_tasks(&client, args(json!({"plan_id": "p", "bucket_id": "b"})))
        .await
        .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].path, "/v1.0/planner/plans/p-1/tasks");
    assert_eq!(
        (calls[0].q("$top"), calls[0].q("$skip")),
        (Some("50"), Some("4"))
    );
    assert!(
        calls[0]
            .q("$select")
            .unwrap()
            .ends_with("completedDateTime,orderHint")
    );
    assert_eq!(calls[1].path, "/v1.0/planner/buckets/b-1/tasks");
    assert_eq!(calls[1].q("$top"), Some("25"));
    assert!(
        both.to_string()
            .contains("exactly one of plan_id or bucket_id")
    );
}

#[tokio::test]
async fn m365_rest_planner_task_is_created_with_only_the_fields_given() {
    let sim = Sim::start(|_, _| Reply::json(201, &json!({"id": "t-1"}))).await;
    let client = graph(&sim);
    create_planner_task(
        &client,
        args(json!({
            "plan_id": "p-1", "title": "Ship", "bucket_id": "b-1",
            "assignee_user_ids": ["u-1", "u-2"], "priority": 1, "percent_complete": 0,
            "due_date": "2026-05-01T00:00:00Z"
        })),
    )
    .await
    .unwrap();
    create_planner_task(&client, args(json!({"plan_id": "p-1", "title": "Bare"})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/planner/tasks");
    assert_eq!(
        calls[0].json(),
        json!({
            "planId": "p-1", "title": "Ship", "bucketId": "b-1",
            "assignments": {"u-1": assigned(), "u-2": assigned()},
            "priority": 1, "percentComplete": 0, "dueDateTime": "2026-05-01T00:00:00Z"
        })
    );
    assert_eq!(calls[1].json(), json!({"planId": "p-1", "title": "Bare"}));
}

#[tokio::test]
async fn m365_rest_planner_task_update_assigns_and_unassigns_under_its_etag() {
    let sim = Sim::start(|seen, _| match seen.method.as_str() {
        "DELETE" => empty(204),
        _ => Reply::json(200, &json!({"id": "t-1", "@odata.etag": "e"})),
    })
    .await;
    let client = graph(&sim);
    update_planner_task(
        &client,
        args(json!({
            "task_id": "t-1", "etag": "task-tag", "percent_complete": 100, "bucket_id": "b-2",
            "start_date": "2026-04-01T00:00:00Z",
            "assignees_to_add": ["u-3"], "assignees_to_remove": ["u-1"]
        })),
    )
    .await
    .unwrap();
    let got = get_planner_task(&client, args(json!({"task_id": "t-1"})))
        .await
        .unwrap();
    let deleted = delete_planner_task(&client, args(json!({"task_id": "t-1", "etag": "t2"})))
        .await
        .unwrap();
    let nothing = update_planner_task(&client, args(json!({"task_id": "t-1", "etag": "e"})))
        .await
        .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[0].path, "/v1.0/planner/tasks/t-1");
    assert_eq!(calls[0].header("if-match"), Some("task-tag"));
    assert_eq!(
        calls[0].json(),
        json!({
            "bucketId": "b-2", "percentComplete": 100, "startDateTime": "2026-04-01T00:00:00Z",
            "assignments": {"u-3": assigned(), "u-1": null}
        })
    );
    assert!(
        calls[1]
            .q("$select")
            .unwrap()
            .starts_with("id,title,planId")
    );
    assert_eq!(got.data["@odata.etag"], "e");
    assert_eq!(
        (calls[2].method.as_str(), calls[2].header("if-match")),
        ("DELETE", Some("t2"))
    );
    assert_eq!(deleted.data, json!({"deleted": true, "task_id": "t-1"}));
    assert!(is_invalid(&nothing));
}

#[tokio::test]
async fn m365_rest_planner_task_details_are_read_and_patched_with_their_own_etag() {
    let sim = Sim::start(|_, _| Reply::json(200, &json!({"description": "d"}))).await;
    let client = graph(&sim);
    let details = get_planner_task_details(&client, args(json!({"task_id": "t-1"})))
        .await
        .unwrap();
    let checklist = json!({"c-1": {
        "@odata.type": "microsoft.graph.plannerChecklistItem", "title": "Step", "isChecked": false
    }, "c-0": null});
    update_planner_task_details(
        &client,
        args(json!({
            "task_id": "t-1", "etag": "details-tag", "description": "Notes",
            "preview_type": "checklist", "checklist": checklist.clone()
        })),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/planner/tasks/t-1/details");
    assert!(calls[0].raw_query.is_none());
    assert_eq!(details.data["description"], "d");
    assert_eq!(calls[1].method, "PATCH");
    assert_eq!(calls[1].path, "/v1.0/planner/tasks/t-1/details");
    assert_eq!(calls[1].header("if-match"), Some("details-tag"));
    assert_eq!(
        calls[1].json(),
        json!({"description": "Notes", "previewType": "checklist", "checklist": checklist})
    );
}

#[tokio::test]
async fn m365_rest_planner_rosters_and_their_members() {
    let sim = Sim::start(|seen, _| match (seen.method.as_str(), seen.path.as_str()) {
        ("GET", _) => Reply::json(200, &collection(&[json!({"id": "m-1"})], None)),
        ("DELETE", _) => empty(204),
        _ => Reply::json(201, &json!({"id": "r-1"})),
    })
    .await;
    let client = graph(&sim);
    let roster = create_planner_roster(&client, args(json!({})))
        .await
        .unwrap();
    let members = list_planner_roster_members(&client, args(json!({"roster_id": "r-1"})))
        .await
        .unwrap();
    add_planner_roster_member(&client, args(json!({"roster_id": "r-1", "user_id": "u-1"})))
        .await
        .unwrap();
    let removed = delete_planner_roster_member(
        &client,
        args(json!({"roster_id": "r-1", "member_id": "m-1"})),
    )
    .await
    .unwrap();
    let gone = delete_planner_roster(&client, args(json!({"roster_id": "r-1", "etag": "r-tag"})))
        .await
        .unwrap();
    let bad =
        add_planner_roster_member(&client, args(json!({"roster_id": "r-1", "user_id": "a b"})))
            .await
            .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 5);
    assert_eq!(
        (calls[0].method.as_str(), calls[0].path.as_str()),
        ("POST", "/v1.0/planner/rosters")
    );
    assert_eq!(calls[0].json(), json!({}));
    assert_eq!(roster.data["id"], "r-1");
    assert_eq!(calls[1].path, "/v1.0/planner/rosters/r-1/members");
    assert_eq!(members.data.len(), 1);
    assert_eq!(calls[2].json(), json!({"userId": "u-1"}));
    assert_eq!(calls[3].path, "/v1.0/planner/rosters/r-1/members/m-1");
    assert!(calls[3].header("if-match").is_none());
    assert_eq!(
        removed.data,
        json!({"deleted": true, "roster_id": "r-1", "member_id": "m-1"})
    );
    assert_eq!(
        (calls[4].path.as_str(), calls[4].header("if-match")),
        ("/v1.0/planner/rosters/r-1", Some("r-tag"))
    );
    assert_eq!(gone.data, json!({"deleted": true, "roster_id": "r-1"}));
    assert!(is_invalid(&bad));
}
