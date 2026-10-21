//! Personal subscription keys through the gateway's credential seam.
//!
//! A `credential: user` provider is served only from the key the calling user
//! stored: a user without one is refused 403 with no fallback to any platform
//! secret, a user with one presents exactly that key upstream (with the
//! provider's user-agent and the `OpenCode` session header), one user's key is
//! never presented for another, and a request billed to it is stamped
//! `billing_source = 'user_key'`.

use std::sync::{Arc, Mutex};

use axum::http::HeaderMap;
use systemprompt::ai::repository::{AiRequestRepository, UserProviderCredentialRepository};
use systemprompt::database::{Database, DbPool};
use systemprompt::gateway::service::DispatchError;
use systemprompt::gateway::service::credentials::{BillingSource, KeyOwner, resolve};
use systemprompt::identifiers::{AiRequestId, UserId};
use systemprompt::manifest::services::ProviderEntry;
use systemprompt::wire::WireProtocol;

use crate::fixtures::{RequestSpec, insert_request, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

const PROVIDER: &str = "opencode-go";
const USER_AGENT: &str = "opencode/1.0 (systemprompt-gateway)";

async fn secrets() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("profile directory");
    let root = dynapps_test_common::repo_path("");
    let yaml = include_str!("../../../contract/admin/fixtures/profile.yaml")
        .replace("__PROFILE_DIR__", &dir.path().to_string_lossy())
        .replace("__REPO__", &root.to_string_lossy());
    std::fs::create_dir_all(dir.path().join("bin")).expect("bin directory");
    std::fs::write(dir.path().join("profile.yaml"), yaml).expect("fixture profile");
    let secrets = serde_json::json!({
        "database_url": "postgres://unused:unused@localhost:5432/postgres",
        "oauth_at_rest_pepper": "personal-keys-test-pepper-1234567890",
        "manifest_signing_secret_seed": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "encryption_master_key": "22".repeat(32),
    });
    std::fs::write(dir.path().join("secrets.json"), secrets.to_string()).expect("secrets");
    systemprompt::config::ProfileBootstrap::init_from_path(&dir.path().join("profile.yaml"))
        .expect("profile bootstrap");
    systemprompt::config::SecretsBootstrap::init()
        .await
        .expect("secrets bootstrap");
    dir
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

fn personal_provider(endpoint: &str) -> ProviderEntry {
    serde_yaml::from_str(&format!(
        "name: {PROVIDER}\nwire: openai_chat\nsurface: openai\nendpoint: {endpoint}\n\
         credential: user\nuser_agent: \"{USER_AGENT}\"\nforward_headers: [x-opencode-session]\n"
    ))
    .expect("provider entry")
}

fn owner(pool: &DbPool, user: &UserId) -> KeyOwner {
    KeyOwner {
        db: Arc::clone(pool),
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
    let _profile = secrets().await;
    let pool: DbPool = Arc::new(Database::from_pools(
        Arc::clone(&db.pool),
        Some(Arc::clone(&db.pool)),
    ));
    let alice = insert_user(&db.pool, &unique("alice"), &unclaimed_email("alice")).await;
    let bob = insert_user(&db.pool, &unique("bob"), &unclaimed_email("bob")).await;
    let (endpoint, seen) = upstream().await;
    let entry = personal_provider(&endpoint);

    let refused = resolve(&entry, &owner(&pool, &alice))
        .await
        .expect_err("no key stored yet");
    let DispatchError::PreAudit(error) = refused else {
        panic!("a missing personal key is refused before the audit row opens");
    };
    assert_eq!(error.status().as_u16(), 403);
    assert!(
        error
            .to_string()
            .contains("personal key required for provider opencode-go")
    );

    let keys = UserProviderCredentialRepository::new(&pool);
    keys.set_key(&alice, PROVIDER, "go-key-alice", Some("laptop"))
        .await
        .expect("store alice's key");
    let listed = keys.list_metadata(&alice).await.expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].label.as_deref(), Some("laptop"));

    let (call, billing) = resolve(&entry, &owner(&pool, &alice))
        .await
        .expect("alice is served from her key");
    assert_eq!(billing, BillingSource::UserKey);
    assert_eq!(send(&call).await, 200);
    let headers = seen.lock().expect("headers lock").pop().expect("one call");
    assert_eq!(headers["authorization"], "Bearer go-key-alice");
    assert_eq!(headers["x-opencode-session"], "ses_fixture");
    assert_eq!(headers["user-agent"], USER_AGENT);
    assert!(headers.get("anthropic-beta").is_none());

    let bob_refused = resolve(&entry, &owner(&pool, &bob)).await;
    assert!(
        matches!(bob_refused, Err(DispatchError::PreAudit(ref e)) if e.status().as_u16() == 403),
        "alice's key must never serve bob"
    );
    assert!(
        keys.find_key(&bob, PROVIDER)
            .await
            .expect("lookup")
            .is_none()
    );

    assert!(keys.delete_key(&alice, PROVIDER).await.expect("delete"));
    assert!(resolve(&entry, &owner(&pool, &alice)).await.is_err());
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
