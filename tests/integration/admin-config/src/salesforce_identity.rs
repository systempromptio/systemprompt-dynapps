//! One Salesforce Username per (user, org); the authz dimension reports the
//! orgs a user is linked for.

use crate::fixtures::{insert_user, unique, user_id};
use crate::tempdb::TempDb;
use systemprompt_web_admin_core::repositories::principal::salesforce_identity::list_identities;
use systemprompt_web_admin_platform::repositories::connectors::salesforce_identity as repo;

#[tokio::test]
async fn identities_are_keyed_per_org_and_listed_for_authz() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let alice = unique("sf-alice");
    insert_user(&db.pool, &alice).await;
    let alice = user_id(&alice);
    repo::upsert_identity(&db.pool, &alice, "salesforce", "alice@astound.prod")
        .await
        .unwrap();
    repo::upsert_identity(&db.pool, &alice, "salesforce-uat", "alice@client-a.uat")
        .await
        .unwrap();
    repo::upsert_identity(&db.pool, &alice, "salesforce", "alice.fixed@astound.prod")
        .await
        .unwrap();

    assert_eq!(
        repo::find_username(&db.pool, &alice, "salesforce")
            .await
            .unwrap()
            .as_deref(),
        Some("alice.fixed@astound.prod")
    );
    assert_eq!(
        repo::find_username(&db.pool, &alice, "salesforce-uat")
            .await
            .unwrap()
            .as_deref(),
        Some("alice@client-a.uat")
    );
    assert!(
        repo::find_username(&db.pool, &alice, "salesforce-other")
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        repo::list_linked_providers(&db.pool, &alice).await.unwrap(),
        ["salesforce", "salesforce-uat"]
    );
    assert!(
        repo::upsert_identity(&db.pool, &alice, "atlassian", "nope")
            .await
            .is_err()
    );

    repo::delete_identity(&db.pool, &alice, "salesforce-uat")
        .await
        .unwrap();
    assert_eq!(
        repo::list_linked_providers(&db.pool, &alice).await.unwrap(),
        ["salesforce"]
    );
    repo::delete_all_identities(&db.pool, &alice).await.unwrap();
    assert!(list_identities(&db.pool, &alice).await.unwrap().is_empty());
}
