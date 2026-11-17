//! The admin pages of the template library, `/admin/routine-templates`:
//! only an admin reaches any page or form; the definition form saves a draft
//! and publishes through the library, and a refused publish comes back on the
//! form with its reason; close, reopen and retire go through the library and
//! return with a flash; and the list and each tab render.

use axum::http::{Method, StatusCode};
use chrono::Utc;
use dynapps_test_common::{db_or_skip, empty_services_view};
use dynwork_routines::ids::{RoutineId, RunId, RunState, TemplateId};
use dynwork_routines::store::{self, PutRun, RunOutcome, TemplateState};
use dynwork_routines::template::{Permissions, RetireMode, ScheduleLock};
use serde_json::json;

use crate::page_support::{get_page, pages, post_form, repo_services};
use crate::support::{Call, router, seed_user};
use crate::template_support::{activate, linked, publish, rule, seed_admin};

const LIST: &str = "/admin/routine-templates";

fn template_id(text: &str) -> TemplateId {
    TemplateId::parse(text).expect("a template id")
}

// A publishable definition form, as the page posts it. `extra` adds or, when
// a name repeats, overrides fields (the form handler takes the last value).
fn form<'a>(
    name: &'a str,
    intent: &'a str,
    extra: &[(&'a str, &'a str)],
) -> Vec<(&'a str, &'a str)> {
    let mut fields = vec![
        ("name", name),
        ("description", "Yesterday's Odoo activity, summarised"),
        ("instructions_kind", "prompt"),
        ("skill", ""),
        ("text", "Review yesterday's Odoo activity"),
        ("agent", "build"),
        ("model", ""),
        ("cron", "0 8 * * 1-5"),
        ("permissions", "ask"),
        ("connectors", "dynapps-odoo"),
        ("time_limit", "30"),
        ("recover", "recent"),
        ("catch_up", "6"),
        ("notify", "failures"),
        ("lock_schedule", "time"),
        ("time_limit_max", "60"),
        ("folder", "none"),
        ("hint", ""),
        ("plugins", "dynapps-commons"),
        ("intent", intent),
    ];
    fields.extend_from_slice(extra);
    fields
}

#[tokio::test]
async fn a_non_admin_is_refused_every_template_page_and_form() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    let alice = seed_user(&db.pool, "alice").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    let pages = pages(&db.pool, &alice, false, empty_services_view());

    for uri in [
        "/routine-templates",
        "/routine-templates/new",
        "/routine-templates/sales-review",
        "/routine-templates/sales-review?tab=audience",
        "/routine-templates/sales-review?tab=activations",
        "/routine-templates/sales-review?tab=history",
    ] {
        assert_eq!(
            get_page(&pages, uri).await.status,
            StatusCode::FORBIDDEN,
            "GET {uri}"
        );
    }
    let new = form("Taken over", "publish", &[("id", "taken-over")]);
    let save = form("Taken over", "publish", &[]);
    let posts: [(&str, &[(&str, &str)]); 5] = [
        ("/routine-templates/new", &new),
        ("/routine-templates/sales-review", &save),
        ("/routine-templates/sales-review/close", &[]),
        ("/routine-templates/sales-review/reopen", &[]),
        (
            "/routine-templates/sales-review/retire",
            &[("mode", "remove")],
        ),
    ];
    for (uri, fields) in posts {
        assert_eq!(
            post_form(&pages, uri, fields).await.status,
            StatusCode::FORBIDDEN,
            "POST {uri}"
        );
    }

    let kept = store::find_template(&db.pool, &template_id("sales-review"))
        .await
        .expect("read")
        .expect("the template");
    assert_eq!((kept.state, kept.revision), (TemplateState::Published, 1));
    assert_eq!(kept.body.name, "Sales review");
    assert!(
        store::find_template(&db.pool, &template_id("taken-over"))
            .await
            .expect("read")
            .is_none()
    );
}

#[tokio::test]
async fn an_admin_saves_a_draft_and_publishes_revisions_from_the_form() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    let pages = pages(&db.pool, &admin, true, repo_services());
    let detail = format!("{LIST}/daily-odoo-review");

    let drafted = post_form(
        &pages,
        "/routine-templates/new",
        &form("Daily Odoo review", "draft", &[("id", "daily-odoo-review")]),
    )
    .await;
    assert!(drafted.redirected(&detail, "ok"), "{:?}", drafted.location);
    let draft = store::find_template(&db.pool, &template_id("daily-odoo-review"))
        .await
        .expect("read")
        .expect("the draft");
    assert_eq!((draft.state, draft.revision), (TemplateState::Draft, 0));

    let published = post_form(
        &pages,
        "/routine-templates/daily-odoo-review",
        &form("Daily Odoo review", "publish", &[]),
    )
    .await;
    assert!(
        published.redirected(&detail, "ok"),
        "{:?}",
        published.location
    );
    let first = store::find_template(&db.pool, &template_id("daily-odoo-review"))
        .await
        .expect("read")
        .expect("the template");
    assert_eq!((first.state, first.revision), (TemplateState::Published, 1));
    assert_eq!(first.plugins, vec!["dynapps-commons".to_owned()]);
    assert!(!first.mandatory);
    let definition = &first.body.definition;
    assert_eq!(definition.schedule.cron, "0 8 * * 1-5");
    assert_eq!(definition.agent.as_deref(), Some("build"));
    assert_eq!(definition.model, None, "a blank model is no model");
    assert_eq!(definition.options.permissions, Permissions::Ask);
    assert_eq!(
        definition.options.connectors,
        vec!["dynapps-odoo".to_owned()]
    );
    assert_eq!(definition.options.time_limit, Some(30));
    assert_eq!(definition.options.recover, Some(json!("recent")));
    assert_eq!(definition.options.catch_up, Some(json!(6)));
    assert_eq!(definition.options.notify.as_deref(), Some("failures"));
    assert_eq!(first.body.locks.schedule, ScheduleLock::Time);
    assert_eq!(first.body.locks.time_limit_max, Some(60));
    assert!(!first.body.locks.model);
    assert_eq!(first.body.personal.hint, None, "a blank hint is no hint");

    let again = post_form(
        &pages,
        "/routine-templates/daily-odoo-review",
        &form("Morning Odoo review", "publish", &[("lock_model", "on")]),
    )
    .await;
    assert!(again.redirected(&detail, "ok"), "{:?}", again.location);
    let second = store::find_template(&db.pool, &template_id("daily-odoo-review"))
        .await
        .expect("read")
        .expect("the template");
    assert_eq!(second.revision, 2);
    assert_eq!(second.body.name, "Morning Odoo review");
    assert!(second.body.locks.model);
    let kept = store::find_template_revision(&db.pool, &template_id("daily-odoo-review"), 1)
        .await
        .expect("read")
        .expect("revision 1 stays");
    assert_eq!(kept.body.name, "Daily Odoo review");
    assert_eq!(kept.published_by, admin);
}

#[tokio::test]
async fn a_refused_publish_comes_back_on_the_form_with_its_reason() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    let pages = pages(&db.pool, &admin, true, repo_services());

    let bypass = post_form(
        &pages,
        "/routine-templates/new",
        &form(
            "Unattended review",
            "publish",
            &[("id", "unattended"), ("permissions", "bypass")],
        ),
    )
    .await;
    assert_eq!(
        bypass.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        bypass.text
    );
    assert!(
        bypass.has("needs a folder chosen at activation"),
        "the library's refusal is on the page"
    );
    assert!(bypass.has("Unattended review"), "the posted values stay");

    let typo = post_form(
        &pages,
        "/routine-templates/new",
        &form(
            "Typo review",
            "publish",
            &[("id", "typo"), ("connectors", "dynapps-odo")],
        ),
    )
    .await;
    assert_eq!(
        typo.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        typo.text
    );
    assert!(typo.has("dynapps-odo"), "the unknown connector is named");

    let unknown_plugin = post_form(
        &pages,
        "/routine-templates/new",
        &form(
            "Stray review",
            "publish",
            &[("id", "stray"), ("plugins", "no-such-plugin")],
        ),
    )
    .await;
    assert_eq!(unknown_plugin.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(unknown_plugin.has("no-such-plugin"));

    for id in ["unattended", "typo", "stray"] {
        assert!(
            store::find_template(&db.pool, &template_id(id))
                .await
                .expect("read")
                .is_none(),
            "{id} was not stored"
        );
    }
}

#[tokio::test]
async fn close_reopen_and_retire_go_through_the_library_and_redirect() {
    let db = db_or_skip!();
    let desktop = router(&db.pool);
    let admin = seed_admin(&db.pool, "admin").await;
    let alice = seed_user(&db.pool, "alice").await;
    rule(&db.pool, "sales", "role", "user", "allow").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    let (status, task) = activate(
        &desktop,
        "alice",
        "task-1",
        linked("Sales review", "sales-review", 1),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{task}");
    let pages = pages(&db.pool, &admin, true, empty_services_view());
    let detail = format!("{LIST}/sales-review");
    let state = || async {
        let template = store::find_template(&db.pool, &template_id("sales-review"))
            .await
            .expect("read")
            .expect("the template");
        (template.state, template.retire_mode)
    };

    let closed = post_form(&pages, "/routine-templates/sales-review/close", &[]).await;
    assert!(closed.redirected(&detail, "ok"), "{:?}", closed.location);
    assert_eq!(state().await, (TemplateState::Closed, None));
    let twice = post_form(&pages, "/routine-templates/sales-review/close", &[]).await;
    assert!(twice.redirected(&detail, "err"), "{:?}", twice.location);

    let reopened = post_form(&pages, "/routine-templates/sales-review/reopen", &[]).await;
    assert!(
        reopened.redirected(&detail, "ok"),
        "{:?}",
        reopened.location
    );
    assert_eq!(state().await, (TemplateState::Published, None));

    let bogus = post_form(
        &pages,
        "/routine-templates/sales-review/retire",
        &[("mode", "archive")],
    )
    .await;
    assert!(bogus.redirected(&detail, "err"), "{:?}", bogus.location);
    assert_eq!(state().await, (TemplateState::Published, None));

    let retired = post_form(
        &pages,
        "/routine-templates/sales-review/retire",
        &[("mode", "pause")],
    )
    .await;
    assert!(retired.redirected(&detail, "ok"), "{:?}", retired.location);
    assert_eq!(
        state().await,
        (TemplateState::Retired, Some(RetireMode::Pause))
    );
    let live = store::list_live_routines(&db.pool, &alice)
        .await
        .expect("list");
    assert_eq!(live.len(), 1);
    assert_eq!(
        live[0].definition["enabled"], false,
        "retire·pause paused it"
    );

    let too_late = post_form(&pages, "/routine-templates/sales-review/reopen", &[]).await;
    assert!(
        too_late.redirected(&detail, "err"),
        "{:?}",
        too_late.location
    );
}

#[tokio::test]
async fn the_pages_show_reach_activations_with_conversations_and_history() {
    let db = db_or_skip!();
    let desktop = router(&db.pool);
    let admin = seed_admin(&db.pool, "admin").await;
    let alice = seed_user(&db.pool, "alice").await;
    seed_user(&db.pool, "bob").await;
    rule(&db.pool, "sales", "user", "alice", "allow").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    publish(&db.pool, &admin, "sales-review", &["sales"], "Sales review").await;
    let (status, task) = activate(
        &desktop,
        "alice",
        "task-1",
        linked("Sales review", "sales-review", 1),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{task}");
    for (run, state) in [
        ("ses_alice_ok", RunState::Ok),
        ("ses_alice_failed", RunState::Failed),
    ] {
        let written = store::put_run(
            &db.pool,
            &PutRun {
                user: &alice,
                id: &RunId::parse(run).expect("a run id"),
                routine_id: &RoutineId::parse("task-1").expect("a routine id"),
                device: "mac-1",
                state,
                reason: None,
                started_at: Some(Utc::now()),
                ended_at: None,
                now: Utc::now(),
            },
        )
        .await
        .expect("a run");
        assert!(matches!(written, RunOutcome::Written(_)));
    }
    // Core's record of the run `ses_alice_ok`: its client session is
    // uuidv5(7c1f5b6e-3a2d-4e8f-9b0c-2d6a1e4f8c73, "ses_alice_ok").
    let conversation = "0b4f5bb6-8c3a-4d0e-9a51-5d1c7f3e2a10";
    sqlx::query(
        "INSERT INTO conversation_session_contexts (user_id, client_session_id, context_id)
         VALUES ('alice', '46a82b9d-9d94-5358-8f48-48d894039cd7', $1)",
    )
    .bind(conversation)
    .execute(&*db.pool)
    .await
    .expect("record the run's conversation");
    let pages = pages(&db.pool, &admin, true, empty_services_view());

    let list = get_page(&pages, "/routine-templates").await;
    assert_eq!(list.status, StatusCode::OK, "{}", list.text);
    assert!(list.has("Scheduled templates"));
    assert!(list.has("Sales review"));
    assert!(list.has(&format!("{LIST}/sales-review")));

    let definition = get_page(&pages, "/routine-templates/sales-review").await;
    assert_eq!(definition.status, StatusCode::OK, "{}", definition.text);
    assert!(definition.has("0 8 * * 1-5"));

    let audience = get_page(&pages, "/routine-templates/sales-review?tab=audience").await;
    assert_eq!(audience.status, StatusCode::OK, "{}", audience.text);
    assert!(audience.has("alice@example.test"), "alice sees it");
    assert!(!audience.has("bob@example.test"), "bob does not");
    assert!(audience.has("/admin/access-control"), "a link to the rules");

    let activations = get_page(&pages, "/routine-templates/sales-review?tab=activations").await;
    assert_eq!(activations.status, StatusCode::OK, "{}", activations.text);
    assert!(activations.has("1 of 2"), "the revision the task follows");
    assert!(activations.has(&format!("/admin/analysis/conversations/{conversation}")));
    assert_eq!(
        activations
            .text
            .matches("/admin/analysis/conversations/")
            .count(),
        1,
        "the run without a recorded conversation has no link"
    );

    let history = get_page(&pages, "/routine-templates/sales-review?tab=history").await;
    assert_eq!(history.status, StatusCode::OK, "{}", history.text);
    assert!(history.has("Revision 2"));
    assert!(history.has("Revision 1"));

    let new = get_page(&pages, "/routine-templates/new").await;
    assert_eq!(new.status, StatusCode::OK, "{}", new.text);
    let missing = get_page(&pages, "/routine-templates/no-such-template").await;
    assert_eq!(missing.status, StatusCode::NOT_FOUND);

    let feed = Call::new(Method::GET, "/changes?since=0", "alice")
        .send(&desktop)
        .await;
    assert_eq!(feed.0, StatusCode::OK, "reading pages wrote nothing");
    assert_eq!(feed.1["routines"].as_array().map(Vec::len), Some(1));
}
