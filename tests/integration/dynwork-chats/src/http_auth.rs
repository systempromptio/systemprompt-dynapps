//! Who may call the chats routes: the desktop JWT and the mobile JWT of a
//! live phone, mapped to the same user; everything else is `401` before a
//! path, a query or a body is read.

use axum::http::{Method, StatusCode};
use dynwork_relay::ids::DeviceId;
use serde_json::json;
use systemprompt::models::auth::JwtAudience;

use crate::http_support::{
    App, RELAY_PING, anonymous_token, desktop_token, foreign_token, http_suite, mobile_token,
    token_with_audience,
};

const THREAD: &str = "/threads/ses_01AUTHTHREAD0000000000000";

async fn refused(app: &App, authorization: Option<&str>) {
    let mut call = app.call(Method::GET, "/changes");
    if let Some(value) = authorization {
        call = call.header("authorization", value);
    }
    let answer = call.send().await;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED, "{authorization:?}");
    assert_eq!(answer.key(), "unauthorized", "{authorization:?}");
    assert!(
        answer.body["error"]["message"].is_string(),
        "{:?}",
        answer.body
    );
}

async fn no_or_malformed_bearer_is_refused(app: &App) {
    refused(app, None).await;
    refused(app, Some("Basic YWxpY2U6c2VjcmV0")).await;
    refused(app, Some("Bearer ")).await;
    refused(app, Some("Bearer not.a.jwt")).await;
    let desktop = desktop_token("alice");
    refused(app, Some(&format!("Token {desktop}"))).await;
}

async fn tokens_of_another_audience_or_deployment_are_refused(app: &App) {
    for audience in [JwtAudience::Web, JwtAudience::Mcp, JwtAudience::A2a] {
        let token = token_with_audience("alice", audience);
        refused(app, Some(&format!("Bearer {token}"))).await;
    }
    refused(app, Some(&format!("Bearer {}", foreign_token("alice")))).await;
    refused(app, Some(&format!("Bearer {}", anonymous_token("alice")))).await;
    let unknown_phone = mobile_token("alice", DeviceId::generate());
    refused(app, Some(&format!("Bearer {unknown_phone}"))).await;
}

async fn desktop_and_mobile_tokens_are_the_same_user(app: &App) {
    let desktop = desktop_token("alice");
    let (_, phone) = app.phone("alice").await;
    let created = app
        .call(Method::PUT, THREAD)
        .bearer(&desktop)
        .json(&json!({ "title": "from the desktop" }))
        .send()
        .await;
    assert_eq!(created.status, StatusCode::OK, "{:?}", created.body);

    let seen = app
        .call(Method::GET, "/changes")
        .bearer(&phone)
        .send()
        .await;
    assert_eq!(seen.status, StatusCode::OK);
    assert_eq!(seen.body["threads"][0]["title"], "from the desktop");

    let renamed = app
        .call(Method::PUT, THREAD)
        .bearer(&phone)
        .if_match("1")
        .json(&json!({ "title": "from the phone" }))
        .send()
        .await;
    assert_eq!(renamed.status, StatusCode::OK, "{:?}", renamed.body);

    let back = app
        .call(Method::GET, "/changes")
        .bearer(&desktop)
        .send()
        .await;
    assert_eq!(back.body["threads"][0]["title"], "from the phone");
    assert_eq!(back.body["threads"][0]["revision"], 2);
}

async fn a_revoked_phone_is_refused_at_once(app: &App) {
    let (device, phone) = app.phone("alice").await;
    let before = app
        .call(Method::GET, "/changes")
        .bearer(&phone)
        .send()
        .await;
    assert_eq!(before.status, StatusCode::OK);

    app.revoke("alice", device).await;
    refused(app, Some(&format!("Bearer {phone}"))).await;

    let (_, bobs) = app.phone("bob").await;
    let someone_else = mobile_token("bob", device);
    refused(app, Some(&format!("Bearer {someone_else}"))).await;
    let fine = app.call(Method::GET, "/changes").bearer(&bobs).send().await;
    assert_eq!(fine.status, StatusCode::OK);
}

async fn the_body_is_read_only_after_the_token(app: &App) {
    let anonymous = app.call(Method::PUT, THREAD).raw("{ not json").send().await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    let signed_in = app
        .call(Method::PUT, THREAD)
        .bearer(&desktop_token("alice"))
        .raw("{ not json")
        .send()
        .await;
    assert_eq!(signed_in.status, StatusCode::BAD_REQUEST);
    assert_eq!(signed_in.key(), "bad_request");
}

async fn the_chats_api_sits_beside_the_relay_mount(app: &App) {
    let relay = app.call(Method::GET, RELAY_PING).send().await;
    assert_eq!(relay.status, StatusCode::OK);
    let unknown = app
        .call(Method::GET, "/nothing-here")
        .bearer(&desktop_token("alice"))
        .send()
        .await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert_eq!(unknown.key(), "not_found");
    let chats = app
        .call(Method::GET, "/changes")
        .bearer(&desktop_token("alice"))
        .send()
        .await;
    assert_eq!(chats.status, StatusCode::OK);
    assert_eq!(
        chats
            .headers
            .get("cache-control")
            .map(axum::http::HeaderValue::as_bytes),
        Some(&b"no-store"[..])
    );
}

http_suite!(
    no_or_malformed_bearer_is_refused,
    tokens_of_another_audience_or_deployment_are_refused,
    desktop_and_mobile_tokens_are_the_same_user,
    a_revoked_phone_is_refused_at_once,
    the_body_is_read_only_after_the_token,
    the_chats_api_sits_beside_the_relay_mount,
);
