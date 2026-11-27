//! Who may call the Brain routes: the desktop JWT and the mobile JWT of a
//! live phone, mapped to the same person; everything else is `401` before a
//! path, a query or a body is read.

use axum::http::{Method, StatusCode};
use dynwork_relay::ids::DeviceId;
use systemprompt::models::auth::JwtAudience;

use crate::http_support::{
    App, anonymous_token, desktop_token, foreign_token, mobile_token, token_with_audience,
};

async fn refused(app: &App, bearer: Option<&str>) {
    let answer = app.request(Method::GET, "/spaces", bearer, None).await;
    assert_eq!(answer.status, StatusCode::UNAUTHORIZED, "{bearer:?}");
    assert_eq!(answer.key(), "unauthorized", "{bearer:?}");
    assert!(answer.body["error"]["message"].is_string());
}

#[tokio::test]
async fn no_bearer_or_a_malformed_one_is_refused() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    refused(&app, None).await;
    refused(&app, Some("")).await;
    refused(&app, Some("not.a.jwt")).await;
    app.finish().await;
}

#[tokio::test]
async fn tokens_of_another_audience_deployment_or_kind_are_refused() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    for audience in [JwtAudience::Web, JwtAudience::Mcp, JwtAudience::A2a] {
        refused(&app, Some(&token_with_audience("alice", audience))).await;
    }
    refused(&app, Some(&foreign_token("alice"))).await;
    refused(&app, Some(&anonymous_token("alice"))).await;
    refused(&app, Some(&mobile_token("alice", DeviceId::generate()))).await;
    app.finish().await;
}

#[tokio::test]
async fn the_desktop_token_opens_the_routes_as_its_person() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let answer = app
        .request(Method::GET, "/spaces", Some(&desktop_token("alice")), None)
        .await;
    assert_eq!(answer.status, StatusCode::OK);
    assert_eq!(answer.body["total"], 0);
    app.finish().await;
}

#[tokio::test]
async fn a_live_phone_enters_as_the_same_person_and_a_revoked_one_does_not() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let space = app.fixture.space("handbook").await;
    let (device, token) = app.phone("alice").await;
    let seen = app
        .request(Method::GET, "/spaces", Some(&token), None)
        .await;
    assert_eq!(seen.status, StatusCode::OK, "{:?}", seen.body);
    assert_eq!(seen.body["spaces"][0]["space"], space.id.0);
    app.revoke(device).await;
    refused(&app, Some(&token)).await;
    app.finish().await;
}

#[tokio::test]
async fn an_unknown_route_or_a_bad_id_is_a_json_not_found() {
    let db = dynapps_test_common::db_or_skip!();
    let app = App::new(db).await;
    let answer = app.get("/nowhere", "alice").await;
    assert_eq!(answer.status, StatusCode::NOT_FOUND);
    assert_eq!(answer.key(), "not_found");
    let bad_id = app.get("/articles/not-a-number", "alice").await;
    assert_eq!(bad_id.status, StatusCode::NOT_FOUND);
    app.finish().await;
}
