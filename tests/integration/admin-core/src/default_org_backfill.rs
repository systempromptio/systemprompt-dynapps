//! `jobs::backfill_default_organization` — the repair for users provisioned
//! before a default organization existed.
//!
//! Membership is what every analytics query, the customer usage report, and
//! every plan grant scope through, so a user with no `organization_members` row
//! is invisible to the dashboards meant to account for them. Until the
//! `is_default` organization existed, an SSO arrival on an unclaimed domain got
//! no row at all. Provisioning now falls back to the default; this job is what
//! repairs the accounts that predate it.
//!
//! The job must be additive and never a reassignment: membership is the billing
//! and authorization boundary, so moving a settled user is an operator
//! decision, not a boot side effect. That is the property most of these tests
//! exist to hold.

use systemprompt_web_admin::repositories::organizations::{crud, membership};
use systemprompt_web_jobs::internals::backfill_default_organization;

use crate::fixtures::{OrgSpec, insert_member, insert_org, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

async fn insert_default_org(pool: &sqlx::PgPool, id: &str) {
    insert_org(
        pool,
        &OrgSpec {
            is_default: true,
            ..OrgSpec::active(id, id)
        },
    )
    .await;
}

#[tokio::test]
async fn an_unattached_user_joins_the_default_organization() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let org_id = unique("org");
    insert_default_org(&db.pool, &org_id).await;
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("stray")).await;

    backfill_default_organization(&db.pool)
        .await
        .expect("backfill succeeds");

    let found = crud::find_organization_for_user(&db.pool, &user)
        .await
        .expect("membership lookup");
    assert_eq!(
        found.as_deref(),
        Some(org_id.as_str()),
        "a user with no membership must be joined to the default organization"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_user_who_already_belongs_somewhere_is_not_moved() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let default_org = unique("org");
    let their_org = unique("org");
    insert_default_org(&db.pool, &default_org).await;
    insert_org(&db.pool, &OrgSpec::active(&their_org, &their_org)).await;
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("settled")).await;
    insert_member(&db.pool, &user, &their_org, "owner").await;

    backfill_default_organization(&db.pool)
        .await
        .expect("backfill succeeds");

    let found = crud::find_organization_for_user(&db.pool, &user)
        .await
        .expect("membership lookup");
    assert_eq!(
        found.as_deref(),
        Some(their_org.as_str()),
        "a settled user must never be relocated by a boot job — membership is \
         the billing and authorization boundary"
    );
    let role = membership::find_member_role(&db.pool, &user, &their_org)
        .await
        .expect("role lookup");
    assert_eq!(
        role.as_deref(),
        Some("owner"),
        "and their role within it must survive untouched"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn running_twice_changes_nothing_the_second_time() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let org_id = unique("org");
    insert_default_org(&db.pool, &org_id).await;
    insert_user(&db.pool, &unique("user"), &unclaimed_email("idempotent")).await;

    let first = backfill_default_organization(&db.pool)
        .await
        .expect("first run");
    let second = backfill_default_organization(&db.pool)
        .await
        .expect("second run");

    assert!(
        first.joined >= 1,
        "the first run attaches the unattached user"
    );
    assert_eq!(
        second.joined, 0,
        "the second run must be a no-op — this runs on every boot"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn no_default_organization_is_a_warning_not_a_failure() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    // Why: a boot job that hard-fails on a configuration gap takes the whole
    // server down with it. An instance that has named no default organization
    // is misconfigured, not broken, and should say so in the log and start.
    let outcome = backfill_default_organization(&db.pool)
        .await
        .expect("backfill succeeds with nothing to do");

    assert_eq!(
        outcome.joined, 0,
        "with no default organization there is nowhere to put anyone"
    );
    db.cleanup().await;
}
