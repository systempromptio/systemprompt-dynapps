//! The `farm` attribution dimension against a real schema: a user holds a
//! farm while one of their runs on it is active, and loses it once the run
//! is cancelled.

use std::sync::Arc;

use systemprompt::governance::authz::SubjectAttributeProvider;
use systemprompt_web_admin_farm::attribution::FarmAttributeProvider;
use systemprompt_web_admin_farm::services::runs::cancel;

use crate::farm_runner_lifecycle::{launch_one, seed};
use crate::tempdb::TempDb;

#[tokio::test]
async fn a_users_active_run_attributes_its_farm_until_cancelled() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let provider = FarmAttributeProvider::new(Arc::clone(&db.pool));
    assert!(
        provider
            .values_for(&seeded.user)
            .await
            .expect("values before launch")
            .is_empty(),
        "a user with no runs holds no farm"
    );

    let run = launch_one(&seeded).await;
    let active = provider
        .values_for(&seeded.user)
        .await
        .expect("values while queued");
    assert_eq!(active, vec![seeded.farm.as_str().to_owned()]);

    let cancelled = cancel(&seeded.state, &run.id, &seeded.user)
        .await
        .expect("cancel");
    assert!(cancelled);
    assert!(
        provider
            .values_for(&seeded.user)
            .await
            .expect("values after cancel")
            .is_empty(),
        "a cancelled run no longer attributes its farm"
    );
    db.cleanup().await;
}
