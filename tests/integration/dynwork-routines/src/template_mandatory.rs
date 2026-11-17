//! Mandatory templates (contract §4): `GET /templates` makes one unclaimed
//! task of each published, mandatory template the caller sees and has no
//! live task of, once however many desktops ask at the same time; a desktop
//! claims it with its time zone, and the person may pause it but never delete
//! it while it stays mandatory for them.

use axum::http::{Method, StatusCode};
use chrono::Utc;
use dynapps_test_common::db_or_skip;
use dynwork_routines::ids::{RoutineId, TemplateId};
use dynwork_routines::store::{self, DeleteRoutine, WriteOutcome};
use serde_json::json;

use crate::support::{Call, drain_feed, router, seed_user};
use crate::template_support::{
    body, draft, drop_rules, get, publish_draft, rule, seed_admin, template_columns,
};

async fn live_tasks(pool: &sqlx::PgPool, who: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM dynwork_routines WHERE user_id = $1 AND deleted_at IS NULL",
    )
    .bind(who)
    .fetch_one(pool)
    .await
    .expect("count the live tasks")
}

#[tokio::test]
async fn a_mandatory_template_is_made_once_unclaimed_and_claimed_with_a_time_zone() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    publish_draft(
        &db.pool,
        &admin,
        draft("morning", &["sales"], true, body("Morning check")),
    )
    .await;
    let router = router(&db.pool);

    let (one, two) = tokio::join!(
        get(&router, "/templates", "alice", None),
        get(&router, "/templates", "alice", None),
    );
    assert_eq!((one.status, two.status), (StatusCode::OK, StatusCode::OK));
    assert_eq!(
        live_tasks(&db.pool, "alice").await,
        1,
        "two desktops, one task"
    );
    get(&router, "/templates", "alice", None).await;
    assert_eq!(
        live_tasks(&db.pool, "alice").await,
        1,
        "and none on the next read"
    );

    let (feed, _) = drain_feed(&router, "alice", "0", 50).await;
    assert_eq!(feed.len(), 1, "{feed:?}");
    let task = &feed[0];
    assert_eq!(task["owner_device"], "", "unclaimed");
    assert_eq!(task["revision"], 1);
    assert_eq!(
        task["definition"],
        json!({
            "name": "Morning check",
            "enabled": true,
            "instructions": { "kind": "prompt", "text": "Review yesterday's Odoo activity" },
            "agent": "build",
            "schedule": { "cron": "0 8 * * 1-5" },
            "options": {
                "permissions": "ask",
                "connectors": ["dynapps-odoo"],
                "timeLimit": 30,
                "recover": "recent",
                "catchUp": 6,
                "notify": "failures",
            },
            "template": { "id": "morning", "revision": 1 },
        }),
        "the template's defaults, linked, in no time zone"
    );
    let id = task["id"].as_str().expect("an id").to_owned();
    assert_eq!(
        template_columns(&db.pool, "alice", &id).await,
        (Some("morning".to_owned()), Some(1))
    );

    let mut claimed = task["definition"].clone();
    claimed["schedule"]["timezone"] = json!("Europe/Madrid");
    let (status, written) = Call::new(Method::PUT, format!("/routines/{id}"), "alice")
        .if_match(1)
        .body(json!({ "owner_device": "mac-1", "owner_name": "Laptop", "definition": claimed }))
        .send(&router)
        .await;
    assert_eq!(status, StatusCode::OK, "the claim: {written}");
    assert_eq!(written["owner_device"], "mac-1");
    assert_eq!(
        written["definition"]["schedule"]["timezone"],
        "Europe/Madrid"
    );
}

#[tokio::test]
async fn a_mandatory_task_is_paused_but_never_deleted_while_it_stays_mandatory() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    let alice = seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    publish_draft(
        &db.pool,
        &admin,
        draft("morning", &["sales"], true, body("Morning check")),
    )
    .await;
    let router = router(&db.pool);
    get(&router, "/templates", "alice", None).await;
    let (feed, _) = drain_feed(&router, "alice", "0", 50).await;
    let id = feed[0]["id"].as_str().expect("an id").to_owned();

    let (status, refused) = Call::new(Method::DELETE, format!("/routines/{id}"), "alice")
        .if_match(1)
        .send(&router)
        .await;
    assert_eq!(status, StatusCode::CONFLICT, "{refused}");
    assert_eq!(refused["error"], "template_mandatory");
    let page_delete = store::delete_routine(
        &db.pool,
        &DeleteRoutine {
            user: &alice,
            id: &RoutineId::parse(&id).expect("a routine id"),
            expected: None,
            mandatory: &[TemplateId::parse("morning").expect("a template id")],
            now: Utc::now(),
        },
    )
    .await
    .expect("the page's delete");
    assert_eq!(page_delete, WriteOutcome::Mandatory, "nor from the page");

    let mut paused = feed[0]["definition"].clone();
    paused["enabled"] = json!(false);
    let (status, written) = Call::new(Method::PUT, format!("/routines/{id}"), "alice")
        .if_match(1)
        .body(json!({ "definition": paused }))
        .send(&router)
        .await;
    assert_eq!(status, StatusCode::OK, "the pause: {written}");

    drop_rules(&db.pool, "sales").await;
    let (status, deleted) = Call::new(Method::DELETE, format!("/routines/{id}"), "alice")
        .if_match(2)
        .send(&router)
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "no longer mandatory for someone who does not see it: {deleted}"
    );
}

#[tokio::test]
async fn whoever_does_not_see_a_mandatory_template_gets_no_task_of_it() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    seed_user(&db.pool, "alice").await;
    seed_user(&db.pool, "bob").await;
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    publish_draft(
        &db.pool,
        &admin,
        draft("morning", &["sales"], true, body("Morning check")),
    )
    .await;
    let router = router(&db.pool);

    let bob = get(&router, "/templates", "bob", None).await;
    assert_eq!(bob.status, StatusCode::OK);
    assert_eq!(live_tasks(&db.pool, "bob").await, 0, "bob never saw it");

    get(&router, "/templates", "alice", None).await;
    assert_eq!(live_tasks(&db.pool, "alice").await, 1);
    drop_rules(&db.pool, "sales").await;
    let after = get(&router, "/templates", "alice", None).await;
    assert_eq!(
        after.body["links"],
        json!([{ "id": "morning", "state": "no_access", "retire": null }])
    );
    assert_eq!(
        live_tasks(&db.pool, "alice").await,
        1,
        "nothing new is made"
    );
}
