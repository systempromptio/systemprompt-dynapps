//! The `farm` attribution dimension when a farm runs as a service account:
//! the run's key is issued to the service user, so the farm's active-run
//! lookup must answer for that user as well as for the launcher.

use systemprompt::identifiers::UserId;
use systemprompt_web_admin_farm::repositories::runs;

use crate::farm_runner_lifecycle::{launch_one, seed};
use crate::tempdb::TempDb;

#[tokio::test]
async fn active_farm_ids_include_runs_of_a_farm_the_user_runs_as() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let seeded = seed(&db.pool).await;
    let service = UserId::new(format!("svc-{}", seeded.user.as_str()));
    let stranger = UserId::new(format!("other-{}", seeded.user.as_str()));
    for id in [&service, &stranger] {
        sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $1, $1 || '@test.invalid')")
            .bind(id.as_str())
            .execute(db.pool.as_ref())
            .await
            .expect("insert user");
    }
    sqlx::query("UPDATE agent_farms SET run_as_user_id = $1 WHERE id = $2")
        .bind(service.as_str())
        .bind(seeded.farm.as_str())
        .execute(db.pool.as_ref())
        .await
        .expect("set run_as");
    launch_one(&seeded).await;

    for (user, expected) in [(&seeded.user, 1), (&service, 1), (&stranger, 0)] {
        let farms = runs::list_active_farm_ids_for_user(&db.pool, user)
            .await
            .expect("list active farm ids");
        assert_eq!(farms.len(), expected, "user {user}");
    }
}
