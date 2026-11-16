//! Lenient OAuth discovery accepts exactly the dynapps-365 shape — a resource
//! that is the bare origin and an `authorization_servers` entry that is the
//! metadata document URL — on the endpoint's own origin, and strict (the
//! default) rejects it. Issuer, PKCE and origin checks hold in both modes.

use reqwest::Url;
use systemprompt_web_admin_core::repositories::connector_discovery::{
    CONNECTORS_FILE, ConnectorDiscoveryPolicy, DiscoveryMode,
};
use systemprompt_web_admin_platform::connector_oauth::Provider;
use systemprompt_web_admin_platform::connector_oauth::generic::{
    advertised_metadata_document, resource_matches,
};

use DiscoveryMode::{Lenient, Strict};

const ENDPOINT: &str = "https://dynapps-365.example.test/mcp";
const METADATA: &str = "https://dynapps-365.example.test/.well-known/oauth-authorization-server";

fn endpoint() -> Url {
    Url::parse(ENDPOINT).unwrap()
}

fn url(value: &str) -> Url {
    Url::parse(value).unwrap()
}

#[test]
fn exact_resource_matches_in_both_modes() {
    for mode in [Strict, Lenient] {
        assert!(resource_matches(ENDPOINT, &endpoint(), mode));
    }
}

#[test]
fn bare_origin_resource_is_a_mismatch_only_in_strict_mode() {
    for origin in [
        "https://dynapps-365.example.test",
        "https://dynapps-365.example.test/",
        "https://dynapps-365.example.test:443",
    ] {
        assert!(!resource_matches(origin, &endpoint(), Strict), "{origin}");
        assert!(resource_matches(origin, &endpoint(), Lenient), "{origin}");
    }
}

#[test]
fn lenient_resource_still_rejects_anything_but_the_endpoint_or_its_origin() {
    let nested = url("https://dynapps-365.example.test/tenant/mcp");
    assert!(!resource_matches(
        "https://dynapps-365.example.test/tenant",
        &nested,
        Lenient
    ));
    for resource in [
        "https://other.example.test",
        "https://other.example.test/mcp",
        "http://dynapps-365.example.test",
        "https://dynapps-365.example.test:8443",
        "https://dynapps-365.example.test/other",
        "https://dynapps-365.example.test/mcp/extra",
        "https://dynapps-365.example.test/?x=1",
        "https://dynapps-365.example.test/#frag",
        "https://user@dynapps-365.example.test",
        "not a url",
    ] {
        assert!(
            !resource_matches(resource, &endpoint(), Lenient),
            "{resource}"
        );
    }
}

#[test]
fn metadata_document_entry_is_an_issuer_in_strict_mode() {
    assert_eq!(
        advertised_metadata_document(&url(METADATA), &endpoint(), Strict),
        None
    );
}

#[test]
fn lenient_reads_a_same_origin_metadata_document_with_the_origin_as_issuer() {
    let document = advertised_metadata_document(&url(METADATA), &endpoint(), Lenient).unwrap();
    assert_eq!(document.url.as_str(), METADATA);
    assert!(
        systemprompt_web_admin_platform::connector_oauth::generic::same_issuer(
            "https://dynapps-365.example.test",
            &document.issuer
        )
    );

    let openid = url("https://dynapps-365.example.test/.well-known/openid-configuration");
    let document = advertised_metadata_document(&openid, &endpoint(), Lenient).unwrap();
    assert_eq!(document.url, openid);
    assert_eq!(document.issuer, "https://dynapps-365.example.test/");

    let inserted =
        url("https://dynapps-365.example.test/.well-known/oauth-authorization-server/tenant");
    let document = advertised_metadata_document(&inserted, &endpoint(), Lenient).unwrap();
    assert_eq!(document.issuer, "https://dynapps-365.example.test/tenant");
}

#[test]
fn lenient_reads_any_other_entry_as_an_issuer() {
    for entry in [
        "https://login.example.test/.well-known/oauth-authorization-server",
        "https://dynapps-365.example.test:8443/.well-known/oauth-authorization-server",
        "https://dynapps-365.example.test/.well-known/oauth-authorization-server?x=1",
        "https://dynapps-365.example.test/.well-known/oauth-authorization-server/",
        "https://dynapps-365.example.test/.well-known/oauth-authorization-serverx",
        "https://dynapps-365.example.test/tenant/.well-known/oauth-authorization-server",
        "https://dynapps-365.example.test/",
        "https://dynapps-365.example.test/tenant",
    ] {
        assert_eq!(
            advertised_metadata_document(&url(entry), &endpoint(), Lenient),
            None,
            "{entry}"
        );
    }
}

#[test]
fn every_connector_is_strict_unless_listed_as_lenient() {
    assert_eq!(
        ConnectorDiscoveryPolicy::default().mode("dynapps-365"),
        Strict
    );
    let policy = ConnectorDiscoveryPolicy::from_yaml(
        "connectors:\n  dynapps-365:\n    discovery: lenient\n  dynbrain:\n    discovery: strict\n  plain: {}\n",
    )
    .unwrap();
    assert_eq!(policy.mode("dynapps-365"), Lenient);
    assert_eq!(policy.mode("dynbrain"), Strict);
    assert_eq!(policy.mode("plain"), Strict);
    assert_eq!(policy.mode("unlisted"), Strict);
    assert_eq!(
        ConnectorDiscoveryPolicy::from_yaml("").unwrap(),
        ConnectorDiscoveryPolicy::default()
    );
}

#[test]
fn unknown_discovery_modes_and_keys_are_refused() {
    for yaml in [
        "connectors:\n  dynapps-365:\n    discovery: permissive\n",
        "connectors:\n  dynapps-365:\n    discovery: lenient\n    resource: any\n",
        "connector:\n  dynapps-365:\n    discovery: lenient\n",
    ] {
        assert!(ConnectorDiscoveryPolicy::from_yaml(yaml).is_err(), "{yaml}");
    }
}

#[test]
fn an_unparseable_policy_file_leaves_every_connector_strict() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join(CONNECTORS_FILE);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "connectors:\n  dynapps-365:\n    discovery: yes\n").unwrap();
    assert!(ConnectorDiscoveryPolicy::load(root.path()).is_err());
    assert_eq!(
        ConnectorDiscoveryPolicy::load_or_default(root.path()).mode("dynapps-365"),
        Strict
    );
    let missing = tempfile::tempdir().unwrap();
    assert_eq!(
        ConnectorDiscoveryPolicy::load(missing.path()).unwrap(),
        ConnectorDiscoveryPolicy::default()
    );
}

#[test]
fn the_committed_connectors_file_parses() {
    let services = crate::support::repo_root().join("services");
    ConnectorDiscoveryPolicy::load(&services).expect("services/web/config/connectors.yaml");
}

#[tokio::test]
async fn strict_discovery_rejects_the_dynapps_365_shape() {
    let fixture = Fixture::start("dynapps-365", Strict);
    let error = fixture.authorize().await.unwrap_err().to_string();
    assert!(
        error.contains("OAuth protected resource mismatch"),
        "{error}"
    );
}

#[tokio::test]
async fn strict_discovery_rejects_a_metadata_document_advertised_as_the_issuer() {
    let fixture = Fixture::start("metadata-document-issuer", Strict);
    assert!(fixture.authorize().await.is_err());
}

#[tokio::test]
async fn lenient_discovery_connects_and_refreshes_the_dynapps_365_shape() {
    use systemprompt_web_admin_platform::connector_oauth::{
        exchange_with_client, refresh_with_client,
    };
    let fixture = Fixture::start("dynapps-365", Lenient);
    let (url, mut grant) = fixture.authorize().await.unwrap();
    let url = Url::parse(&url).unwrap();
    assert_eq!(url.path(), "/authorize");
    let params = url
        .query_pairs()
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(params["code_challenge_method"], "S256");
    assert_eq!(params["resource"], format!("{}/mcp", fixture.origin));
    assert_eq!(grant.authorization_issuer, fixture.origin);
    assert_eq!(grant.token_endpoint, format!("{}/token", fixture.origin));
    exchange_with_client(
        &fixture.services,
        &mut grant,
        "test-code",
        CALLBACK,
        &fixture.http,
    )
    .await
    .unwrap();
    assert_eq!(grant.access_token, "access-token");
    refresh_with_client(&fixture.services, &mut grant, &fixture.http)
        .await
        .unwrap();
    assert_eq!(
        grant.refresh_token.as_deref(),
        Some("rotated-refresh-token")
    );
}

#[tokio::test]
async fn lenient_discovery_still_requires_the_issuer_and_pkce_s256() {
    for mode in ["dynapps-365-issuer-mismatch", "dynapps-365-no-s256"] {
        let fixture = Fixture::start(mode, Lenient);
        let error = fixture.authorize().await.unwrap_err().to_string();
        assert!(
            error.contains("OAuth issuer mismatch or PKCE S256 unavailable"),
            "{mode}: {error}"
        );
    }
}

const SERVER: &str = "lenient-mcp";
const CALLBACK: &str = "https://dashboard.example.test/api/public/connectors/lenient-mcp/callback";

struct Server(std::process::Child);

impl Drop for Server {
    fn drop(&mut self) {
        if self.0.kill().is_ok() {
            self.0.wait().ok();
        }
    }
}

struct Fixture {
    _server: Server,
    _root: tempfile::TempDir,
    origin: String,
    services: systemprompt_web_admin_core::extension_services::ExtensionServices,
    http: reqwest::Client,
}

impl Fixture {
    fn start(shape: &str, mode: DiscoveryMode) -> Self {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};
        let root = tempfile::tempdir().unwrap();
        let script =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/oauth/server.py");
        let mut server = Server(
            Command::new("python3")
                .arg("-u")
                .arg(script)
                .arg(root.path())
                .arg(shape)
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        let mut origin = String::new();
        BufReader::new(server.0.stdout.take().unwrap())
            .read_line(&mut origin)
            .unwrap();
        let origin = origin.trim().to_owned();
        let yaml = format!(
            "mcp_servers:\n  {SERVER}:\n    server_type: external\n    endpoint: {origin}/mcp\n    enabled: true\n    display_in_web: false\n    oauth: {{required: false, scopes: [user], audience: mcp, client_id: null}}\n    connector: {{adapter: generic, scopes: [tools:read]}}\n"
        );
        let config = root.path().join("config.yaml");
        std::fs::write(&config, yaml).unwrap();
        let policy = match mode {
            Strict => ConnectorDiscoveryPolicy::default(),
            Lenient => ConnectorDiscoveryPolicy::from_yaml(&format!(
                "connectors:\n  {SERVER}:\n    discovery: lenient\n"
            ))
            .unwrap(),
        };
        let services = systemprompt_web_admin_core::extension_services::ExtensionServices::new(
            crate::boot_fixture::services_view(&config),
            std::sync::Arc::new(
                systemprompt::governance::authz::SubjectProviderSet::from_providers(Vec::new()),
            ),
        )
        .with_connector_discovery(policy);
        let ca =
            reqwest::Certificate::from_pem(&std::fs::read(root.path().join("ca.pem")).unwrap())
                .unwrap();
        let http = reqwest::Client::builder()
            .add_root_certificate(ca)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap();
        Self {
            _server: server,
            _root: root,
            origin,
            services,
            http,
        }
    }

    async fn authorize(
        &self,
    ) -> systemprompt_web_admin_core::error::AdminResult<(
        String,
        systemprompt_web_admin_platform::connector_oauth::Grant,
    )> {
        use systemprompt_web_admin_platform::connector_oauth::{ConnectorAccess, generic};
        let secrets = crate::boot_fixture::secret_store();
        generic::authorize_with_client(
            ConnectorAccess {
                services: &self.services,
                secrets: secrets.as_ref(),
            },
            generic::Consent {
                user: "active-user",
                state: "test-state",
                verifier: "test-verifier".into(),
            },
            Provider::try_from(SERVER.to_owned()).unwrap(),
            CALLBACK,
            &self.http,
        )
        .await
    }
}
