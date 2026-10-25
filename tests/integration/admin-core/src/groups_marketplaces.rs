//! `repositories::groups::marketplaces` — entitlement expressed as ordinary
//! access-control rules.

use systemprompt::identifiers::MarketplaceId;
use systemprompt_web_admin_core::types::groups::CreateGroupRequest;
use systemprompt_web_admin_identity::repositories::groups::crud::insert_group;
use systemprompt_web_admin_identity::repositories::groups::marketplaces::{
    list_group_marketplace_ids, set_group_marketplaces,
};
use systemprompt_web_shared::GroupId;

use crate::fixtures::{actor, unique_group};
use crate::tempdb::TempDb;

async fn seed_group(pool: &sqlx::PgPool, id: &GroupId) {
    insert_group(
        pool,
        &CreateGroupRequest {
            id: id.clone(),
            name: id.as_str().to_owned(),
            description: None,
        },
        "dashboard",
    )
    .await
    .expect("insert group");
}

#[tokio::test]
async fn setting_the_set_adds_and_withdraws_in_one_call() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let group = unique_group("grp");
    seed_group(&db.pool, &group).await;

    set_group_marketplaces(
        &db.pool,
        &group,
        &[
            MarketplaceId::new("dynapps-developers"),
            MarketplaceId::new("dynapps-cowork"),
        ],
        &actor(),
    )
    .await
    .expect("grant two");
    let mut granted = list_group_marketplace_ids(&db.pool, &group)
        .await
        .expect("read");
    granted.sort();
    assert_eq!(
        granted,
        vec![
            MarketplaceId::new("dynapps-cowork"),
            MarketplaceId::new("dynapps-developers")
        ]
    );

    set_group_marketplaces(
        &db.pool,
        &group,
        &[MarketplaceId::new("dynapps-cowork")],
        &actor(),
    )
    .await
    .expect("narrow to one");
    assert_eq!(
        list_group_marketplace_ids(&db.pool, &group)
            .await
            .expect("read"),
        vec![MarketplaceId::new("dynapps-cowork")],
        "a marketplace dropped from the set loses its rule"
    );
    db.cleanup().await;
}

// Why: the rules are keyed by group, so one group's edit must not read as, or
// disturb, another's.
#[tokio::test]
async fn one_groups_entitlement_does_not_leak_into_another() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let first = unique_group("grp");
    let second = unique_group("grp");
    seed_group(&db.pool, &first).await;
    seed_group(&db.pool, &second).await;

    set_group_marketplaces(
        &db.pool,
        &first,
        &[MarketplaceId::new("dynapps-developers")],
        &actor(),
    )
    .await
    .expect("grant");
    set_group_marketplaces(
        &db.pool,
        &second,
        &[MarketplaceId::new("dynapps-consultants")],
        &actor(),
    )
    .await
    .expect("grant");

    assert_eq!(
        list_group_marketplace_ids(&db.pool, &first)
            .await
            .expect("read"),
        vec![MarketplaceId::new("dynapps-developers")]
    );
    assert_eq!(
        list_group_marketplace_ids(&db.pool, &second)
            .await
            .expect("read"),
        vec![MarketplaceId::new("dynapps-consultants")]
    );
    db.cleanup().await;
}

// Why: `default_included` is the marketplace's own answer about the estate.
// A group screen that flipped it would silently re-entitle every other group.
#[tokio::test]
async fn setting_marketplaces_never_touches_the_entity_default() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let group = unique_group("grp");
    seed_group(&db.pool, &group).await;
    sqlx::query(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source) \
         VALUES ('marketplace', 'open-to-everyone', true, 'code:test-fixture') \
         ON CONFLICT (entity_type, entity_id) DO UPDATE SET default_included = true",
    )
    .execute(&*db.pool)
    .await
    .expect("seed an open marketplace");

    set_group_marketplaces(
        &db.pool,
        &group,
        &[MarketplaceId::new("open-to-everyone")],
        &actor(),
    )
    .await
    .expect("grant");

    let default_included: bool = sqlx::query_scalar(
        "SELECT default_included FROM access_control_entities \
         WHERE entity_type = 'marketplace' AND entity_id = 'open-to-everyone'",
    )
    .fetch_one(&*db.pool)
    .await
    .expect("read the entity");
    assert!(
        default_included,
        "granting one group must not narrow what the estate already has"
    );
    db.cleanup().await;
}
