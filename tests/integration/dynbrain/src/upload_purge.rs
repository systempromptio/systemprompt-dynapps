//! The job that deletes uploads nobody adopted.

use std::path::Path;

use chrono::{Duration, Utc};
use dynbrain::assets::model::StoredUpload;
use dynbrain::ids::UploadId;
use dynbrain::ingest::purge::{GRACE, PurgeReport, purge_uploads};
use systemprompt::storage::LocalFileStorage;
use systemprompt::traits::FileStorage;

use crate::fixture::{Fixture, user};
use crate::ingest_support::count;

async fn slot(
    f: &Fixture,
    storage: &LocalFileStorage,
    expires_in: Duration,
    bytes: bool,
) -> UploadId {
    let upload = UploadId::generate();
    f.repo
        .open_upload(&upload, &user("alice"), Utc::now() + expires_in)
        .await
        .expect("open a slot");
    if bytes {
        let key = format!("dynbrain/uploads/{upload}");
        let stored = storage
            .store(Path::new(&key), b"bytes")
            .await
            .expect("store the bytes");
        f.repo
            .fill_upload(
                &upload,
                &StoredUpload {
                    storage_ref: stored.0,
                    size_bytes: 5,
                    sha256: "00".to_owned(),
                    mimetype: "text/plain".to_owned(),
                },
            )
            .await
            .expect("record the bytes");
    }
    upload
}

#[tokio::test]
async fn an_expired_upload_nobody_adopted_is_deleted_with_its_bytes_and_nothing_else_is() {
    let db = dynapps_test_common::db_or_skip!();
    let f = Fixture::new(db).await;
    let root = std::env::temp_dir().join(format!("dynbrain-purge-{}", std::process::id()));
    let storage = LocalFileStorage::new(root.clone());
    let on_disk = |upload: &UploadId| root.join("dynbrain/uploads").join(upload.as_str()).exists();
    let received = slot(&f, &storage, -Duration::days(3), true).await;
    let never_filled = slot(&f, &storage, -Duration::days(2), false).await;
    let just_expired = slot(&f, &storage, -Duration::hours(1), true).await;
    let live = slot(&f, &storage, Duration::minutes(15), true).await;
    let adopted = slot(&f, &storage, -Duration::days(3), true).await;
    sqlx::query("UPDATE dynbrain_upload SET consumed_at = CURRENT_TIMESTAMP WHERE id = $1")
        .bind(adopted.as_str())
        .execute(f.pool())
        .await
        .expect("adopt");
    let report = purge_uploads(&f.repo, &storage, Utc::now() - GRACE)
        .await
        .expect("purge");
    assert_eq!(
        report,
        PurgeReport {
            slots: 2,
            deleted: 1,
            kept: 0
        }
    );
    assert!(
        !on_disk(&received),
        "the bytes of an unadopted upload are deleted"
    );
    assert!(on_disk(&just_expired) && on_disk(&live) && on_disk(&adopted));
    let left = count(&f, "SELECT COUNT(*) FROM dynbrain_upload").await;
    assert_eq!(left, 3);
    let gone = count(
        &f,
        &format!("SELECT COUNT(*) FROM dynbrain_upload WHERE id = '{never_filled}'"),
    )
    .await;
    assert_eq!(gone, 0);
    std::fs::remove_dir_all(&root).ok();
    f.finish().await;
}
