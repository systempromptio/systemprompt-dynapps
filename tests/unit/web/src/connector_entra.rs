//! The Microsoft 365 connector on its own: the internal `microsoft-365` server
//! is a connector with the `entra` adapter, its authorization request pins the
//! tenant endpoint, PKCE S256 and the Graph scopes, its tenant secrets cannot
//! steer a URL, an ID token is admitted only for its own issuer, tenant and
//! client, and a sealed grant stops opening once its tenant leaves the
//! allow-list. The token round trips against a simulated Entra live in
//! `connector_entra_flow`.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use systemprompt::identifiers::UserId;
use systemprompt::traits::{
    ExtensionSecret, ReloadOutcome, SECRET_SEED_BYTES, SecretStore, SecretStoreError, SecretsStatus,
};
use systemprompt_web_admin_core::extension_services::ExtensionServices;
use systemprompt_web_admin_platform::connector_oauth::entra_claims::{EntraRefusal, identity};
use systemprompt_web_admin_platform::connector_oauth::entra_tenancy::EntraTenancy;
use systemprompt_web_admin_platform::connector_oauth::{
    ConnectorAccess, Consent, Grant, Provider, entra, generic, open, seal,
};

pub(crate) const TENANT_A: &str = "11111111-1111-4111-8111-111111111111";
pub(crate) const TENANT_B: &str = "22222222-2222-4222-8222-222222222222";
pub(crate) const CLIENT: &str = "entra-client";
pub(crate) const CLIENT_SECRET: &str = "entra-secret";
pub(crate) const CALLBACK: &str =
    "https://dashboard.example.test/api/public/connectors/microsoft-365/callback";
const PUBLIC_TOKEN: &str = "https://login.microsoftonline.com/organizations/oauth2/v2.0/token";

const SERVICES: &str = "mcp_servers:
  microsoft-365:
    server_type: internal
    binary: systemprompt-mcp-microsoft-365
    port: 5098
    enabled: true
    display_in_web: false
    tool_policy: allow
    oauth: {required: true, scopes: [user], audience: mcp, client_id: null}
  other-internal:
    server_type: internal
    binary: systemprompt-mcp-other
    port: 5099
    enabled: true
    display_in_web: false
    tool_policy: allow
    oauth: {required: true, scopes: [user], audience: mcp, client_id: null}
";

// The fixture secrets store with extra named secrets layered on top.
#[derive(Debug)]
pub(crate) struct Secrets {
    base: Arc<dyn SecretStore>,
    extra: HashMap<String, String>,
}

#[async_trait::async_trait]
impl SecretStore for Secrets {
    fn secret(&self, name: &str) -> Option<ExtensionSecret> {
        self.extra
            .get(name)
            .map(|value| ExtensionSecret::new(value.clone()))
            .or_else(|| self.base.secret(name))
    }
    fn key_names(&self) -> Result<Vec<String>, SecretStoreError> {
        self.base.key_names()
    }
    fn manifest_signing_secret_seed(&self) -> Result<[u8; SECRET_SEED_BYTES], SecretStoreError> {
        self.base.manifest_signing_secret_seed()
    }
    fn encryption_master_key(&self) -> Result<[u8; SECRET_SEED_BYTES], SecretStoreError> {
        self.base.encryption_master_key()
    }
    fn status(&self) -> SecretsStatus {
        self.base.status()
    }
    fn refresh_interval(&self) -> Option<Duration> {
        self.base.refresh_interval()
    }
    async fn reload(&self) -> Result<ReloadOutcome, SecretStoreError> {
        self.base.reload().await
    }
}

pub(crate) fn secrets(extra: &[(&str, &str)]) -> Secrets {
    Secrets {
        base: crate::boot_fixture::secret_store(),
        extra: extra
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect(),
    }
}

pub(crate) fn tenant_secrets(authority: Option<&str>, allowed: &str) -> Secrets {
    let mut extra = vec![
        ("microsoft_365_client_id", CLIENT),
        ("microsoft_365_client_secret", CLIENT_SECRET),
        ("microsoft_365_allowed_tenants", allowed),
    ];
    if let Some(authority) = authority {
        extra.push(("microsoft_365_authority", authority));
    }
    secrets(&extra)
}

pub(crate) fn services() -> ExtensionServices {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("config.yaml");
    std::fs::write(&path, SERVICES).unwrap();
    ExtensionServices::new(
        crate::boot_fixture::services_view(&path),
        Arc::new(systemprompt::governance::authz::SubjectProviderSet::from_providers(Vec::new())),
    )
}

pub(crate) fn provider() -> Provider {
    Provider::try_from("microsoft-365".to_owned()).unwrap()
}

pub(crate) fn authorize(
    services: &ExtensionServices,
    secrets: &Secrets,
    state: &str,
) -> (String, Grant) {
    entra::authorize(
        ConnectorAccess { services, secrets },
        Consent {
            user: "entra-user",
            state,
            verifier: "test-verifier".into(),
        },
        provider(),
        CALLBACK,
    )
    .unwrap()
}

fn query(url: &str) -> HashMap<String, String> {
    reqwest::Url::parse(url)
        .unwrap()
        .query_pairs()
        .into_owned()
        .collect()
}

fn jwt(claims: &serde_json::Value) -> String {
    let encode = |value: &serde_json::Value| URL_SAFE_NO_PAD.encode(value.to_string());
    format!(
        "{}.{}.c2ln",
        encode(&serde_json::json!({"alg": "RS256"})),
        encode(claims)
    )
}

fn claims(iss_tenant: &str, tid: &str) -> serde_json::Value {
    serde_json::json!({
        "iss": format!("https://login.microsoftonline.com/{iss_tenant}/v2.0"),
        "aud": CLIENT, "tid": tid, "oid": "user-object-id",
        "preferred_username": "ana@contoso.example",
        "exp": chrono::Utc::now().timestamp() + 3600,
    })
}

#[test]
fn entra_connector_is_synthesized_for_the_internal_microsoft_365_server() {
    let services = services();
    let config = services.config();
    let provider = provider();
    assert!(provider.configured(&config) && provider.requires_auth(&config));
    assert!(
        !provider.is_session_attested(&config),
        "it is a consent connector"
    );
    assert_eq!(provider.display_name(&config), "Microsoft 365");
    let settings = provider.settings(&config).unwrap();
    assert_eq!(settings.adapter, "entra");
    assert_eq!(
        settings.client_id_secret.as_deref(),
        Some("microsoft_365_client_id")
    );
    assert_eq!(
        settings.client_secret.as_deref(),
        Some("microsoft_365_client_secret")
    );
    for scope in [
        "offline_access",
        "openid",
        "Mail.Read",
        "Files.ReadWrite.All",
    ] {
        assert!(settings.scopes.iter().any(|s| s == scope), "{scope}");
    }
    let other = Provider::try_from("other-internal".to_owned()).unwrap();
    assert!(
        other.settings(&config).is_none(),
        "only microsoft-365 is synthesized"
    );
    assert!(
        other.is_session_attested(&config),
        "other internal servers keep the session"
    );
}

#[tokio::test]
async fn entra_authorization_url_pins_tenant_endpoint_pkce_and_graph_scopes() {
    let services = services();
    let secrets = tenant_secrets(None, TENANT_A);
    let (url, grant) = generic::authorize_with_client(
        ConnectorAccess {
            services: &services,
            secrets: &secrets,
        },
        Consent {
            user: "entra-user",
            state: "test-state",
            verifier: "test-verifier".into(),
        },
        provider(),
        CALLBACK,
        &reqwest::Client::new(),
    )
    .await
    .unwrap();
    let parsed = reqwest::Url::parse(&url).unwrap();
    assert_eq!(parsed.host_str(), Some("login.microsoftonline.com"));
    assert_eq!(parsed.path(), "/organizations/oauth2/v2.0/authorize");
    let params = query(&url);
    assert_eq!(params["client_id"], CLIENT);
    assert_eq!(params["response_type"], "code");
    assert_eq!(params["redirect_uri"], CALLBACK);
    assert_eq!(params["state"], "test-state");
    assert_eq!(params["code_challenge_method"], "S256");
    assert_eq!(
        params["code_challenge"],
        URL_SAFE_NO_PAD.encode(Sha256::digest(b"test-verifier"))
    );
    let scopes: Vec<&str> = params["scope"].split(' ').collect();
    assert!(
        ["offline_access", "openid", "Mail.Send"]
            .iter()
            .all(|s| scopes.contains(s))
    );
    assert!(
        !params.contains_key("resource"),
        "Entra v2.0 rejects resource"
    );
    assert!(!url.contains("test-verifier") && !url.contains(CLIENT_SECRET));
    assert_eq!(grant.token_endpoint, PUBLIC_TOKEN);
    assert_eq!(grant.token_auth_method, "client_secret_post");
    assert_eq!(
        (grant.client.as_str(), grant.verifier.as_str()),
        (CLIENT, "test-verifier")
    );
    assert!(grant.authorization_issuer.is_empty() && !grant.configuration_binding.is_empty());
}

#[test]
fn entra_state_is_encoded_into_the_url_not_spliced() {
    let services = services();
    let (url, _) = authorize(&services, &tenant_secrets(None, TENANT_A), "s&scope=evil#x");
    let params = query(&url);
    assert_eq!(params["state"], "s&scope=evil#x");
    assert!(params["scope"].contains("offline_access"));
    assert_eq!(reqwest::Url::parse(&url).unwrap().fragment(), None);
}

#[test]
fn entra_dedicated_tenant_goes_in_the_path_and_is_the_only_admitted_tenant() {
    let upper = TENANT_A.to_ascii_uppercase();
    let dedicated = secrets(&[("microsoft_365_tenant", upper.as_str())]);
    let tenancy = EntraTenancy::load(&dedicated).unwrap();
    assert_eq!(tenancy.tenant(), TENANT_A);
    assert!(tenancy.admits(TENANT_A) && !tenancy.admits(TENANT_B));
    assert_eq!(
        tenancy.authorize_url().path(),
        format!("/{TENANT_A}/oauth2/v2.0/authorize")
    );
    let both = format!("{TENANT_A}, {TENANT_B}");
    let shared = EntraTenancy::load(&secrets(&[(
        "microsoft_365_allowed_tenants",
        both.as_str(),
    )]))
    .unwrap();
    assert_eq!(shared.tenant(), "organizations");
    assert!(shared.admits(TENANT_A) && shared.admits(TENANT_B));
    let outside = secrets(&[
        ("microsoft_365_tenant", TENANT_A),
        ("microsoft_365_allowed_tenants", TENANT_B),
    ]);
    assert!(
        EntraTenancy::load(&outside).is_err(),
        "a dedicated tenant must be admitted"
    );
}

#[test]
fn entra_tenant_secrets_cannot_steer_the_endpoint() {
    for tenant in [
        "common",
        "consumers",
        "evil.example/x",
        "organizations?x=1",
        "../organizations",
    ] {
        let store = secrets(&[
            ("microsoft_365_tenant", tenant),
            ("microsoft_365_allowed_tenants", TENANT_A),
        ]);
        assert!(EntraTenancy::load(&store).is_err(), "{tenant}");
    }
    for authority in [
        "http://login.microsoftonline.com",
        "https://login.example.test/tenant",
        "https://user@login.example.test",
        "https://login.example.test/?x=1",
    ] {
        let store = secrets(&[
            ("microsoft_365_authority", authority),
            ("microsoft_365_allowed_tenants", TENANT_A),
        ]);
        assert!(EntraTenancy::load(&store).is_err(), "{authority}");
    }
    assert!(
        EntraTenancy::load(&secrets(&[])).is_err(),
        "shared mode needs an allow-list"
    );
    let junk = secrets(&[("microsoft_365_allowed_tenants", "contoso.example")]);
    assert!(EntraTenancy::load(&junk).is_err());
}

#[test]
fn entra_id_token_must_match_issuer_tenant_and_client() {
    let now = chrono::Utc::now().timestamp();
    let check =
        |claims: &serde_json::Value, endpoint: &str| identity(&jwt(claims), endpoint, CLIENT, now);
    let admitted = check(&claims(TENANT_A, TENANT_A), PUBLIC_TOKEN).unwrap();
    assert_eq!(admitted.tenant, TENANT_A);
    assert_eq!(
        (admitted.account.as_str(), admitted.label.as_str()),
        ("user-object-id", "ana@contoso.example")
    );
    let upper_client = CLIENT.to_ascii_uppercase();
    let any_case = identity(
        &jwt(&claims(TENANT_A, TENANT_A)),
        PUBLIC_TOKEN,
        &upper_client,
        now,
    );
    assert!(any_case.is_ok(), "a GUID client id compares without case");
    assert_eq!(
        check(&claims(TENANT_B, TENANT_A), PUBLIC_TOKEN),
        Err(EntraRefusal::Issuer)
    );
    let mut foreign_host = claims(TENANT_A, TENANT_A);
    foreign_host["iss"] = format!("https://evil.example/{TENANT_A}/v2.0").into();
    assert_eq!(
        check(&foreign_host, PUBLIC_TOKEN),
        Err(EntraRefusal::Issuer)
    );
    let mut other_app = claims(TENANT_A, TENANT_A);
    other_app["aud"] = "another-app".into();
    assert_eq!(check(&other_app, PUBLIC_TOKEN), Err(EntraRefusal::Audience));
    let dedicated = format!("https://login.microsoftonline.com/{TENANT_A}/oauth2/v2.0/token");
    assert_eq!(
        check(&claims(TENANT_B, TENANT_B), &dedicated),
        Err(EntraRefusal::Tenant)
    );
    assert_eq!(
        check(&claims("contoso", "contoso"), PUBLIC_TOKEN),
        Err(EntraRefusal::Tenant)
    );
    let mut expired = claims(TENANT_A, TENANT_A);
    expired["exp"] = (now - 3600).into();
    assert_eq!(check(&expired, PUBLIC_TOKEN), Err(EntraRefusal::Expired));
    let mut anonymous = claims(TENANT_A, TENANT_A);
    anonymous.as_object_mut().unwrap().remove("oid");
    assert_eq!(check(&anonymous, PUBLIC_TOKEN), Err(EntraRefusal::Subject));
    for garbage in ["not-a-jwt", "a.b.c.d.e"] {
        assert_eq!(
            identity(garbage, PUBLIC_TOKEN, CLIENT, now),
            Err(EntraRefusal::Undecodable)
        );
    }
}

#[test]
fn entra_grant_stops_opening_when_its_tenant_leaves_the_allow_list() {
    let services = services();
    let admitted = tenant_secrets(None, TENANT_A);
    let user = UserId::new("entra-user");
    let (_, pending) = authorize(&services, &admitted, "test-state");
    let sealed = seal(&admitted, &pending).unwrap();
    assert!(
        open(&services, &admitted, &sealed, &user, &provider()).is_ok(),
        "pending consent opens"
    );
    let (_, mut connected) = authorize(&services, &admitted, "test-state");
    connected.access_token = "graph-access".into();
    connected.resource_id = TENANT_A.into();
    let sealed = seal(&admitted, &connected).unwrap();
    let rotated = secrets(&[
        ("microsoft_365_client_id", CLIENT),
        ("microsoft_365_client_secret", "rotated-secret"),
        ("microsoft_365_allowed_tenants", TENANT_A),
    ]);
    let opened = open(&services, &rotated, &sealed, &user, &provider()).unwrap();
    assert_eq!(
        opened.client_secret, "rotated-secret",
        "the current app secret is used"
    );
    let removed = tenant_secrets(None, TENANT_B);
    assert!(matches!(
        open(&services, &removed, &sealed, &user, &provider()),
        Err(systemprompt_web_admin_core::error::AdminError::Unauthorized(_))
    ));
    let other_app = secrets(&[
        ("microsoft_365_client_id", "another-app"),
        ("microsoft_365_client_secret", CLIENT_SECRET),
        ("microsoft_365_allowed_tenants", TENANT_A),
    ]);
    assert!(open(&services, &other_app, &sealed, &user, &provider()).is_err());
}

#[test]
fn entra_connector_card_reads_as_microsoft_365() {
    use systemprompt_web_admin_core::connections::Connection;
    let card = systemprompt_web_admin_platform::test_support::connector_card(
        &services(),
        &Connection {
            provider: "microsoft-365".into(),
            display_name: "Microsoft 365".into(),
            requires_auth: true,
            session_attested: false,
            configured: true,
            entitled: true,
            status: "not_connected".into(),
            auth_method: None,
            account_id: None,
            account_name: None,
            resource_id: None,
            resource_name: None,
            error_code: None,
            verified_at: None,
            actions: vec!["connect".into()],
        },
    );
    assert_eq!(
        (card.glyph, card.kind, card.monogram.as_str()),
        ("microsoft", "Microsoft 365", "M")
    );
    assert!(card.blurb.contains("Microsoft 365"));
}
