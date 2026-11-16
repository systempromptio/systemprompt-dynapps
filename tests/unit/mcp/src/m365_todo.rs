//! The To Do tools against a simulated Graph: lists and tasks read without
//! `$select` (Graph v1.0 refuses it there), task create and update bodies
//! with UTC due dates, list creation, the delete reply, and ids refused
//! before Graph.

use serde_json::json;
use systemprompt_mcp_microsoft_365::tools::todo::{
    create_todo_list, create_todo_task, delete_todo_task, get_todo_task, list_todo_lists,
    list_todo_tasks, update_todo_task,
};

use crate::m365_pim_support::{args, assert_domain_registered, empty, graph, is_invalid};
use crate::m365_support::{Reply, Sim, collection};

async fn echo() -> Sim {
    Sim::start(|seen, _| match seen.method.as_str() {
        "DELETE" => empty(204),
        "GET" if seen.path.ends_with("/tasks") || seen.path.ends_with("/lists") => {
            Reply::json(200, &collection(&[json!({"id": "t1"})], None))
        },
        _ => Reply::json(201, &json!({"id": "t1", "title": "Pay"})),
    })
    .await
}

#[test]
fn m365_todo_all_seven_belgian_tools_are_registered_as_pinned() {
    assert_domain_registered("todo", 7);
}

#[tokio::test]
async fn m365_todo_lists_and_tasks_are_read_without_select() {
    let sim = echo().await;
    let client = graph(&sim);
    let lists = list_todo_lists(&client, args(json!({"skip": 3})))
        .await
        .unwrap();
    let tasks = list_todo_tasks(&client, args(json!({"list_id": "L1=", "top": 75})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/todo/lists");
    assert_eq!(calls[0].q("$skip"), Some("3"));
    assert_eq!(calls[0].q("$top"), None);
    assert_eq!(calls[1].path, "/v1.0/me/todo/lists/L1=/tasks");
    assert_eq!(calls[1].q("$top"), Some("50"));
    assert_eq!(calls[1].q("$skip"), None);
    assert!(calls.iter().all(|c| c.q("$select").is_none()));
    assert_eq!(lists.data, vec![json!({"id": "t1"})]);
    assert_eq!(tasks.data, vec![json!({"id": "t1"})]);
}

#[tokio::test]
async fn m365_todo_get_task_reads_one_task_of_one_list() {
    let sim = echo().await;
    let reply = get_todo_task(
        &graph(&sim),
        args(json!({"list_id": "L1", "task_id": "T1"})),
    )
    .await
    .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.method, "GET");
    assert_eq!(call.path, "/v1.0/me/todo/lists/L1/tasks/T1");
    assert_eq!(reply.data["title"], "Pay");
}

#[tokio::test]
async fn m365_todo_create_task_posts_title_importance_body_and_due_date() {
    let sim = echo().await;
    let input = args(json!({
        "list_id": "L1",
        "title": "Pay invoice",
        "body": "Before Friday",
        "due_date": "2024-12-31"
    }));
    create_todo_task(&graph(&sim), input).await.unwrap();
    create_todo_task(
        &graph(&sim),
        args(json!({"list_id": "L1", "title": "Call", "importance": "high"})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].method, "POST");
    assert_eq!(calls[0].path, "/v1.0/me/todo/lists/L1/tasks");
    assert_eq!(
        calls[0].json(),
        json!({
            "title": "Pay invoice",
            "importance": "normal",
            "body": {"content": "Before Friday", "contentType": "text"},
            "dueDateTime": {"dateTime": "2024-12-31", "timeZone": "UTC"}
        })
    );
    assert_eq!(
        calls[1].json(),
        json!({"title": "Call", "importance": "high"})
    );
}

#[tokio::test]
async fn m365_todo_update_task_patches_only_what_was_given() {
    let sim = echo().await;
    let input = args(json!({
        "list_id": "L1",
        "task_id": "T1",
        "status": "completed",
        "due_date": "2025-01-15",
        "title": null
    }));
    update_todo_task(&graph(&sim), input).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.method, "PATCH");
    assert_eq!(call.path, "/v1.0/me/todo/lists/L1/tasks/T1");
    assert_eq!(
        call.json(),
        json!({
            "status": "completed",
            "dueDateTime": {"dateTime": "2025-01-15", "timeZone": "UTC"}
        })
    );
}

#[tokio::test]
async fn m365_todo_delete_task_answers_like_the_belgian_server() {
    let sim = echo().await;
    let reply = delete_todo_task(
        &graph(&sim),
        args(json!({"list_id": "L1", "task_id": "T1"})),
    )
    .await
    .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.method, "DELETE");
    assert_eq!(call.path, "/v1.0/me/todo/lists/L1/tasks/T1");
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({
            "data": {"deleted": true, "list_id": "L1", "task_id": "T1"},
            "message": "success"
        })
    );
}

#[tokio::test]
async fn m365_todo_create_list_posts_its_name() {
    let sim = echo().await;
    let reply = create_todo_list(&graph(&sim), args(json!({"display_name": "Q3 close"})))
        .await
        .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.method, "POST");
    assert_eq!(call.path, "/v1.0/me/todo/lists");
    assert_eq!(call.json(), json!({"displayName": "Q3 close"}));
    assert_eq!(reply.data["id"], "t1");
}

#[tokio::test]
async fn m365_todo_list_and_task_ids_are_checked_before_graph() {
    let sim = echo().await;
    let client = graph(&sim);
    let errors = [
        list_todo_tasks(&client, args(json!({"list_id": "../x"})))
            .await
            .unwrap_err(),
        get_todo_task(&client, args(json!({"list_id": "L1", "task_id": "a\\b"})))
            .await
            .unwrap_err(),
        delete_todo_task(&client, args(json!({"list_id": "L 1", "task_id": "T1"})))
            .await
            .unwrap_err(),
        update_todo_task(&client, args(json!({"list_id": "L1", "task_id": ""})))
            .await
            .unwrap_err(),
        create_todo_task(&client, args(json!({"list_id": "L1/..", "title": "t"})))
            .await
            .unwrap_err(),
    ];

    assert!(errors.iter().all(is_invalid), "{errors:?}");
    assert!(sim.calls().is_empty());
}
