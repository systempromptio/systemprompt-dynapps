//! NFR-5.6: the audit hash chain is sealed by `audit_chain_seal`, not by an
//! insert trigger. Three gateway rows land unlinked; one seal links every one
//! of them, leaves nothing waiting, and the chain verifies clean.

use crate::fixtures::{RequestSpec, insert_request, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

#[tokio::test]
async fn one_seal_links_every_new_request_and_the_chain_verifies() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("seal")).await;
    for _ in 0..3 {
        let id = unique("req");
        insert_request(&db.pool, &RequestSpec::completed(&id, &user)).await;
    }

    let waiting: i64 = sqlx::query_scalar("SELECT audit_chain_unsealed('ai_requests')")
        .fetch_one(&*db.pool)
        .await
        .expect("count unsealed rows");
    assert!(waiting >= 3, "an insert no longer links itself");

    let sealed: i64 = sqlx::query_scalar("SELECT audit_chain_seal(5000)")
        .fetch_one(&*db.pool)
        .await
        .expect("seal the chain");
    assert_eq!(sealed, waiting, "the sealer links each new row once");

    let unsealed: i64 = sqlx::query_scalar(
        "SELECT audit_chain_unsealed('ai_requests') + audit_chain_unsealed('ai_safety_findings')",
    )
    .fetch_one(&*db.pool)
    .await
    .expect("count unsealed rows");
    assert_eq!(unsealed, 0, "nothing is left waiting after a seal");

    let again: i64 = sqlx::query_scalar("SELECT audit_chain_seal(5000)")
        .fetch_one(&*db.pool)
        .await
        .expect("seal again");
    assert_eq!(again, 0, "a second seal finds nothing new");

    let problems: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM verify_audit_chain('ai_requests', now() - interval '1 day', now() + interval '1 minute')",
    )
    .fetch_one(&*db.pool)
    .await
    .expect("verify the chain");
    assert_eq!(problems, 0, "the sealed chain verifies");
    db.cleanup().await;
}
