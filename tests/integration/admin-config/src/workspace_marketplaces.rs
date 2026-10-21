//! The Dynapps workspaces (`dynapps-portal`, which carries the showcase and
//! farm plugins, and the
//! agent-farm workspace `dynapps-engineering`) resolve through the runtime
//! policy resolver against the shipped `rules.yaml`: each one reaches the
//! principals it names and stays hidden from a plain signed-in user.

use std::path::PathBuf;
use std::sync::Arc;

use systemprompt::governance::authz::{
    AccessControlRepository, BulkKeepQuery, ChainSources, EntityKind, ParentChainIndex, allowed_ids,
};
use systemprompt::identifiers::UserId;
use systemprompt::loader::ConfigLoader;
use systemprompt_web_admin_core::authz::{dimensions, subject_attributes_for};
use systemprompt_web_admin_core::repositories::access_rules::declared_load::load_declared_set;
use systemprompt_web_admin_core::repositories::access_rules::sync::{SyncMode, apply_sync};
use systemprompt_web_admin_core::repositories::gateway_catalog::{
    dispatchable_route_ids, registered_routes,
};
use systemprompt_web_admin_identity::repositories::config::groups_yaml_loader;

use crate::fixtures::{insert_user_with_roles, unique};
use crate::tempdb::TempDb;

const WORKSPACES: [&str; 2] = ["dynapps-portal", "dynapps-engineering"];

fn services_path() -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !root.join("services/config/config.yaml").is_file() {
        assert!(root.pop(), "repository services tree");
    }
    root.join("services")
}

#[test]
fn workspace_marketplaces_ship_with_resolvable_plugins() {
    let services = ConfigLoader::load_from_path(&services_path().join("config/config.yaml"))
        .expect("shipped services load and validate");
    for id in WORKSPACES {
        let marketplace = services
            .marketplaces
            .iter()
            .find(|(key, _)| key.as_str() == id)
            .unwrap_or_else(|| panic!("marketplace {id} exists"))
            .1;
        assert!(
            !marketplace.plugins.include.is_empty(),
            "{id} carries plugins"
        );
        for plugin in &marketplace.plugins.include {
            assert!(
                services
                    .plugins
                    .iter()
                    .any(|(key, _)| key.as_str() == plugin),
                "{id} names unknown plugin {plugin}"
            );
        }
    }
}

#[tokio::test]
async fn workspace_marketplaces_reach_only_entitled_principals() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let path = services_path();
    let services =
        ConfigLoader::load_from_path(&path.join("config/config.yaml")).expect("services");
    // Why: the parent-chain index reads the shipped catalog from the
    // process-wide services bootstrap, as in `executive_marketplace`.
    systemprompt::loader::ServicesBootstrap::init_from_path(&path.join("config/config.yaml"))
        .expect("install shipped services");
    groups_yaml_loader::load_groups_from_yaml(&db.pool, &path)
        .await
        .expect("groups");
    let marketplace_ids: Vec<String> = services
        .marketplaces
        .keys()
        .map(|id| id.as_str().to_owned())
        .collect();
    let registered = registered_routes(&dispatchable_route_ids(&services));
    let declared = load_declared_set(&path, &marketplace_ids, &registered)
        .await
        .expect("rules.yaml");
    apply_sync(&db.pool, &declared, SyncMode::Overwrite)
        .await
        .expect("seed");
    let repo = AccessControlRepository::from_pool(Arc::clone(&db.pool));
    let chains = ParentChainIndex::load(&repo, Arc::new(ChainSources::from_services(&services)))
        .await
        .expect("parent policies");
    // (role, staff member, portal, engineering)
    for (role, staff, portal, engineering) in [
        ("user", false, false, false),
        ("user", true, true, false),
        ("developer", false, false, true),
        ("project_manager", false, false, true),
        ("admin", false, true, true),
    ] {
        let user = UserId::new(unique("workspace-access"));
        let roles = vec!["user".to_owned(), role.to_owned()];
        insert_user_with_roles(&db.pool, user.as_str(), &roles).await;
        if staff {
            sqlx::query(
                "INSERT INTO group_members (group_id,user_id,source) VALUES ('staff',$1,'manual')",
            )
            .bind(user.as_str())
            .execute(&*db.pool)
            .await
            .expect("staff membership");
        }
        let attributes = subject_attributes_for(&db.pool, &user)
            .await
            .expect("attributes");
        let ids: Vec<String> = WORKSPACES.iter().map(|id| (*id).to_owned()).collect();
        let allowed = allowed_ids(
            &repo,
            BulkKeepQuery {
                user_id: &user,
                roles: &roles,
                kind: EntityKind::Marketplace,
                ids: &ids,
                chains: &chains,
                attributes: &attributes,
                dimensions: dimensions(&db.pool),
            },
        )
        .await
        .expect("runtime policy");
        for (id, expected) in WORKSPACES.iter().zip([portal, engineering]) {
            assert_eq!(
                allowed.contains(*id),
                expected,
                "{role}, staff={staff}: marketplace {id}"
            );
        }
    }
    db.cleanup().await;
}
