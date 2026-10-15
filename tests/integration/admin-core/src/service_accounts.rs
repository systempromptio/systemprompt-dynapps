//! Service accounts and the machine credentials the broker issues from.
//!
//! The rules that matter are the refusals: a person cannot hold a machine
//! credential, and a service account's credential is only spent for a
//! workflow write-back or farm caller. Minting is core's `MachineGrant`; the
//! GitHub exchange runs through it against a local mock that checks the App
//! JWT is presented and answers with an installation token.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::database::Database;
use systemprompt::identifiers::UserId;
use systemprompt::users::UserRepository;
use systemprompt_web_admin_platform::machine_credentials::{self, AttachCredential, Holder};
use systemprompt_web_admin_platform::service_accounts::ServiceAccounts;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

// A throwaway App key, generated once per test binary: no private key is
// committed (`*.pem` is ignored on purpose).
fn test_key() -> &'static str {
    static KEY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    KEY.get_or_init(|| {
        use rsa::pkcs1::EncodeRsaPrivateKey;
        let key = rsa::RsaPrivateKey::new(&mut rand_core::OsRng, 2048).expect("generate RSA key");
        key.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
            .expect("encode RSA key")
            .to_string()
    })
}

// `load_master_key` accepts only a 32-byte hex key.
#[expect(
    unsafe_code,
    reason = "set_var is unsafe in edition 2024; see the SAFETY note"
)]
fn set_master_key() {
    // SAFETY: nextest gives each test its own process, so nothing else reads
    // the environment while this runs.
    unsafe {
        std::env::set_var(
            "ENCRYPTION_MASTER_KEY",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        );
    }
}

fn accounts(pool: &Arc<PgPool>) -> ServiceAccounts {
    let db = Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    ServiceAccounts::new(UserRepository::new(&db))
}

async fn service_account(pool: &Arc<PgPool>) -> UserId {
    let admin = insert_user(pool, &unique("sa-admin"), &unclaimed_email("sa-admin")).await;
    accounts(pool)
        .create("Release notes", &unclaimed_email("svc"), &admin)
        .await
        .expect("create service account")
}

fn github(api_url: &str) -> AttachCredential {
    serde_json::from_value(serde_json::json!({
        "kind": "github_app", "app_id": "4242", "installation_id": "777",
        "api_url": api_url, "secret": test_key(),
    }))
    .expect("attach body")
}

// Answers one request with a GitHub installation token and returns the raw
// request it received.
async fn mock_github() -> (String, tokio::task::JoinHandle<String>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        let mut buffer = [0; 8192];
        loop {
            let count = socket.read(&mut buffer).await.unwrap();
            request.extend_from_slice(&buffer[..count]);
            if count == 0 || String::from_utf8_lossy(&request).contains("\r\n\r\n") {
                break;
            }
        }
        let body = r#"{"token":"ghs_installation_token","expires_at":"2999-01-01T00:00:00Z"}"#;
        let response = format!(
            "HTTP/1.1 201 Created\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        );
        socket.write_all(response.as_bytes()).await.unwrap();
        String::from_utf8_lossy(&request).into_owned()
    });
    (format!("http://{address}"), task)
}

#[tokio::test]
async fn a_created_service_account_is_listed_and_found() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = service_account(&db.pool).await;
    assert!(accounts(&db.pool).is_service_account(&id).await.unwrap());
    let listed = accounts(&db.pool).list().await.unwrap();
    assert!(listed.iter().any(|a| a.user_id == id));
    db.cleanup().await;
}

#[tokio::test]
async fn a_person_is_not_a_service_account() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let person = insert_user(&db.pool, &unique("person"), &unclaimed_email("person")).await;
    assert!(
        !accounts(&db.pool)
            .is_service_account(&person)
            .await
            .unwrap()
    );
    assert!(accounts(&db.pool).find(&person).await.unwrap().is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn a_person_cannot_hold_a_machine_credential() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    set_master_key();
    let person = insert_user(&db.pool, &unique("person"), &unclaimed_email("person")).await;
    let refused = machine_credentials::attach(
        &db.pool,
        &accounts(&db.pool),
        Holder {
            user: &person,
            provider: "github",
        },
        github("https://api.github.com"),
        "test",
    )
    .await;
    assert!(matches!(
        refused,
        Err(systemprompt_web_admin_core::error::AdminError::Unprocessable(_))
    ));
    db.cleanup().await;
}

#[tokio::test]
async fn the_broker_refuses_a_caller_that_is_not_a_write_back() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    set_master_key();
    let id = service_account(&db.pool).await;
    machine_credentials::attach(
        &db.pool,
        &accounts(&db.pool),
        Holder {
            user: &id,
            provider: "github",
        },
        github("https://api.github.com"),
        "test",
    )
    .await
    .expect("attach");
    for caller in [None, Some("interactive"), Some("workflow:x:read")] {
        let refused = machine_credentials::broker_token(&db.pool, &id, "github", caller).await;
        assert!(
            matches!(
                refused,
                Err(systemprompt_web_admin_core::error::AdminError::Forbidden(_))
            ),
            "{caller:?} was not refused"
        );
    }
    db.cleanup().await;
}

#[tokio::test]
async fn the_broker_returns_a_github_installation_token() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    set_master_key();
    let id = service_account(&db.pool).await;
    let (api, request) = mock_github().await;
    machine_credentials::attach(
        &db.pool,
        &accounts(&db.pool),
        Holder {
            user: &id,
            provider: "github",
        },
        github(&api),
        "test",
    )
    .await
    .expect("attach");
    let value = machine_credentials::broker_token(
        &db.pool,
        &id,
        "github",
        Some("workflow:release-notes:write"),
    )
    .await
    .expect("token");
    assert_eq!(value, "Bearer ghs_installation_token");
    let request = request.await.unwrap();
    assert!(request.starts_with("POST /app/installations/777/access_tokens "));
    assert!(request.to_lowercase().contains("authorization: bearer ey"));

    let cached = machine_credentials::broker_token(&db.pool, &id, "github", Some("farm:triage"))
        .await
        .expect("cached token; the mock answers only once");
    assert_eq!(cached, value);
    db.cleanup().await;
}

#[tokio::test]
async fn a_credential_with_no_row_is_not_connected() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = service_account(&db.pool).await;
    let missing =
        machine_credentials::broker_token(&db.pool, &id, "github", Some("farm:triage")).await;
    assert!(matches!(
        missing,
        Err(systemprompt_web_admin_core::error::AdminError::NotFound(_))
    ));
    db.cleanup().await;
}

#[tokio::test]
async fn the_broker_returns_an_odoo_basic_pair() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    set_master_key();
    let id = service_account(&db.pool).await;
    let body: AttachCredential = serde_json::from_value(serde_json::json!({
        "kind": "odoo_api_key", "url": "https://odoo.example.com", "db": "main",
        "login": "bot@example.com", "secret": "tok",
    }))
    .expect("attach body");
    machine_credentials::attach(
        &db.pool,
        &accounts(&db.pool),
        Holder {
            user: &id,
            provider: "odoo",
        },
        body,
        "test",
    )
    .await
    .expect("attach");
    let value = machine_credentials::broker_token(&db.pool, &id, "odoo", Some("farm:triage"))
        .await
        .expect("token");
    assert_eq!(value, "Basic Ym90QGV4YW1wbGUuY29tOnRvaw==");
    db.cleanup().await;
}
