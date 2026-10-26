//! Personal subscription keys through the gateway's credential seam.
//!
//! A hop on a `kind: personal` credential is served only from the key the
//! calling user stored under that credential id: a user without one is refused
//! 403 with no fallback to any platform secret, a user with one presents
//! exactly that key upstream (with the provider's user-agent and the `OpenCode`
//! session header), one user's key is never presented for another, and a
//! request billed to it is stamped `billing_source = 'user_key'`.

use std::sync::{Arc, Mutex};

use axum::http::HeaderMap;
use systemprompt::ai::BillingSource;
use systemprompt::ai::repository::{AiRequestRepository, UserProviderCredentialRepository};
use systemprompt::database::{Database, DbPool};
use systemprompt::extension::prelude::{Extension, ExtensionContext};
use systemprompt::gateway::GatewayEngines;
use systemprompt::gateway::service::DispatchError;
use systemprompt::gateway::service::credentials::{KeyOwner, resolve};
use systemprompt::identifiers::{AiRequestId, CredentialId, UserId};
use systemprompt::manifest::services::catalog::{
    CatalogContext, CatalogDocument, ResolvedCatalog, ResolvedHop,
};
use systemprompt::manifest::services::root::ActiveServicesRoot;
use systemprompt::manifest::{Config, Profile};
use systemprompt::models::auth::TokenAuthority;
use systemprompt::security::at_rest::AtRestCipher;
use systemprompt::security::credential::TokenCache;
use systemprompt::storage::LocalFileStorage;
use systemprompt::traits::{
    BackgroundTasks, ConfigProvider, DatabaseHandle, FileStorage, SecretStore,
};
use systemprompt::wire::WireProtocol;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

use crate::fixtures::{RequestSpec, insert_request, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

const CREDENTIAL: &str = "opencode-go";
const MODEL: &str = "go-kimi-k3";
const USER_AGENT: &str = "opencode/1.0 (systemprompt-gateway)";

#[derive(Debug)]
struct StubConfig;

impl ConfigProvider for StubConfig {
    fn get(&self, _key: &str) -> Option<String> {
        None
    }
    fn database_url(&self) -> &'static str {
        "postgres://unused/unused"
    }
    fn system_path(&self) -> &'static str {
        "/tmp"
    }
    fn api_port(&self) -> u16 {
        0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct KeysCtx {
    database: DbPool,
    boot: ExtensionBoot,
    tokens: Arc<TokenCache>,
}

impl ExtensionContext for KeysCtx {
    fn profile(&self) -> Arc<Profile> {
        Arc::new(self.boot.profile().clone())
    }
    fn app_config(&self) -> Arc<Config> {
        Arc::clone(self.boot.config_arc())
    }
    fn services_root(&self) -> ActiveServicesRoot {
        self.boot.active_services_root().clone()
    }
    fn token_authority(&self) -> Arc<dyn TokenAuthority> {
        Arc::clone(self.boot.token_authority())
    }
    fn secrets(&self) -> Arc<dyn SecretStore> {
        Arc::clone(self.boot.secrets())
    }
    fn dependencies(&self) -> systemprompt::traits::Dependencies {
        systemprompt::traits::Dependencies::new()
    }
    fn system_owner_id(&self) -> UserId {
        UserId::new("personal-keys-owner")
    }
    fn config(&self) -> Arc<dyn ConfigProvider> {
        Arc::new(StubConfig)
    }
    fn database(&self) -> Arc<dyn DatabaseHandle> {
        Arc::<Database>::clone(&self.database)
    }
    fn get_extension(&self, _id: &str) -> Option<Arc<dyn Extension>> {
        None
    }
    fn file_storage(&self) -> Arc<dyn FileStorage> {
        Arc::new(LocalFileStorage::new(std::env::temp_dir()))
    }
    fn instance_id(&self) -> systemprompt::identifiers::InstanceId {
        systemprompt::identifiers::InstanceId::new("personal-keys")
    }
    fn background_tasks(&self) -> BackgroundTasks {
        BackgroundTasks::new()
    }
    fn services(&self) -> systemprompt::manifest::services::ServicesView {
        dynapps_test_common::empty_services_view()
    }
    fn provider_tokens(&self) -> Arc<dyn systemprompt::traits::ProviderTokenCache> {
        Arc::<TokenCache>::clone(&self.tokens)
    }
    fn subject_attributes(&self) -> Arc<dyn systemprompt::models::authz::SubjectAttributeSource> {
        Arc::new(systemprompt::governance::authz::SubjectProviderSet::from_providers(Vec::new()))
    }
}

fn engines(fixture: &dynapps_test_common::BootFixture, pool: &DbPool) -> GatewayEngines {
    let tokens = Arc::new(TokenCache::default());
    let ctx: Arc<dyn ExtensionContext> = Arc::new(KeysCtx {
        database: Arc::clone(pool),
        boot: fixture.extension_boot(),
        tokens: Arc::clone(&tokens),
    });
    GatewayEngines::new(tokens, &ctx, Arc::clone(fixture.boot().secrets()))
        .expect("gateway engines")
}

fn cipher(fixture: &dynapps_test_common::BootFixture) -> AtRestCipher {
    AtRestCipher::from_secrets(&fixture.boot().secrets().get()).expect("at-rest cipher")
}

type Seen = Arc<Mutex<Vec<HeaderMap>>>;

async fn upstream() -> (String, Seen) {
    let seen: Seen = Arc::default();
    let record = Arc::clone(&seen);
    let app = axum::Router::new().route(
        "/v1/chat/completions",
        axum::routing::post(move |headers: HeaderMap| {
            let record = Arc::clone(&record);
            async move {
                record.lock().expect("headers lock").push(headers);
                "{}"
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock upstream");
    let addr = listener.local_addr().expect("mock address");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("mock upstream") });
    (format!("http://{addr}/v1"), seen)
}

fn personal_catalog(endpoint: &str) -> ResolvedCatalog {
    let yaml = format!(
        "credentials:\n  {CREDENTIAL}:\n    kind: personal\n    label: OpenCode Go\n\
         providers:\n  opencode-go:\n    credential: {CREDENTIAL}\n    user_agent: \"{USER_AGENT}\"\n\
         \x20   forward_headers: [x-opencode-session]\n    endpoints:\n      chat:\n\
         \x20       wire: openai-chat\n        url: {endpoint}\n\
         models:\n  {MODEL}:\n    upstream_model: kimi-k3\n    clients: [openai]\n\
         deployments:\n  {MODEL}:\n  - opencode-go/chat\n"
    );
    CatalogDocument::from_yaml_str("personal-keys", &yaml)
        .expect("catalog document")
        .resolve(&CatalogContext::for_install("personal-keys-test"))
        .expect("resolved catalog")
}

fn hop(catalog: &ResolvedCatalog) -> ResolvedHop {
    catalog
        .deployments(MODEL)
        .first()
        .cloned()
        .expect("one personal hop")
}

fn owner(keys: &UserProviderCredentialRepository, user: &UserId) -> KeyOwner {
    KeyOwner {
        credentials: keys.clone(),
        user_id: user.clone(),
    }
}

async fn send(call: &systemprompt::ai::UpstreamCall) -> u16 {
    let forward = vec![("x-opencode-session".to_owned(), "ses_fixture".to_owned())];
    let mut request = reqwest::Client::new()
        .post(call.url(WireProtocol::OpenAiChat, "kimi-k3", false))
        .body("{}");
    for (name, value) in call.headers_forwarding(WireProtocol::OpenAiChat, &forward) {
        request = request.header(name, value);
    }
    request
        .send()
        .await
        .expect("mock upstream answers")
        .status()
        .as_u16()
}

#[tokio::test]
async fn personal_key_is_required_used_and_never_shared() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let fixture = dynapps_test_common::bootstrap_master_key().await;
    let pool: DbPool = Arc::new(Database::from_pools(
        Arc::clone(&db.pool),
        Some(Arc::clone(&db.pool)),
    ));
    let alice = insert_user(&db.pool, &unique("alice"), &unclaimed_email("alice")).await;
    let bob = insert_user(&db.pool, &unique("bob"), &unclaimed_email("bob")).await;
    let (endpoint, seen) = upstream().await;
    let catalog = personal_catalog(&endpoint);
    let hop = hop(&catalog);
    let credential = CredentialId::new(CREDENTIAL);
    let at_rest = cipher(&fixture);
    let keys = UserProviderCredentialRepository::new(&pool, at_rest.clone());
    let tokens = engines(&fixture, &pool);

    let refused = resolve(&catalog, &hop, &owner(&keys, &alice), &tokens)
        .await
        .expect_err("no key stored yet");
    let DispatchError::PreAudit(error) = refused else {
        panic!("a missing personal key is refused before the audit row opens");
    };
    assert_eq!(error.status().as_u16(), 403);
    assert!(
        error
            .to_string()
            .contains("personal key required for credential opencode-go")
    );

    let sealed = at_rest.seal("go-key-alice").expect("seal alice's key");
    keys.set_key(&alice, &credential, &sealed, Some("laptop"))
        .await
        .expect("store alice's key");
    let listed = keys.list_metadata(&alice).await.expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].label.as_deref(), Some("laptop"));

    let (call, billing) = resolve(&catalog, &hop, &owner(&keys, &alice), &tokens)
        .await
        .expect("alice is served from her key");
    assert_eq!(billing, BillingSource::UserKey);
    assert_eq!(send(&call).await, 200);
    let headers = seen.lock().expect("headers lock").pop().expect("one call");
    assert_eq!(headers["authorization"], "Bearer go-key-alice");
    assert_eq!(headers["x-opencode-session"], "ses_fixture");
    assert_eq!(headers["user-agent"], USER_AGENT);
    assert!(headers.get("anthropic-beta").is_none());

    let bob_refused = resolve(&catalog, &hop, &owner(&keys, &bob), &tokens).await;
    assert!(
        matches!(bob_refused, Err(DispatchError::PreAudit(ref e)) if e.status().as_u16() == 403),
        "alice's key must never serve bob"
    );
    assert!(
        keys.find_key(&bob, &credential)
            .await
            .expect("lookup")
            .is_none()
    );

    assert!(keys.delete_key(&alice, &credential).await.expect("delete"));
    assert!(
        resolve(&catalog, &hop, &owner(&keys, &alice), &tokens)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn user_key_billing_is_stamped_on_the_request_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let pool: DbPool = Arc::new(Database::from_pools(
        Arc::clone(&db.pool),
        Some(Arc::clone(&db.pool)),
    ));
    let user = insert_user(&db.pool, &unique("biller"), &unclaimed_email("biller")).await;
    let id = unique("req");
    insert_request(&db.pool, &RequestSpec::completed(&id, &user)).await;
    let default: String =
        sqlx::query_scalar("SELECT billing_source FROM ai_requests WHERE id = $1")
            .bind(&id)
            .fetch_one(&*db.pool)
            .await
            .expect("row");
    assert_eq!(default, "platform");

    AiRequestRepository::new(&pool)
        .update_billing_source(
            &AiRequestId::new(id.clone()),
            BillingSource::UserKey.as_str(),
        )
        .await
        .expect("stamp");
    let stamped: String =
        sqlx::query_scalar("SELECT billing_source FROM ai_requests WHERE id = $1")
            .bind(&id)
            .fetch_one(&*db.pool)
            .await
            .expect("row");
    assert_eq!(stamped, "user_key");
}
