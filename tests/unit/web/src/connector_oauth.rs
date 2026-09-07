//! Hosted connector refresh uses provider credentials and fails closed.

use systemprompt_web_admin::connector_oauth::{Grant, Provider, refresh_at};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn grant() -> Grant {
    Grant {
        user: "fixture-user".into(),
        provider: Provider::Github,
        client: "fixture-app".into(),
        client_secret: "fixture-secret".into(),
        verifier: String::new(),
        access_token: "old".into(),
        refresh_token: Some("fixture-refresh".into()),
        expires_at: 0,
        token_endpoint: String::new(),
        generation: 0,
        session: None,
        auth_method: "oauth".into(),
        account_id: String::new(),
        account_name: String::new(),
        resource_id: String::new(),
        resource_name: String::new(),
        authorization_scheme: "Bearer".into(),
    }
}

async fn endpoint(body: &'static str) -> (String, tokio::task::JoinHandle<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        loop {
            let mut buffer = [0; 4096];
            let count = socket.read(&mut buffer).await.unwrap();
            assert!(count > 0);
            request.extend_from_slice(&buffer[..count]);
            let text = String::from_utf8_lossy(&request);
            if let Some((headers, payload)) = text.split_once("\r\n\r\n") {
                let length: usize = headers
                    .lines()
                    .find_map(|line| {
                        line.to_lowercase()
                            .strip_prefix("content-length:")
                            .map(|n| n.trim().parse().unwrap())
                    })
                    .unwrap();
                if payload.len() >= length {
                    break;
                }
            }
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).await.unwrap();
        String::from_utf8(request).unwrap()
    });
    (format!("http://{address}/token"), task)
}

#[tokio::test]
async fn refresh_sends_the_user_grant_and_preserves_rotation() {
    let (url, mock) =
        endpoint(r#"{"access_token":"fresh","refresh_token":"rotated","expires_in":3600}"#).await;
    let mut credential = grant();
    refresh_at(&mut credential, &url).await.unwrap();
    assert_eq!(credential.access_token, "fresh");
    assert_eq!(credential.refresh_token.as_deref(), Some("rotated"));
    let request = mock.await.unwrap();
    for field in [
        "grant_type=refresh_token",
        "refresh_token=fixture-refresh",
        "client_id=fixture-app",
        "client_secret=fixture-secret",
    ] {
        assert!(request.contains(field));
    }
    assert!(!request.to_lowercase().contains("authorization: bearer"));
}

#[tokio::test]
async fn revoked_grants_are_authentication_failures() {
    let (url, mock) = endpoint(r#"{"error":"invalid_grant"}"#).await;
    let error = refresh_at(&mut grant(), &url).await.unwrap_err();
    assert_eq!(error.status(), axum::http::StatusCode::UNAUTHORIZED);
    mock.await.unwrap();
}

#[tokio::test]
async fn empty_access_tokens_cannot_pass_refresh() {
    let (url, mock) = endpoint(r#"{"access_token":"","expires_in":3600}"#).await;
    assert!(refresh_at(&mut grant(), &url).await.is_err());
    mock.await.unwrap();
}

#[test]
fn atlassian_identity_accepts_wrapped_and_text_tool_results() {
    use serde_json::json;
    use systemprompt_web_admin::connector_oauth::payload::atlassian_user;
    let user = json!({"account_id":"account-123","displayName":"Pilot user"});
    for response in [
        json!({"structuredContent":user}),
        json!({"structuredContent":{"result":{"user":user}}}),
        json!({"structuredContent":{},"content":[{"type":"text","text":user.to_string()}]}),
        json!({"content":[{"type":"text","text":json!({"data":{"account":user}}).to_string()}]}),
    ] {
        assert_eq!(
            atlassian_user(&response).unwrap()["account_id"],
            "account-123"
        );
    }
    assert!(atlassian_user(&json!({"structuredContent":{"id":"site-id"}})).is_err());
    assert!(atlassian_user(&json!({"isError":true,"structuredContent":user})).is_err());
    assert!(
        atlassian_user(
            &json!({"structuredContent":user,"content":[{"text":"{\"accountId\":\"other\"}"}]})
        )
        .is_err()
    );
}

#[test]
fn atlassian_sites_accepts_wrapped_array_and_ignores_empty_structured_content() {
    use serde_json::json;
    use systemprompt_web_admin::connector_oauth::payload::atlassian_sites;
    let sites = json!([{"id":"cloud-123","url":"https://astounddigital.atlassian.net"}]);
    for response in [
        json!({"structuredContent":{"result":sites}}),
        json!({"structuredContent":{},"content":[{"text":sites.to_string()}]}),
    ] {
        assert_eq!(atlassian_sites(&response).unwrap()[0]["id"], "cloud-123");
    }
    assert!(atlassian_sites(&json!({"content":[{"text":"not JSON"}]})).is_err());
}

#[test]
fn atlassian_v2_resources_use_cloud_id_without_url() {
    use serde_json::json;
    use systemprompt_web_admin::connector_oauth::payload::atlassian_sites;
    let response = json!({"content":[{"type":"text","text":json!({"data":{"resources":[
        {"cloudId":"cloud-123","products":[{"id":"jira","access":"read"}]}
    ]}}).to_string()}],"isError":false});
    let sites = atlassian_sites(&response).unwrap();
    assert_eq!(sites[0]["id"], "cloud-123");
}

#[test]
fn atlassian_site_lookup_cannot_target_arbitrary_hosts() {
    use systemprompt_web_admin::connector_oauth::site::tenant_metadata_url;
    assert_eq!(
        tenant_metadata_url("https://astounddigital.atlassian.net")
            .unwrap()
            .as_str(),
        "https://astounddigital.atlassian.net/_edge/tenant_info"
    );
    for site in [
        "http://example.atlassian.net",
        "https://evil.test",
        "https://example.atlassian.net.evil.test",
        "https://user:secret@example.atlassian.net",
        "https://example.atlassian.net/jira",
        "https://example.atlassian.net/?token=secret",
    ] {
        assert!(tenant_metadata_url(site).is_err(), "{site}");
    }
}
