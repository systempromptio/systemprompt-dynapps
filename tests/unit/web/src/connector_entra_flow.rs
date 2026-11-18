//! The Microsoft 365 connector against a simulated Entra
//! (`tests/fixtures/oauth/entra.py`, loopback TLS): the exchange records the
//! tenant and user from the ID token and the token request never carries
//! `resource`; a refresh keeps the same tenant and user; a foreign tenant, a
//! forged issuer, a missing ID token, a personal Microsoft account or a
//! refresh that switches tenant is refused with its tokens wiped, and the
//! refused tenant id is logged; and the generic exchange, which has no
//! allow-list, refuses an Entra grant outright.

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};

use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::connections::report::VerificationReport;
use systemprompt_web_admin_core::error::AdminError;
use systemprompt_web_admin_platform::connector_oauth::{
    ConnectorAccess, Grant, entra, exchange_with_client, open, refresh_with_client, seal, verify,
};

use crate::connector_entra::{
    CALLBACK, Capture, PERSONAL, Secrets, TENANT_A, TENANT_B, authorize, provider, services,
    tenant_secrets,
};

struct Entra {
    child: Child,
    origin: String,
    http: reqwest::Client,
    _root: tempfile::TempDir,
}

impl Drop for Entra {
    fn drop(&mut self) {
        if self.child.kill().is_ok() {
            self.child.wait().ok();
        }
    }
}

fn simulated_entra() -> Entra {
    let root = tempfile::tempdir().unwrap();
    let script =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/oauth/entra.py");
    let mut child = Command::new("python3")
        .arg("-u")
        .arg(script)
        .arg(root.path())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut origin = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut origin)
        .unwrap();
    let origin = origin.trim().to_owned();
    assert!(origin.starts_with("https://localhost:"), "{origin}");
    let ca = reqwest::Certificate::from_pem(&std::fs::read(root.path().join("ca.pem")).unwrap())
        .unwrap();
    let http = reqwest::Client::builder()
        .add_root_certificate(ca)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();
    Entra {
        child,
        origin,
        http,
        _root: root,
    }
}

fn wiped(grant: &Grant) -> bool {
    grant.access_token.is_empty() && grant.refresh_token.is_none()
}

async fn exchanged(
    entra: &Entra,
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    secrets: &Secrets,
    code: &str,
) -> (Grant, Result<(), AdminError>) {
    let (_, mut grant) = authorize(services, secrets, "test-state");
    assert!(grant.token_endpoint.starts_with(&entra.origin));
    let outcome = entra::exchange_with_client(
        ConnectorAccess { services, secrets },
        &mut grant,
        code,
        CALLBACK,
        &entra.http,
    )
    .await;
    (grant, outcome)
}

#[tokio::test]
async fn entra_exchange_records_the_tenant_and_refresh_keeps_it() {
    let entra = simulated_entra();
    let services = services();
    let secrets = tenant_secrets(Some(&entra.origin), TENANT_A);
    let (mut grant, outcome) = exchanged(&entra, &services, &secrets, "code-allowed").await;
    outcome.unwrap();
    assert_eq!(grant.access_token, "graph-access-1");
    assert_eq!(grant.refresh_token.as_deref(), Some("refresh-a"));
    assert!(grant.verifier.is_empty(), "the PKCE verifier is spent");
    assert!(grant.expires_at > chrono::Utc::now().timestamp());
    assert_eq!(grant.resource_id, TENANT_A);
    assert_eq!(grant.account_id, "0000aaaa-0000-4000-8000-00000000aaaa");
    assert_eq!(grant.account_name, "ana@contoso.example");
    assert_eq!(grant.resource_name, "https://graph.microsoft.com");
    refresh_with_client(&services, &mut grant, &entra.http)
        .await
        .unwrap();
    assert_eq!(grant.access_token, "graph-access-2");
    assert_eq!(
        grant.refresh_token.as_deref(),
        Some("refresh-a2"),
        "rotation is kept"
    );
    assert_eq!(grant.resource_id, TENANT_A);
    let mut report = VerificationReport::for_provider("microsoft-365");
    verify::verify_reporting(&services, &mut grant, &entra.http, &mut report)
        .await
        .unwrap();
    let report = report.finish();
    assert!(report.ok);
    assert_eq!(report.steps[0].step, "identity");
    assert!(report.steps[0].detail.contains("ana@contoso.example"));
}

#[tokio::test]
async fn entra_exchange_refuses_a_tenant_outside_the_allow_list() {
    let entra = simulated_entra();
    let services = services();
    let secrets = tenant_secrets(Some(&entra.origin), TENANT_A);
    let (grant, outcome) = exchanged(&entra, &services, &secrets, "code-other-tenant").await;
    let error = outcome.unwrap_err().to_string();
    assert!(error.contains("not allowed on this instance"), "{error}");
    assert!(wiped(&grant), "nothing of a refused tenant can be stored");
}

#[tokio::test]
async fn entra_exchange_refusal_logs_the_refused_tenant_id() {
    let entra = simulated_entra();
    let services = services();
    let secrets = tenant_secrets(Some(&entra.origin), TENANT_A);
    let capture = Capture::default();
    let scope = tracing::subscriber::set_default(capture.clone());
    let (grant, outcome) = exchanged(&entra, &services, &secrets, "code-other-tenant").await;
    drop(scope);
    assert!(outcome.is_err() && wiped(&grant));
    assert_eq!(
        capture.refused_tenant("tenant_not_allowed").as_deref(),
        Some(TENANT_B),
        "operators can tell which tenant tried"
    );
    capture.assert_never_logged(&["graph-access", "refresh-b", "ana@contoso.example"]);
}

#[tokio::test]
async fn entra_exchange_refuses_a_personal_microsoft_account() {
    let entra = simulated_entra();
    let services = services();
    let secrets = tenant_secrets(Some(&entra.origin), TENANT_A);
    let capture = Capture::default();
    let scope = tracing::subscriber::set_default(capture.clone());
    let (grant, outcome) = exchanged(&entra, &services, &secrets, "code-personal").await;
    drop(scope);
    let error = outcome.unwrap_err().to_string();
    assert!(error.contains("personal Microsoft accounts"), "{error}");
    assert!(wiped(&grant) && grant.resource_id.is_empty());
    assert_eq!(
        capture.refused_tenant("personal_account").as_deref(),
        Some(PERSONAL)
    );
    capture.assert_never_logged(&["graph-access", "refresh-personal", "ana@contoso.example"]);
}

#[tokio::test]
async fn entra_exchange_refuses_an_issuer_that_does_not_match_the_tenant() {
    let entra = simulated_entra();
    let services = services();
    let secrets = tenant_secrets(Some(&entra.origin), TENANT_A);
    let (grant, outcome) = exchanged(&entra, &services, &secrets, "code-forged-issuer").await;
    assert!(outcome.is_err());
    assert!(wiped(&grant) && grant.resource_id.is_empty());
    let (grant, outcome) = exchanged(&entra, &services, &secrets, "code-no-id-token").await;
    assert!(
        outcome.is_err(),
        "an exchange without an ID token names no tenant"
    );
    assert!(wiped(&grant));
    let (grant, outcome) = exchanged(&entra, &services, &secrets, "code-no-token-type").await;
    assert!(outcome.is_err(), "a malformed token response is refused");
    assert!(wiped(&grant), "an unvetted refresh token is never kept");
}

#[tokio::test]
async fn entra_refresh_that_switches_tenant_is_refused() {
    let entra = simulated_entra();
    let services = services();
    let secrets = tenant_secrets(Some(&entra.origin), TENANT_A);
    let (mut grant, outcome) = exchanged(&entra, &services, &secrets, "code-allowed").await;
    outcome.unwrap();
    grant.refresh_token = Some("refresh-switch".into());
    let error = refresh_with_client(&services, &mut grant, &entra.http)
        .await
        .unwrap_err()
        .to_string();
    assert!(error.contains("another tenant or user"), "{error}");
    assert!(wiped(&grant));
}

#[tokio::test]
async fn generic_exchange_refuses_an_entra_grant_without_the_allow_list() {
    let entra = simulated_entra();
    let services = services();
    let secrets = tenant_secrets(Some(&entra.origin), TENANT_A);
    let (_, mut grant) = authorize(&services, &secrets, "test-state");
    let outcome =
        exchange_with_client(&services, &mut grant, "code-allowed", CALLBACK, &entra.http).await;
    assert!(outcome.is_err());
    assert!(wiped(&grant));
}

#[tokio::test]
async fn entra_refresh_without_id_token_keeps_the_recorded_identity() {
    let entra = simulated_entra();
    let services = services();
    let secrets = tenant_secrets(Some(&entra.origin), TENANT_A);
    let (mut grant, outcome) = exchanged(&entra, &services, &secrets, "code-allowed").await;
    outcome.unwrap();
    let account = grant.account_id.clone();
    grant.refresh_token = Some("refresh-no-id-token".into());
    refresh_with_client(&services, &mut grant, &entra.http)
        .await
        .unwrap();
    assert_eq!(
        grant.access_token, "graph-access-2",
        "the refreshed token is kept"
    );
    assert_eq!(grant.refresh_token.as_deref(), Some("refresh-a2"));
    assert_eq!(
        (grant.resource_id.as_str(), grant.account_id.as_str()),
        (TENANT_A, account.as_str())
    );
    assert_eq!(grant.account_name, "ana@contoso.example");
}

#[tokio::test]
async fn entra_tenant_removed_after_a_refresh_without_id_token_gets_no_token() {
    let entra = simulated_entra();
    let services = services();
    let admitted = tenant_secrets(Some(&entra.origin), TENANT_A);
    let (mut grant, outcome) = exchanged(&entra, &services, &admitted, "code-allowed").await;
    outcome.unwrap();
    grant.refresh_token = Some("refresh-no-id-token".into());
    refresh_with_client(&services, &mut grant, &entra.http)
        .await
        .unwrap();
    let sealed = seal(&admitted, &grant).unwrap();
    let user = UserId::new("entra-user");
    let reopened = open(&services, &admitted, &sealed, &user, &provider()).unwrap();
    assert_eq!(
        reopened.resource_id, TENANT_A,
        "the recorded tenant survives the reseal"
    );
    let removed = tenant_secrets(Some(&entra.origin), TENANT_B);
    assert!(
        matches!(
            open(&services, &removed, &sealed, &user, &provider()),
            Err(AdminError::Unauthorized(_))
        ),
        "the broker opens the grant before handing out a token, so a removed tenant gets none"
    );
}
