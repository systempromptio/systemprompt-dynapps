//! `GET /models`: the list the instance can serve, in the shape both clients
//! read, behind the same auth as every other route.

use axum::http::{Method, StatusCode};
use serde_json::json;

use crate::http_support::{App, desktop_token, http_suite};

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
        json!({ "models": [
            {
                "id": "claude-sonnet",
                "name": "Claude Sonnet",
                "providerID": "anthropic",
                "provider": { "id": "anthropic", "name": "Anthropic" },
                "reasoning": true,
                "tools": true
            },
            {
                "id": "deepseek-chat",
                "name": "deepseek-chat",
                "providerID": "deepseek",
                "provider": { "id": "deepseek", "name": "deepseek" },
                "reasoning": false,
                "tools": false
            }
        ] })
    );
}

async fn a_phone_reads_the_same_list(app: &App) {
    let (_, token) = app.phone("alice").await;
    let answer = app.call(Method::GET, "/models").bearer(&token).send().await;
    assert_eq!(answer.status, StatusCode::OK, "{:?}", answer.body);
    assert_eq!(answer.body["models"].as_array().map(Vec::len), Some(2));
}

async fn a_model_of_the_list_is_a_model_a_thread_and_a_prompt_accept(app: &App) {
    let listed = app
        .call(Method::GET, "/models")
        .bearer(&desktop_token("alice"))
        .send()
        .await;
    let model = &listed.body["models"][1];
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
    without_a_token_nothing_is_listed,
    an_instance_without_a_model_service_answers_503,
);
