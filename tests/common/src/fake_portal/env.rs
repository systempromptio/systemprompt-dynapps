//! A process configured to talk to a [`FakePortal`].
//!
//! A throwaway profile whose
//! services tree declares `dynapps-odoo` and `dynbrain` at the fake's MCP
//! paths (adapter `generic`, as in `services/mcp/*.yaml`), secrets carrying a
//! credential-broker secret and master key, the signing keys
//! [`PortalEnv::mint`] signs with, and `SSL_CERT_FILE` naming the fake's CA so
//! every default `reqwest` client in the connector code trusts it.
//!
//! Core's profile and config are process-wide, so one test process holds one
//! `PortalEnv`; nextest gives each test its own process. The services tree is
//! not: [`PortalEnv::services`] is the view the code under test is handed.

use std::collections::BTreeMap;

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, Header, encode};
use std::sync::OnceLock;
use systemprompt::identifiers::SessionId;
use systemprompt::models::auth::{
    CustomClaims, JwtAudience, JwtClaims, Permission, RateLimitTier, RegisteredClaims, TokenType,
    UserType,
};

use systemprompt::security::keys::SigningKeys;

use super::{DYNBRAIN_PATH, FakePortal, ODOO_PATH};

// The broker secret core presents as `x-systemprompt-credential-broker`.
pub const BROKER_SECRET: &str = "fake-portal-broker-secret-not-a-real-secret";

static KEYS: OnceLock<SigningKeys> = OnceLock::new();

const PROFILE: &str = include_str!("../../../contract/admin/fixtures/profile.yaml");
const COPIED: [&str; 3] = [
    "services/web/config.yaml",
    "services/web/metadata.yaml",
    "services/content/config.yaml",
];

#[derive(Debug)]
pub struct PortalEnv {
    pub portal: FakePortal,
    pub services: systemprompt_web_admin_core::extension_services::ExtensionServices,
    _dir: tempfile::TempDir,
}

// Who a minted JWT speaks for.
#[derive(Debug, Clone, Copy)]
pub struct Principal<'a> {
    pub user_id: &'a str,
    pub email: &'a str,
    pub session_id: Option<&'a str>,
}

impl PortalEnv {
    pub async fn start() -> Self {
        let portal = FakePortal::start().await;
        let dir = tempfile::tempdir().expect("portal env dir");
        let root = dir.path();
        std::fs::create_dir_all(root.join("bin")).expect("bin dir");
        std::fs::create_dir_all(root.join("services/skills")).expect("skills dir");
        std::fs::create_dir_all(root.join("services/config")).expect("config dir");
        for relative in COPIED {
            let destination = root.join(relative);
            std::fs::create_dir_all(destination.parent().expect("parent")).expect("mkdir");
            std::fs::copy(crate::repo_path(relative), destination).expect("copy services file");
        }
        let config = root.join("services/config/config.yaml");
        std::fs::write(&config, services_yaml(&portal)).expect("write services config");
        let profile = PROFILE
            .replace("__REPO__", &crate::repo_root().to_string_lossy())
            .replace("__PROFILE_DIR__", &root.to_string_lossy());
        std::fs::write(root.join("profile.yaml"), profile).expect("write profile");
        std::fs::write(root.join("secrets.json"), secrets()).expect("write secrets");

        trust_ca(&portal);
        systemprompt::config::ProfileBootstrap::init_from_path(&root.join("profile.yaml"))
            .expect("init profile");
        systemprompt::config::SecretsBootstrap::try_init()
            .await
            .expect("init secrets");
        systemprompt::config::try_init_config(None).expect("init config");
        let services = crate::offline_extension_services(crate::services_view(&config));
        KEYS.get_or_init(crate::ephemeral_signing_keys);
        Self {
            portal,
            services,
            _dir: dir,
        }
    }

    // The keys every minted JWT is signed with, and the routers under test
    // validate against.
    pub fn signing_keys() -> &'static SigningKeys {
        KEYS.get_or_init(crate::ephemeral_signing_keys)
    }

    // An `aud=mcp` JWT for `who`, signed by the installed authority, the
    // credential core forwards to the broker on a user's behalf.
    pub fn mint(who: Principal<'_>) -> String {
        let now = Utc::now();
        let registered = RegisteredClaims {
            sub: who.user_id.to_owned(),
            iat: now.timestamp(),
            exp: (now + Duration::hours(1)).timestamp(),
            nbf: Some(now.timestamp()),
            iss: systemprompt::manifest::Config::get()
                .expect("config")
                .jwt_issuer
                .clone(),
            aud: vec![JwtAudience::Api, JwtAudience::Mcp],
            jti: uuid::Uuid::new_v4().to_string(),
        };
        let custom = CustomClaims {
            scope: vec![Permission::User],
            username: who.email.to_owned(),
            email: who.email.to_owned(),
            user_type: UserType::User,
            roles: vec!["user".to_owned()],
            attributes: BTreeMap::new(),
            client_id: None,
            token_type: TokenType::Bearer,
            auth_time: now.timestamp(),
            session_id: who.session_id.map(|s| SessionId::new(s.to_owned())),
            rate_limit_tier: Some(RateLimitTier::User),
            plugin_id: None,
            purpose: None,
            act: None,
        };
        let claims = JwtClaims { registered, custom };
        let mut header = Header::new(Algorithm::RS256);
        let keys = Self::signing_keys();
        header.kid = Some(keys.active_kid().to_owned());
        encode(&header, &claims, keys.encoding_key()).expect("sign JWT")
    }
}

fn services_yaml(portal: &FakePortal) -> String {
    let server = |id: &str, path: &str| {
        format!(
            "  {id}:\n    server_type: external\n    endpoint: {}\n    enabled: true\n    \
             display_in_web: false\n    oauth: {{required: false, scopes: [user], audience: mcp, \
             client_id: null}}\n    connector: {{adapter: generic, display_name: {id}, scopes: \
             [odoo]}}\n",
            portal.endpoint(path)
        )
    };
    format!(
        "mcp_servers:\n{}{}",
        server("dynapps-odoo", ODOO_PATH),
        server("dynbrain", DYNBRAIN_PATH)
    )
}

fn secrets() -> String {
    format!(
        r#"{{
  "database_url": "postgres://unused:unused@localhost:5432/postgres",
  "oauth_at_rest_pepper": "fake-portal-pepper-not-a-real-secret",
  "mcp_credential_broker_secret": "{BROKER_SECRET}",
  "manifest_signing_secret_seed": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
  "encryption_master_key": "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
}}"#
    )
}

#[expect(
    unsafe_code,
    reason = "set_var is unsafe in edition 2024; see the SAFETY note"
)]
fn trust_ca(portal: &FakePortal) {
    // SAFETY: nextest gives each test its own process, and this runs before
    // the test builds any HTTP client or spawns a reader of the environment.
    unsafe {
        std::env::set_var("SSL_CERT_FILE", portal.ca_pem_path());
    }
}
