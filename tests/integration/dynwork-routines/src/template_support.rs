//! What the template suites share: an admin author, templates in the
//! contract's shape, access rules on their anchor plugins, a task linked to a
//! template, and `GET` calls that keep the response's headers.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use dynwork_routines::library;
use dynwork_routines::store::TemplateRecord;
use dynwork_routines::template::TemplateDraft;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use tower::ServiceExt;

use crate::support::{desktop_token, seed_user};

pub(crate) async fn seed_admin(pool: &PgPool, name: &str) -> UserId {
    let admin = seed_user(pool, name).await;
    sqlx::query("UPDATE users SET roles = ARRAY['user', 'admin'] WHERE id = $1")
        .bind(name)
        .execute(pool)
        .await
        .expect("make the user an admin");
    admin
}

// A template's body as §1 of the contract shapes it, without id, revision
// and mandatory.
pub(crate) fn body(name: &str) -> Value {
    json!({
        "name": name,
        "description": "Yesterday's Odoo activity, summarised",
        "where": "local",
        "definition": {
            "instructions": { "kind": "prompt", "text": "Review yesterday's Odoo activity" },
            "agent": "build",
            "schedule": { "cron": "0 8 * * 1-5" },
            "options": {
                "permissions": "ask",
                "connectors": ["dynapps-odoo"],
                "timeLimit": 30,
                "notify": "failures",
            },
        },
        "locks": { "model": false, "schedule": "time", "timeLimitMax": 60 },
        "personal": { "folder": "none", "hint": null },
    })
}

pub(crate) fn draft(id: &str, plugins: &[&str], mandatory: bool, body: Value) -> TemplateDraft {
    let mut draft = json!({ "id": id, "mandatory": mandatory, "plugins": plugins });
    draft["body"] = body;
    serde_json::from_value(draft).expect("a template draft")
}

pub(crate) async fn publish(
    pool: &PgPool,
    author: &UserId,
    id: &str,
    plugins: &[&str],
    name: &str,
) -> TemplateRecord {
    library::publish(
        pool,
        &draft(id, plugins, false, body(name)),
        author,
        Utc::now(),
    )
    .await
    .expect("publish the template")
}

// An anchor plugin as a catalogue entity closed by default, with one rule.
pub(crate) async fn rule(pool: &PgPool, plugin: &str, rule_type: &str, value: &str, access: &str) {
    sqlx::query(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
         VALUES ('plugin', $1, false, 'migration') ON CONFLICT DO NOTHING",
    )
    .bind(plugin)
    .execute(pool)
    .await
    .expect("an anchor plugin entity");
    sqlx::query(
        "INSERT INTO access_control_rules (entity_type, entity_id, rule_type, rule_value, access, source)
         VALUES ('plugin', $1, $2, $3, $4, 'migration')",
    )
    .bind(plugin)
    .bind(rule_type)
    .bind(value)
    .bind(access)
    .execute(pool)
    .await
    .expect("an access rule");
}

pub(crate) async fn drop_rules(pool: &PgPool, plugin: &str) {
    sqlx::query("DELETE FROM access_control_rules WHERE entity_type = 'plugin' AND entity_id = $1")
        .bind(plugin)
        .execute(pool)
        .await
        .expect("drop the plugin's rules");
}

// Why: phase 04 derives the two columns from `definition.template` on every
// write; until then a test links a task by setting them.
pub(crate) async fn link(pool: &PgPool, who: &str, routine: &str, template: &str, revision: i64) {
    sqlx::query(
        "UPDATE dynwork_routines SET template_id = $3, template_revision = $4
         WHERE user_id = $1 AND id = $2",
    )
    .bind(who)
    .bind(routine)
    .bind(template)
    .bind(revision)
    .execute(pool)
    .await
    .expect("link the task to the template");
}

pub(crate) struct Answer {
    pub(crate) status: StatusCode,
    pub(crate) etag: Option<String>,
    pub(crate) body: Value,
}

pub(crate) async fn get(
    router: &Router,
    uri: &str,
    who: &str,
    if_none_match: Option<&str>,
) -> Answer {
    let mut request = Request::builder()
        .uri(uri)
        .header("authorization", format!("Bearer {}", desktop_token(who)));
    if let Some(tag) = if_none_match {
        request = request.header("if-none-match", tag);
    }
    let response = router
        .clone()
        .oneshot(request.body(Body::empty()).expect("a request"))
        .await
        .expect("the router answers");
    let status = response.status();
    let etag = response
        .headers()
        .get("etag")
        .map(|value| value.to_str().expect("an ASCII ETag").to_owned());
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("a body")
        .to_bytes();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("a JSON body")
    };
    Answer { status, etag, body }
}

// The ids of the templates a `GET /templates` answer serves.
pub(crate) fn ids(answer: &Answer) -> Vec<String> {
    let mut ids: Vec<String> = answer.body["templates"]
        .as_array()
        .expect("templates")
        .iter()
        .map(|template| template["id"].as_str().expect("an id").to_owned())
        .collect();
    ids.sort();
    ids
}
