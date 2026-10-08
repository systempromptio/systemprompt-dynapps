//! Salesforce's numeric string expiry is normalized only at its token boundary.
use super::connector_oauth::{endpoint, grant};
use systemprompt_web_admin_platform::connector_oauth::{
    Provider, exchange_with_client, refresh_with_client,
};

async fn response(body: &'static str, refreshing: bool, salesforce: bool) -> Result<i64, String> {
    let (url, mock) = endpoint(body).await;
    let mut credential = grant();
    if salesforce {
        credential.provider = Provider::try_from("salesforce".to_owned()).unwrap();
    }
    credential.token_endpoint = url;
    let http = reqwest::Client::new();
    let result = if refreshing {
        refresh_with_client(&mut credential, &http).await
    } else {
        exchange_with_client(
            &mut credential,
            "fixture-code",
            "http://localhost/callback",
            &http,
        )
        .await
    };
    mock.await.unwrap();
    result
        .map(|()| credential.expires_at)
        .map_err(|error| error.to_string())
}

#[tokio::test]
async fn salesforce_exchange_and_refresh_accept_number_and_string_expiry() {
    for refreshing in [false, true] {
        for body in [
            r#"{"access_token":"fixture-access","refresh_token":"fixture-refresh","token_type":"Bearer","expires_in":3600,"extra":"ignored"}"#,
            r#"{"access_token":"fixture-access","refresh_token":"fixture-refresh","token_type":"Bearer","expires_in":"3600","extra":"ignored"}"#,
        ] {
            let start = chrono::Utc::now().timestamp();
            let expiry = response(body, refreshing, true).await.unwrap();
            assert!((start + 3599..=chrono::Utc::now().timestamp() + 3600).contains(&expiry));
        }
    }
}

#[tokio::test]
async fn salesforce_rejects_invalid_declared_expiry() {
    for body in [
        r#"{"access_token":"fresh","refresh_token":"rotated","expires_in":"nope"}"#,
        r#"{"access_token":"fresh","refresh_token":"rotated","expires_in":"0"}"#,
        r#"{"access_token":"fresh","refresh_token":"rotated","expires_in":"-1"}"#,
        r#"{"access_token":"fresh","refresh_token":"rotated","expires_in":"31536001"}"#,
        r#"{"access_token":"fresh","refresh_token":"rotated","expires_in":"9223372036854775808"}"#,
        r#"{"access_token":"fresh","refresh_token":"rotated","expires_in":true}"#,
        r#"{"access_token":"fresh","refresh_token":"rotated","expires_in":1.5}"#,
        r#"{"access_token":"fresh","refresh_token":"rotated","expires_in":"1.5"}"#,
    ] {
        for refreshing in [false, true] {
            assert!(response(body, refreshing, true).await.is_err(), "{body}");
        }
    }
}

#[tokio::test]
async fn missing_salesforce_expiry_retains_short_opaque_token_lifetime() {
    let start = chrono::Utc::now().timestamp();
    let expiry = response(
        r#"{"access_token":"fresh","refresh_token":"rotated"}"#,
        false,
        true,
    )
    .await
    .unwrap();
    assert!((start + 899..=chrono::Utc::now().timestamp() + 900).contains(&expiry));
}

#[tokio::test]
async fn other_providers_keep_strict_numeric_expiry() {
    assert!(
        response(
            r#"{"access_token":"fresh","refresh_token":"rotated","expires_in":"3600"}"#,
            false,
            false
        )
        .await
        .is_err()
    );
}
