//! Regression coverage for shared OAuth failures, rotation and callback
//! notices.

use super::connector_oauth::{endpoint, endpoint_with_status, grant};
use axum::http::StatusCode;
use systemprompt_web_admin_platform::connector_oauth::{Provider, refresh_at};

#[tokio::test]
async fn application_credentials_fail_without_claiming_the_user_grant_was_revoked() {
    for code in ["invalid_client", "unauthorized_client", "invalid_scope"] {
        let body = match code {
            "invalid_client" => {
                r#"{"error":"invalid_client","error_description":"fixture-secret fixture-refresh"}"#
            },
            "unauthorized_client" => r#"{"error":"unauthorized_client"}"#,
            _ => r#"{"error":"invalid_scope"}"#,
        };
        let (url, mock) = endpoint_with_status(400, "Bad Request", body).await;
        let mut saved = grant();
        saved.provider = Provider::try_from("salesforce".to_owned()).unwrap();
        let error = refresh_at(&mut saved, &url).await.unwrap_err();
        assert_eq!(error.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(error.to_string().contains("app configuration"));
        assert!(!error.to_string().contains("fixture-secret"));
        assert!(!error.to_string().contains("fixture-refresh"));
        assert_eq!(saved.refresh_token.as_deref(), Some("fixture-refresh"));
        mock.await.unwrap();
    }
}

#[tokio::test]
async fn an_unrecognized_error_code_cannot_echo_credentials() {
    let (url, mock) = endpoint_with_status(
        400,
        "Bad Request",
        r#"{"error":"fixture-secret","error_description":"fixture-refresh"}"#,
    )
    .await;
    let error = refresh_at(&mut grant(), &url).await.unwrap_err();
    assert_eq!(error.status(), StatusCode::BAD_GATEWAY);
    assert!(!error.to_string().contains("fixture-"));
    mock.await.unwrap();
}

#[tokio::test]
async fn salesforce_concurrent_processing_is_an_outage_not_a_revocation() {
    let (url, mock) = endpoint_with_status(400, "Bad Request", r#"{"error":"invalid_grant","error_description":"Token request is already being processed"}"#).await;
    let mut saved = grant();
    saved.provider = Provider::try_from("salesforce".to_owned()).unwrap();
    assert_eq!(
        refresh_at(&mut saved, &url).await.unwrap_err().status(),
        StatusCode::BAD_GATEWAY
    );
    assert_eq!(saved.refresh_token.as_deref(), Some("fixture-refresh"));
    mock.await.unwrap();
}

#[tokio::test]
async fn expired_salesforce_jwts_are_not_extended_and_rotation_is_preserved() {
    for body in [
        r#"{"access_token":"header.eyJleHAiOjF9.signature","refresh_token":"replacement"}"#,
        r#"{"access_token":"header.eyJleHAiOjF9.signature","refresh_token":"replacement","expires_in":3600}"#,
    ] {
        let (url, mock) = endpoint(body).await;
        let mut saved = grant();
        saved.provider = Provider::try_from("salesforce".to_owned()).unwrap();
        assert_eq!(
            refresh_at(&mut saved, &url).await.unwrap_err().status(),
            StatusCode::BAD_GATEWAY
        );
        assert!(saved.expires_at <= chrono::Utc::now().timestamp());
        assert_eq!(saved.refresh_token.as_deref(), Some("replacement"));
        mock.await.unwrap();
    }
}

#[tokio::test]
async fn malformed_access_tokens_keep_the_replacement_refresh_token() {
    for body in [
        r#"{"access_token":"","refresh_token":"replacement","expires_in":3600}"#,
        r#"{"access_token":"new","token_type":"MAC","refresh_token":"replacement","expires_in":3600}"#,
        r#"{"access_token":"new","refresh_token":"replacement","expires_in":-1}"#,
    ] {
        let (url, mock) = endpoint(body).await;
        let mut saved = grant();
        assert_eq!(
            refresh_at(&mut saved, &url).await.unwrap_err().status(),
            StatusCode::BAD_GATEWAY
        );
        assert_eq!(saved.refresh_token.as_deref(), Some("replacement"));
        assert_eq!(saved.expires_at, 0);
        mock.await.unwrap();
    }
}

#[tokio::test]
async fn nonrotating_refreshes_keep_the_existing_refresh_token() {
    let (url, mock) = endpoint(r#"{"access_token":"new","expires_in":3600}"#).await;
    let mut saved = grant();
    refresh_at(&mut saved, &url).await.unwrap();
    assert_eq!(saved.refresh_token.as_deref(), Some("fixture-refresh"));
    mock.await.unwrap();
}

#[tokio::test]
async fn empty_replacement_tokens_and_unreadable_successes_require_reconnect_without_replay() {
    for body in [
        r#"{"access_token":"new","refresh_token":" ","expires_in":3600}"#,
        "not-json",
    ] {
        let (url, mock) = endpoint(body).await;
        let mut saved = grant();
        let error = refresh_at(&mut saved, &url).await.unwrap_err();
        assert_eq!(error.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(error.to_string().contains("may have rotated"));
        assert_eq!(saved.refresh_token.as_deref(), Some("fixture-refresh"));
        mock.await.unwrap();
    }
}

#[test]
fn callback_notices_name_the_stage_and_only_accept_bounded_fields() {
    use systemprompt_web_admin_platform::connector_oauth::callback_notice::CallbackNotice;
    for stage in ["token", "initialize", "tools", "identity"] {
        let notice: CallbackNotice = serde_json::from_value(serde_json::json!({
            "connection_provider": "salesforce", "connection_stage": stage,
            "connection_failure": "configuration", "connection_ref": "00000000-0000-0000-0000-000000000001"
        })).unwrap();
        let message = notice.message().unwrap();
        assert!(message.contains("failed during") && message.contains("Reference:"));
        assert!(message.contains("app configuration"));
    }
    let notice: CallbackNotice = serde_json::from_value(serde_json::json!({
        "connection_provider": "salesforce", "connection_stage": "fixture-secret",
        "connection_failure": "configuration", "connection_ref": "00000000-0000-0000-0000-000000000001"
    })).unwrap();
    assert!(notice.message().is_none());
}
