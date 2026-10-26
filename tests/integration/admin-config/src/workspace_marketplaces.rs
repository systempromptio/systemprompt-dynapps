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
    let view = dynapps_test_common::extension_services(
        &db.pool,
        dynapps_test_common::services_view(&path.join("config/config.yaml")),
    );
    groups_yaml_loader::load_groups_from_yaml(&db.pool, &path)
        .await
        .expect("groups");
    crate::fixtures::apply_access_rules(&db.pool, &path).await;
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
        let attributes = view.subject_attributes(&user).await.expect("attributes");
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
                dimensions: view.dimensions(),
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
