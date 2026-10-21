//! `repositories::provider_credentials` — the admin view of who holds a
//! personal-subscription key, and the guarantee that it carries no key.

use systemprompt_web_admin_identity::repositories::provider_credentials::list_provider_credential_holders;

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

#[tokio::test]
async fn list_provider_credential_holders_returns_metadata_per_member_and_provider() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("byok"), &unclaimed_email("byok")).await;
    sqlx::query(
        "INSERT INTO user_provider_credentials \
           (user_id, provider, ciphertext, nonce, label, verified_at) \
         VALUES ($1, 'opencode-go', 'sealed-secret-ciphertext', 'nonce-hex', 'laptop', now())",
    )
    .bind(user.as_str())
    .execute(&*db.pool)
    .await
    .expect("seed a credential");

    let holders = list_provider_credential_holders(&db.pool)
        .await
        .expect("listing succeeds");

    let row = holders
        .iter()
        .find(|h| h.user_id == user)
        .expect("the member's key is listed");
    assert_eq!(row.provider, "opencode-go");
    assert_eq!(row.label.as_deref(), Some("laptop"));
    assert!(row.verified_at.is_some());
    assert!(row.last_used_at.is_none());
    let serialized = serde_json::to_string(&holders).expect("serializes");
    assert!(
        !serialized.contains("sealed-secret-ciphertext") && !serialized.contains("nonce-hex"),
        "the sealed key never leaves the repository"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_provider_credential_holders_is_empty_without_keys() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("nokey"), &unclaimed_email("nokey")).await;
    let holders = list_provider_credential_holders(&db.pool)
        .await
        .expect("listing succeeds");
    assert!(!holders.iter().any(|h| h.user_id == user));
    db.cleanup().await;
}
