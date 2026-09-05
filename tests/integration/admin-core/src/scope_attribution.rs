//! Exclusive attribution against the database: a person in two groups counts
//! once, the group totals partition the instance, and manual choices survive
//! recomputation.

use systemprompt_web_admin::repositories::people_usage::get_scope_usage;
use systemprompt_web_admin::repositories::people_usage::totals::list_scope_totals;
use systemprompt_web_admin::repositories::scope::defaults::{
    find_scope_defaults, recompute_scope_defaults, set_scope_defaults,
};
use systemprompt_web_admin::repositories::scope::{Attribution, ScopeKind, ScopeQuery};

use crate::fixtures::{
    RequestSpec, insert_group, insert_group_member, insert_request, insert_user, unclaimed_email,
    unique,
};
use crate::tempdb::TempDb;

const WINDOW: i32 = 30;

#[tokio::test]
async fn a_member_of_two_groups_counts_once_exclusively_and_twice_as_a_member() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let first = unique("grp");
    let second = unique("grp");
    insert_group(&db.pool, &first, &first).await;
    insert_group(&db.pool, &second, &second).await;
    let person = insert_user(&db.pool, &unique("user"), &unclaimed_email("both")).await;
    insert_group_member(&db.pool, &first, &person, "adfs").await;
    insert_group_member(&db.pool, &second, &person, "adfs").await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &person)).await;
    recompute_scope_defaults(&db.pool)
        .await
        .expect("recompute defaults");

    let mut member_total = 0i64;
    for id in [first.as_str(), second.as_str()] {
        member_total += get_scope_usage(
            &db.pool,
            &ScopeQuery::new(ScopeKind::Group, Attribution::Member, id, WINDOW),
        )
        .await
        .expect("member usage")
        .requests;
    }
    assert_eq!(
        member_total, 2,
        "membership attribution counts the same request under both groups"
    );

    let mut exclusive_total = 0i64;
    for id in [first.as_str(), second.as_str()] {
        exclusive_total += get_scope_usage(
            &db.pool,
            &ScopeQuery::new(ScopeKind::Group, Attribution::Exclusive, id, WINDOW),
        )
        .await
        .expect("exclusive usage")
        .requests;
    }
    assert_eq!(
        exclusive_total, 1,
        "exclusive attribution files the request under the primary group only"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn exclusive_group_totals_sum_to_the_instance_total() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let group = unique("grp");
    insert_group(&db.pool, &group, &group).await;
    let member = insert_user(&db.pool, &unique("user"), &unclaimed_email("member")).await;
    let unplaced = insert_user(&db.pool, &unique("user"), &unclaimed_email("unplaced")).await;
    insert_group_member(&db.pool, &group, &member, "adfs").await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &member)).await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &unplaced)).await;
    recompute_scope_defaults(&db.pool)
        .await
        .expect("recompute defaults");

    let rows = list_scope_totals(&db.pool, ScopeKind::Group, Attribution::Exclusive, WINDOW)
        .await
        .expect("totals");
    let instance: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::BIGINT FROM ai_requests
         WHERE created_at >= NOW() - INTERVAL '30 days'",
    )
    .fetch_one(&*db.pool)
    .await
    .expect("instance total");

    assert_eq!(
        rows.iter().map(|row| row.requests).sum::<i64>(),
        instance,
        "every request lands in exactly one bucket, unattributed included"
    );
    assert!(
        rows.iter().any(|row| row.scope_id == group),
        "the seeded group is one of the buckets"
    );

    db.cleanup().await;
}

// Why: the recomputation job runs hourly, so a manual choice that it could
// overwrite would last less than an hour and mean nothing.
#[tokio::test]
async fn recomputation_leaves_a_manual_choice_alone() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let auto = unique("grp");
    let chosen = unique("grp");
    insert_group(&db.pool, &auto, &auto).await;
    insert_group(&db.pool, &chosen, &chosen).await;
    let person = insert_user(&db.pool, &unique("user"), &unclaimed_email("manual")).await;
    insert_group_member(&db.pool, &auto, &person, "adfs").await;
    recompute_scope_defaults(&db.pool)
        .await
        .expect("recompute defaults");

    set_scope_defaults(&db.pool, &person, Some(&chosen), None)
        .await
        .expect("set defaults");
    recompute_scope_defaults(&db.pool)
        .await
        .expect("recompute again");

    let stored = find_scope_defaults(&db.pool, &person)
        .await
        .expect("find defaults")
        .expect("a row");
    assert_eq!(stored.primary_group_id.as_deref(), Some(chosen.as_str()));
    assert_eq!(stored.source, "manual");

    db.cleanup().await;
}

#[tokio::test]
async fn a_person_in_one_group_gets_it_as_their_primary() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let group = unique("grp");
    insert_group(&db.pool, &group, &group).await;
    let person = insert_user(&db.pool, &unique("user"), &unclaimed_email("single")).await;
    insert_group_member(&db.pool, &group, &person, "adfs").await;
    recompute_scope_defaults(&db.pool)
        .await
        .expect("recompute defaults");

    let stored = find_scope_defaults(&db.pool, &person)
        .await
        .expect("find defaults")
        .expect("a row");
    assert_eq!(stored.primary_group_id.as_deref(), Some(group.as_str()));
    assert_eq!(stored.source, "auto");

    db.cleanup().await;
}
