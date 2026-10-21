//! Shipped executive (control-plane) access through the runtime policy
//! resolver.

use std::path::PathBuf;
use std::sync::Arc;

use systemprompt::identifiers::UserId;
use systemprompt::loader::ConfigLoader;
use systemprompt::governance::authz::{
    AccessControlRepository, BulkKeepQuery, ChainSources, EntityKind, ParentChainIndex, allowed_ids,
};
use systemprompt_web_admin_core::authz::{dimensions, subject_attributes_for};
use systemprompt_web_admin_core::repositories::access_rules::declared_load::load_declared_set;
use systemprompt_web_admin_core::repositories::access_rules::sync::{SyncMode, apply_sync};
use systemprompt_web_admin_core::repositories::gateway_catalog::{
    dispatchable_route_ids, registered_routes,
};
use systemprompt_web_admin_identity::repositories::config::groups_yaml_loader;

use crate::fixtures::{insert_user_with_roles, set_project, unique};
use crate::tempdb::TempDb;

fn services_path() -> PathBuf {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !root.join("services/config/config.yaml").is_file() {
        assert!(root.pop(), "repository services tree");
    }
    root.join("services")
}

#[test]
fn executive_services_load_with_resolvable_plugin_members() {
    let services = ConfigLoader::load_from_path(&services_path().join("config/config.yaml"))
        .expect("shipped services load and validate");
    for id in ["dynapps-super-admin"] {
        let marketplace = services
            .marketplaces
            .iter()
            .find(|(key, _)| key.as_str() == id)
            .expect("marketplace exists")
            .1;
        for plugin in &marketplace.plugins.include {
            assert!(
                services
                    .plugins
                    .iter()
                    .any(|(key, _)| key.as_str() == plugin)
            );
        }
    }
}

#[tokio::test]
async fn executive_access_grants_only_the_control_plane() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let path = services_path();
    let services =
        ConfigLoader::load_from_path(&path.join("config/config.yaml")).expect("services");
    // Why: the parent-chain index reads the shipped catalog from the
    // process-wide services bootstrap, so this test installs the shipped tree
    // rather than the empty fixture the user helper falls back to.
    systemprompt::loader::ServicesBootstrap::init_from_path(&path.join("config/config.yaml"))
        .expect("install shipped services");
    groups_yaml_loader::load_groups_from_yaml(&db.pool, &path)
        .await
        .expect("groups");
    // Why: the shipped rules.yaml, projected the way boot seeds it, so the
    // assertions below hold against the file the release actually carries.
    let marketplace_ids: Vec<String> = services
        .marketplaces
        .keys()
        .map(|id| id.as_str().to_owned())
        .collect();
    // Why: the file declares `gateway_route/*`, which expands over the live
    // route catalog — the same one boot registers — never the database.
    let registered = registered_routes(&dispatchable_route_ids(&services));
    let declared = load_declared_set(&path, &marketplace_ids, &registered)
        .await
        .expect("rules.yaml");
    apply_sync(&db.pool, &declared, SyncMode::Overwrite)
        .await
        .expect("seed");
    let project_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM projects WHERE id = 'odoo-delivery')")
            .fetch_one(&*db.pool)
            .await
            .expect("configured project");
    assert!(project_exists);
    let repo = AccessControlRepository::from_pool(Arc::clone(&db.pool));
    let chains = ParentChainIndex::load(&repo, Arc::new(ChainSources::from_services(&services)))
        .await
        .expect("parent policies");
    // Why: `control_plane` is reached by every administrator role, not by
    // super_admin alone — dynapps-super-admin absorbed the old
    // systemprompt-admin marketplace, and splitting the control plane in two is
    // exactly how an administrator ended up holding half of it. Staff
    // membership or a project assignment alone grants nothing.
    for (role, staff, project, control_plane) in [
        ("user", false, false, false),
        ("admin", false, false, true),
        ("platform_admin", false, false, true),
        ("knowledge_worker", false, false, false),
        ("project_manager", false, false, false),
        ("user", true, false, false),
        ("user", false, true, false),
        ("super_admin", false, false, true),
        ("super_admin", true, true, true),
    ] {
        let user = UserId::new(unique("executive-access"));
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
        if project {
            set_project(&db.pool, user.as_str(), Some("odoo-delivery")).await;
        }
        let attributes = subject_attributes_for(&db.pool, &user)
            .await
            .expect("attributes");
        for (kind, id, expected) in [
            (
                EntityKind::Marketplace,
                "dynapps-super-admin",
                control_plane,
            ),
            (EntityKind::Plugin, "dynapps-super-admin", control_plane),
            (EntityKind::Skill, "systemprompt_cli", control_plane),
        ] {
            let ids = vec![id.to_owned()];
            let allowed = allowed_ids(
                &repo,
                BulkKeepQuery {
                    user_id: &user,
                    roles: &roles,
                    kind,
                    ids: &ids,
                    chains: &chains,
                    attributes: &attributes,
                    dimensions: dimensions(&db.pool),
                },
            )
            .await
            .expect("runtime policy");
            assert_eq!(
                allowed.contains(id),
                expected,
                "{role}, staff={staff}, project={project}: {id}"
            );
        }
    }
    db.cleanup().await;
}

#[tokio::test]
async fn executive_manual_role_can_be_saved_and_survives_directory_recomputation() {
    use systemprompt_web_admin_identity::repositories::users::roles;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = UserId::new(unique("executive-role"));
    insert_user_with_roles(&db.pool, user.as_str(), &["user".into()]).await;
    roles::set_manual_roles(&db.pool, &user, &["super_admin".into()], &user, None)
        .await
        .expect("manual role database constraint admits super_admin");
    let effective = roles::recompute_roles(&db.pool, &user, Some(&["user".into()]))
        .await
        .expect("directory sign-in");
    assert!(effective.iter().any(|role| role == "super_admin"));
    db.cleanup().await;
}
