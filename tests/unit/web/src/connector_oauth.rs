//! Hosted connector refresh uses provider credentials and fails closed.

use systemprompt_web_admin_platform::connector_oauth::{Grant, Provider};

fn grant() -> Grant {
    Grant {
        configuration_binding: String::new(),
        authorization_issuer: String::new(),
        token_auth_method: String::new(),
        user: "fixture-user".into(),
        provider: Provider::Generic("fixture-mcp".into()),
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

#[test]
fn debug_redacts_connector_credentials() {
    assert_eq!(format!("{:?}", grant()), "Grant { <redacted> }");
}

// A failed connection probe is returned to the Connectors page as a report,
// rather than bubbling out as a blank error. Keep the report stateful: a
// later successful cleanup step must not make a failed handshake look green.
#[test]
fn verification_report_keeps_the_failed_stage_and_finishes_unsuccessfully() {
    use std::time::Instant;
    use systemprompt_web_admin_core::connections::report::VerificationReport;
    use systemprompt_web_admin_core::error::AdminError;

    let mut report = VerificationReport::for_provider("dynbrain");
    report.record(
        "initialize",
        Instant::now(),
        &Ok("MCP session open at portal.dynapps.es".to_owned()),
    );
    report.record(
        "tools",
        Instant::now(),
        &Err(AdminError::Upstream(
            "MCP server has no accessible tools".to_owned(),
        )),
    );
    // The caller can still record a later stage while cleaning up a failed
    // attempt; the first failure remains the outcome shown to the user.
    report.record("identity", Instant::now(), &Ok("Pilot user".to_owned()));

    let report = report.finish();
    assert!(!report.ok);
    assert_eq!(report.provider, "dynbrain");
    assert_eq!(report.steps.len(), 3);
    assert!(report.steps[0].ok);
    assert!(!report.steps[1].ok);
    assert_eq!(report.steps[1].step, "tools");
    assert_eq!(
        report.error.as_deref(),
        Some("Upstream error: MCP server has no accessible tools")
    );
}

#[test]
fn verification_report_does_not_mark_an_empty_probe_as_connected() {
    use systemprompt_web_admin_core::connections::report::VerificationReport;

    let report = VerificationReport::for_provider("dynbrain").finish();
    assert!(!report.ok);
    assert!(report.steps.is_empty());
    assert!(report.error.is_none());
}

#[test]
fn configured_provider_ids_preserve_legacy_wire_names() {
    for id in ["dynbrain", "dynapps-odoo", "fifth-mcp", "fourth-mcp"] {
        let provider: Provider = serde_json::from_value(serde_json::json!(id)).unwrap();
        assert_eq!(provider.slug(), id);
        assert_eq!(
            serde_json::to_value(provider).unwrap(),
            serde_json::json!(id)
        );
    }
    for invalid in [
        "",
        "../dynbrain",
        "https://evil.test",
        "provider?x=1",
        "a/b",
    ] {
        assert!(serde_json::from_value::<Provider>(serde_json::json!(invalid)).is_err());
    }
}

#[test]
fn generic_oauth_cannot_send_credentials_outside_approved_origins() {
    use systemprompt_web_admin_platform::connector_oauth::generic::validate_endpoint;
    let resource = "https://mcp.example.test/resource";
    let origins = vec!["https://login.example.test".to_owned()];
    for endpoint in [
        "https://mcp.example.test/.well-known/oauth-protected-resource",
        "https://login.example.test/token",
        "https://login.example.test/authorize?scope=read",
    ] {
        assert!(validate_endpoint(resource, &origins, endpoint).is_ok());
    }
    for endpoint in [
        "http://login.example.test/token",
        "https://evil.test/token",
        "https://login.example.test.evil.test/token",
        "https://secret@login.example.test/token",
        "https://login.example.test:444/token",
        "https://login.example.test/token#fragment",
    ] {
        assert!(
            validate_endpoint(resource, &origins, endpoint).is_err(),
            "{endpoint}"
        );
    }
}

#[tokio::test]
async fn gateway_does_not_allow_a_request_when_its_policy_database_is_unavailable() {
    use systemprompt::extension::{GatewayDenyKind, GatewayGuardRequest, GatewayRequestGuard};
    use systemprompt_web_admin_governance::gateway_entitlement::RouteEntitlementGuard;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://test:test@localhost:1/test")
        .unwrap();
    pool.close().await;
    let database = systemprompt::database::Database::from_pools(std::sync::Arc::new(pool), None);
    let user_id = systemprompt::identifiers::UserId::new("00000000-0000-0000-0000-000000000001");
    let model = systemprompt::identifiers::ModelId::new("test-model");
    let route = systemprompt::identifiers::RouteId::new("test-route");
    let provider = systemprompt::identifiers::ProviderId::new("test-provider");
    let attribution = systemprompt::models::attribution::RequestAttribution::none();
    let request = GatewayGuardRequest {
        user_id: &user_id,
        model: &model,
        route_id: Some(&route),
        provider: &provider,
        streaming: false,
        attribution: &attribution,
        api_key_id: None,
    };
    let denial = RouteEntitlementGuard
        .check(&database, &request)
        .await
        .unwrap_err();
    assert!(matches!(denial.kind, GatewayDenyKind::Unavailable));
    assert_eq!(denial.retry_after_seconds, 5);
    let unresolved = GatewayGuardRequest {
        route_id: None,
        ..request
    };
    assert!(
        RouteEntitlementGuard
            .check(&database, &unresolved)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn fourth_configured_connector_completes_oauth_and_mcp_verification() {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    use systemprompt_web_admin_platform::connector_oauth::{exchange_with_client, generic, verify};
    struct Server(std::process::Child);
    impl Drop for Server {
        fn drop(&mut self) {
            if self.0.kill().is_ok() {
                self.0.wait().ok();
            }
        }
    }
    let root = tempfile::tempdir().unwrap();
    let script =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/oauth/server.py");
    let mut server = Server(
        Command::new("python3")
            .arg("-u")
            .arg(script)
            .arg(root.path())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let mut origin = String::new();
    BufReader::new(server.0.stdout.take().unwrap())
        .read_line(&mut origin)
        .unwrap();
    let origin = origin.trim();
    assert!(origin.starts_with("https://localhost:"));
    let yaml = format!(
        "mcp_servers:\n  fourth-mcp:\n    server_type: external\n    endpoint: {origin}/mcp\n    enabled: true\n    display_in_web: false\n    oauth: {{required: false, scopes: [user], audience: mcp, client_id: null}}\n    connector: {{adapter: generic, scopes: [tools:read]}}\n"
    );
    let config = root.path().join("config.yaml");
    std::fs::write(&config, yaml).unwrap();
    systemprompt::loader::ServicesBootstrap::init_from_path(&config).unwrap();
    let ca = reqwest::Certificate::from_pem(&std::fs::read(root.path().join("ca.pem")).unwrap())
        .unwrap();
    let http = reqwest::Client::builder()
        .add_root_certificate(ca)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let provider = Provider::try_from("fourth-mcp".to_owned()).unwrap();
    assert!(provider.configured() && provider.requires_auth());
    let callback = "https://dashboard.example.test/api/public/connectors/fourth-mcp/callback";
    let (url, mut grant) = generic::authorize_with_client(
        generic::Consent {
            user: "active-user",
            state: "test-state",
            verifier: "test-verifier".into(),
        },
        provider,
        callback,
        &http,
    )
    .await
    .unwrap();
    let url = reqwest::Url::parse(&url).unwrap();
    let params = url
        .query_pairs()
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(params["state"], "test-state");
    assert_eq!(params["code_challenge_method"], "S256");
    assert!(!url.as_str().contains("test-verifier"));
    exchange_with_client(&mut grant, "test-code", callback, &http)
        .await
        .unwrap();
    assert_eq!(grant.access_token, "access-token");
    assert!(grant.verifier.is_empty());
    systemprompt_web_admin_platform::connector_oauth::refresh_with_client(&mut grant, &http)
        .await
        .unwrap();
    assert_eq!(grant.refresh_token.as_deref(), Some("refresh-token"));
    verify::verify_with_client(&mut grant, &http).await.unwrap();
    assert_eq!(grant.resource_name, format!("{origin}/mcp"));
    assert!(grant.account_id.is_empty());
    grant.configuration_binding = "changed-config".into();
    assert!(generic::validate_grant(&grant).is_err());
}

#[test]
fn issuer_comparison_ignores_a_single_trailing_slash_and_nothing_else() {
    use systemprompt_web_admin_platform::connector_oauth::generic::same_issuer;
    assert!(same_issuer(
        "https://accounts.google.com",
        "https://accounts.google.com/"
    ));
    assert!(same_issuer(
        "https://idp.example/tenant/",
        "https://idp.example/tenant"
    ));
    assert!(!same_issuer(
        "https://accounts.google.com",
        "https://accounts.google.com/foo"
    ));
    assert!(!same_issuer(
        "https://accounts.google.com",
        "http://accounts.google.com"
    ));
    assert!(!same_issuer(
        "https://accounts.google.com",
        "https://accounts.google.com:8443"
    ));
    assert!(!same_issuer(
        "https://idp.example",
        "https://idp.example/?x=1"
    ));
    assert!(!same_issuer("not a url", "not a url"));
}

#[tokio::test]
async fn google_shaped_connector_sends_offline_params_and_reads_userinfo() {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    use systemprompt_web_admin_platform::connector_oauth::{exchange_with_client, generic, verify};
    struct Server(std::process::Child);
    impl Drop for Server {
        fn drop(&mut self) {
            if self.0.kill().is_ok() {
                self.0.wait().ok();
            }
        }
    }
    let root = tempfile::tempdir().unwrap();
    let script =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/oauth/server.py");
    let mut server = Server(
        Command::new("python3")
            .arg("-u")
            .arg(script)
            .arg(root.path())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    let mut origin = String::new();
    BufReader::new(server.0.stdout.take().unwrap())
        .read_line(&mut origin)
        .unwrap();
    let origin = origin.trim();
    let yaml = format!(
        "mcp_servers:\n  google_rag:\n    server_type: external\n    endpoint: {origin}/mcp\n    enabled: true\n    display_in_web: false\n    oauth: {{required: false, scopes: [user], audience: mcp, client_id: null}}\n    connector:\n      adapter: generic\n      display_name: Google Drive knowledge\n      scopes: [openid, email, tools:read]\n      authorization_params: {{access_type: offline, prompt: consent}}\n      identity: userinfo\n"
    );
    let config = root.path().join("config.yaml");
    std::fs::write(&config, yaml).unwrap();
    systemprompt::loader::ServicesBootstrap::init_from_path(&config).unwrap();
    let ca = reqwest::Certificate::from_pem(&std::fs::read(root.path().join("ca.pem")).unwrap())
        .unwrap();
    let http = reqwest::Client::builder()
        .add_root_certificate(ca)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    let provider = Provider::try_from("google_rag".to_owned()).unwrap();
    assert_eq!(provider.display_name(), "Google Drive knowledge");
    let callback = "https://dashboard.example.test/api/public/connectors/google_rag/callback";
    let (url, mut grant) = generic::authorize_with_client(
        generic::Consent {
            user: "active-user",
            state: "test-state",
            verifier: "test-verifier".into(),
        },
        provider,
        callback,
        &http,
    )
    .await
    .unwrap();
    let url = reqwest::Url::parse(&url).unwrap();
    let params = url
        .query_pairs()
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(params["access_type"], "offline");
    assert_eq!(params["prompt"], "consent");
    assert_eq!(params["scope"], "openid email tools:read");
    assert_eq!(
        grant.authorization_issuer, origin,
        "the issuer is banked as the AS declares it"
    );
    exchange_with_client(&mut grant, "test-code", callback, &http)
        .await
        .unwrap();
    verify::verify_with_client(&mut grant, &http).await.unwrap();
    assert_eq!(grant.account_id, "1029384756");
    assert_eq!(grant.account_name, "sales.tester@dynapps.example");
    assert_eq!(grant.resource_name, format!("{origin}/mcp"));
}

#[tokio::test]
async fn subject_attribute_failure_is_not_an_empty_membership() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://test:test@localhost:1/test")
        .unwrap();
    pool.close().await;
    let user = systemprompt::identifiers::UserId::new("00000000-0000-0000-0000-000000000001");
    assert!(
        systemprompt_web_admin_core::authz::subject_attributes_for(&pool, &user)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn gateway_rejects_incompatible_database_handles_even_when_reported_connected() {
    use systemprompt::extension::{GatewayDenyKind, GatewayGuardRequest, GatewayRequestGuard};
    use systemprompt::identifiers::{ModelId, ProviderId, RouteId, UserId};
    use systemprompt::traits::DatabaseHandle;
    use systemprompt_web_admin_governance::gateway_entitlement::RouteEntitlementGuard;

    struct ForeignDatabase;
    impl DatabaseHandle for ForeignDatabase {
        fn is_connected(&self) -> bool {
            true
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    let user_id = UserId::new("gateway-foreign-database-consumer");
    let model = ModelId::new("test-model");
    let route = RouteId::new("test-route");
    let provider = ProviderId::new("test-provider");
    let attribution = systemprompt::models::attribution::RequestAttribution::none();
    let request = GatewayGuardRequest {
        user_id: &user_id,
        model: &model,
        route_id: Some(&route),
        provider: &provider,
        streaming: false,
        attribution: &attribution,
        api_key_id: None,
    };
    let denial = RouteEntitlementGuard
        .check(&ForeignDatabase, &request)
        .await
        .expect_err("unknown database implementations cannot authorize routes");
    assert!(matches!(denial.kind, GatewayDenyKind::Unavailable));
    assert_eq!(denial.retry_after_seconds, 5);
}
