//! `GET /models`: the list the instance can serve, in the shape both clients
//! read, behind the same auth as every other route.

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};

use crate::http_support::{App, desktop_token, desktop_token_for_client, http_suite};
use crate::models_support::DENIED_ROLE;

fn ids(body: &Value) -> Vec<&str> {
    body["models"]
        .as_array()
        .map(|models| {
            models
                .iter()
                .filter_map(|model| model["id"].as_str())
                .collect()
        })
        .unwrap_or_default()
}

async fn the_list_carries_what_the_phone_and_the_desktop_read(app: &App) {
    let answer = app
        .call(Method::GET, "/models")
        .bearer(&desktop_token("alice"))
        .send()
        .await;
    assert_eq!(answer.status, StatusCode::OK, "{:?}", answer.body);
    assert_eq!(answer.headers["cache-control"], "no-store");
    assert_eq!(
        answer.body,
        json!({
            "models": [
                {
                    "id": "claude-opus",
                    "name": "Claude Opus",
                    "providerID": "anthropic",
                    "provider": { "id": "anthropic", "name": "Anthropic" },
                    "reasoning": true,
                    "tools": true,
                    "limit": { "context": 1_000_000, "output": 128_000 }
                },
                {
                    "id": "claude-sonnet",
                    "name": "Claude Sonnet",
                    "providerID": "anthropic",
                    "provider": { "id": "anthropic", "name": "Anthropic" },
                    "reasoning": true,
                    "tools": true,
                    "limit": { "context": 200_000, "output": 64_000 }
                },
                {
                    "id": "deepseek-chat",
                    "name": "deepseek-chat",
                    "providerID": "deepseek",
                    "provider": { "id": "deepseek", "name": "deepseek" },
                    "reasoning": false,
                    "tools": false,
                    "limit": { "context": 128_000, "output": 8192 }
                }
            ],
            "default": "claude-sonnet"
        })
    );
}

async fn a_phone_reads_the_same_list(app: &App) {
    let (_, token) = app.phone("alice").await;
    let answer = app.call(Method::GET, "/models").bearer(&token).send().await;
    assert_eq!(answer.status, StatusCode::OK, "{:?}", answer.body);
    assert_eq!(answer.body["models"].as_array().map(Vec::len), Some(3));
}

async fn a_model_of_the_list_is_a_model_a_thread_and_a_prompt_accept(app: &App) {
    let listed = app
        .call(Method::GET, "/models")
        .bearer(&desktop_token("alice"))
        .send()
        .await;
    let model = &listed.body["models"][2];
    let created = app
        .call(Method::PUT, "/threads/ses_01MODELS00000000000000001")
        .bearer(&desktop_token("alice"))
        .json(&json!({
            "title": "pick",
            "model": { "providerID": model["providerID"], "id": model["id"] }
        }))
        .send()
        .await;
    assert_eq!(created.status, StatusCode::OK, "{:?}", created.body);
    assert_eq!(created.body["model"]["providerID"], "deepseek");
    assert_eq!(created.body["model"]["id"], "deepseek-chat");
}

async fn a_role_the_access_rules_deny_loses_a_model(app: &App) {
    app.source.set_roles("alice", &["staff"]);
    app.source.set_roles("bob", &[DENIED_ROLE]);
    let alice = app
        .call(Method::GET, "/models")
        .bearer(&desktop_token("alice"))
        .send()
        .await;
    let bob = app
        .call(Method::GET, "/models")
        .bearer(&desktop_token("bob"))
        .send()
        .await;
    assert_eq!(bob.status, StatusCode::OK, "{:?}", bob.body);
    assert_eq!(
        ids(&alice.body),
        ["claude-opus", "claude-sonnet", "deepseek-chat"]
    );
    assert_eq!(ids(&bob.body), ["claude-sonnet", "deepseek-chat"]);
    let (_, phone) = app.phone("bob").await;
    let on_phone = app.call(Method::GET, "/models").bearer(&phone).send().await;
    assert_eq!(ids(&on_phone.body), ["claude-sonnet", "deepseek-chat"]);
}

async fn the_clients_of_the_token_reaches_the_access_rules(app: &App) {
    app.source.set_roles("carol", &[DENIED_ROLE]);
    let client = desktop_token_for_client("carol", "sp_nodeepseek");
    let answer = app
        .call(Method::GET, "/models")
        .bearer(&client)
        .send()
        .await;
    assert_eq!(ids(&answer.body), ["claude-sonnet"]);
    assert_eq!(answer.body["default"], "claude-sonnet");
}

async fn without_a_token_nothing_is_listed(app: &App) {
    let bare = app.call(Method::GET, "/models").send().await;
    assert_eq!(bare.status, StatusCode::UNAUTHORIZED);
    let foreign = app
        .call(Method::GET, "/models")
        .bearer(&crate::http_support::foreign_token("alice"))
        .send()
        .await;
    assert_eq!(foreign.status, StatusCode::UNAUTHORIZED);
    assert!(foreign.body["models"].is_null());
}

async fn an_instance_without_a_model_service_answers_503(app: &App) {
    app.source.switch_off();
    let answer = app
        .call(Method::GET, "/models")
        .bearer(&desktop_token("alice"))
        .send()
        .await;
    assert_eq!(answer.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(answer.key(), "ai_unavailable");
}

http_suite!(
    the_list_carries_what_the_phone_and_the_desktop_read,
    a_phone_reads_the_same_list,
    a_model_of_the_list_is_a_model_a_thread_and_a_prompt_accept,
    a_role_the_access_rules_deny_loses_a_model,
    the_clients_of_the_token_reaches_the_access_rules,
    without_a_token_nothing_is_listed,
    an_instance_without_a_model_service_answers_503,
);
