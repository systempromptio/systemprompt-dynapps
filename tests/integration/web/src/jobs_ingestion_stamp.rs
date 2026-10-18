//! Content ingestion is versioned, so a pod of an older release cannot
//! ingest its tree over the content a newer release already ingested.

use std::sync::Arc;

use systemprompt::database::Database;
use systemprompt::identifiers::{Actor, UserId};
use systemprompt::traits::{Job, JobContext};
use systemprompt_web_jobs::{ContentIngestionJob, release_key};

use crate::tempdb::TempDb;

#[test]
fn releases_order_numerically() {
    assert!(release_key("0.70.0") > release_key("0.69.9"));
    assert!(release_key("0.10.0") > release_key("0.9.0"));
    assert_eq!(release_key("0.70.0-rc.1"), release_key("0.70.0"));
}

#[tokio::test]
async fn an_older_release_refuses_to_ingest_over_a_newer_one() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    sqlx::query("INSERT INTO content_ingestion_stamp (id, version) VALUES (TRUE, '999.0.0')")
        .execute(&*db.pool)
        .await
        .expect("seed a newer release's stamp");
    let database = Arc::new(Database::from_pools(
        Arc::clone(&db.pool),
        Some(Arc::clone(&db.pool)),
    ));
    let ctx = JobContext::new(
        Actor::user(UserId::new("ingestion-stamp-test")),
        systemprompt::traits::Dependencies::new().with(database),
    );

    let result = ContentIngestionJob.execute(&ctx).await.expect("job runs");
    assert!(result.success);
    assert!(
        result
            .message
            .as_deref()
            .is_some_and(|m| m.contains("999.0.0 already ingested")),
        "{result:?}"
    );
    let version: String =
        sqlx::query_scalar("SELECT version FROM content_ingestion_stamp WHERE id")
            .fetch_one(&*db.pool)
            .await
            .expect("stamp");
    assert_eq!(version, "999.0.0", "the newer stamp is left alone");
    db.cleanup().await;
}
