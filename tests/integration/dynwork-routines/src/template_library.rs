//! The template library: only an admin writes it, a draft is kept until it
//! is published, publishing refuses what §1, §4 and §5 of the contract rule
//! out, and every publish bumps the revision and keeps the one before it.

use chrono::Utc;
use dynapps_test_common::db_or_skip;
use dynwork_routines::ids::TemplateId;
use dynwork_routines::library::{self, LibraryError};
use dynwork_routines::store::{self, TemplateState};
use dynwork_routines::template::{Refusal, TemplateDraft};
use serde_json::{Value, json};
use systemprompt::identifiers::UserId;

use crate::support::seed_user;
use crate::template_support::{body, draft, seed_admin};

fn bypass(connectors: Value, folder: &str) -> Value {
    let mut body = body("Unattended review");
    body["definition"]["options"]["permissions"] = json!("bypass");
    body["definition"]["options"]["connectors"] = connectors;
    body["personal"]["folder"] = json!(folder);
    body
}

fn template_id(text: &str) -> TemplateId {
    TemplateId::parse(text).expect("a template id")
}

async fn refused(pool: &sqlx::PgPool, admin: &UserId, draft: &TemplateDraft) -> Refusal {
    match library::publish(pool, draft, admin, Utc::now()).await {
        Err(LibraryError::Refused(refusal)) => refusal,
        other => panic!("publish was not refused: {other:?}"),
    }
}

#[tokio::test]
async fn publishing_refuses_what_the_contract_rules_out() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    let cases = [
        (
            draft("no-connectors", &["sales"], false, bypass(json!([]), "required")),
            Refusal::BypassWithoutConnectors,
        ),
        (
            draft("no-folder", &["sales"], false, bypass(json!(["dynapps-odoo"]), "none")),
            Refusal::BypassWithoutFolder,
        ),
        (
            draft("mandatory-bypass", &["sales"], true, bypass(json!(["dynapps-odoo"]), "none")),
            Refusal::BypassMandatory,
        ),
        (
            draft("mandatory-folder", &["sales"], true, {
                let mut body = body("Needs a folder");
                body["personal"]["folder"] = json!("required");
                body
            }),
            Refusal::MandatoryFolder,
        ),
        (
            draft("all-connectors", &["sales"], false, {
                let mut body = body("Everything");
                body["definition"]["options"]["connectors"] = json!(["all"]);
                body
            }),
            Refusal::AllConnectors,
        ),
        (
            draft("no-anchor", &[], false, body("Nobody sees it")),
            Refusal::NoPlugins,
        ),
    ];
    for (draft, expected) in &cases {
        assert_eq!(refused(&db.pool, &admin, draft).await, *expected, "{}", draft.id);
    }
    let kept = store::list_templates(&db.pool).await.expect("list");
    assert!(kept.is_empty(), "a refused publish writes nothing: {kept:?}");
}

#[tokio::test]
async fn a_bypass_template_with_its_controls_publishes() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    let published = library::publish(
        &db.pool,
        &draft("unattended", &["sales"], false, bypass(json!(["dynapps-odoo"]), "required")),
        &admin,
        Utc::now(),
    )
    .await
    .expect("publish");
    assert_eq!(published.revision, 1);
    assert_eq!(published.state, TemplateState::Published);
    let revision = store::find_template_revision(&db.pool, &template_id("unattended"), 1)
        .await
        .expect("read the revision")
        .expect("the revision is kept");
    assert_eq!(revision.published_by, admin, "the history names its author");
}

#[tokio::test]
async fn a_draft_waits_and_every_publish_keeps_the_revision_before_it() {
    let db = db_or_skip!();
    let admin = seed_admin(&db.pool, "admin").await;
    let id = template_id("daily-odoo-review");

    let saved = library::save_draft(
        &db.pool,
        &draft("daily-odoo-review", &["sales"], false, body("Draft")),
        &admin,
        Utc::now(),
    )
    .await
    .expect("save the draft");
    assert_eq!((saved.revision, saved.state), (0, TemplateState::Draft));
    assert!(
        store::find_template_revision(&db.pool, &id, 1)
            .await
            .expect("read")
            .is_none(),
        "a draft has no revision yet"
    );

    let first = library::publish(
        &db.pool,
        &draft("daily-odoo-review", &["sales"], false, body("Daily Odoo review")),
        &admin,
        Utc::now(),
    )
    .await
    .expect("publish");
    assert_eq!((first.revision, first.state), (1, TemplateState::Published));
    let second = library::publish(
        &db.pool,
        &draft("daily-odoo-review", &["sales", "ops"], false, body("Odoo review")),
        &admin,
        Utc::now(),
    )
    .await
    .expect("publish again");
    assert_eq!(second.revision, 2);
    assert_eq!(second.plugins, vec!["sales".to_owned(), "ops".to_owned()]);

    let old = store::find_template_revision(&db.pool, &id, 1)
        .await
        .expect("read")
        .expect("revision 1 is kept");
    let new = store::find_template_revision(&db.pool, &id, 2)
        .await
        .expect("read")
        .expect("revision 2 is kept");
    assert_eq!(old.body.name, "Daily Odoo review");
    assert_eq!(new.body.name, "Odoo review");

    let late = library::save_draft(
        &db.pool,
        &draft("daily-odoo-review", &["sales"], false, body("Back to draft")),
        &admin,
        Utc::now(),
    )
    .await;
    assert!(
        matches!(late, Err(LibraryError::NotDraft)),
        "a published template changes only by publishing: {late:?}"
    );
}

#[tokio::test]
async fn only_an_admin_writes_the_library() {
    let db = db_or_skip!();
    let person = seed_user(&db.pool, "alice").await;
    let template = draft("daily", &["sales"], false, body("Daily"));
    let published = library::publish(&db.pool, &template, &person, Utc::now()).await;
    assert!(matches!(published, Err(LibraryError::NotAdmin)), "{published:?}");
    let saved = library::save_draft(&db.pool, &template, &person, Utc::now()).await;
    assert!(matches!(saved, Err(LibraryError::NotAdmin)), "{saved:?}");
    assert!(
        store::list_templates(&db.pool)
            .await
            .expect("list")
            .is_empty()
    );
}

#[test]
fn a_template_never_carries_a_timezone_or_a_folder() {
    let mut zoned = body("Zoned");
    zoned["definition"]["schedule"]["timezone"] = json!("Europe/Madrid");
    let mut foldered = body("Foldered");
    foldered["definition"]["options"]["folder"] = json!("/Users/alice");
    for body in [zoned, foldered] {
        let parsed: Result<TemplateDraft, _> = serde_json::from_value(json!({
            "id": "personal", "mandatory": false, "plugins": ["sales"], "body": body,
        }));
        assert!(parsed.is_err(), "personal fields are refused");
    }
}
