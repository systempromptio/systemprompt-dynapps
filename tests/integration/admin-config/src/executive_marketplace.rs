//! Shipped executive and UK sales access through the runtime policy resolver.

use std::path::PathBuf;
use std::sync::Arc;

use systemprompt::identifiers::UserId;
use systemprompt::loader::ConfigLoader;
use systemprompt_security::authz::{
    AccessControlRepository, BulkKeepQuery, ChainSources, EntityKind, ParentChainIndex,
    RegisteredEntities, allowed_ids,
};
use systemprompt_web_admin::authz::{dimensions, subject_attributes_for};
use systemprompt_web_admin::repositories::config::{
    acl_yaml_loader, groups_yaml_loader, member_grants_yaml_loader,
};

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
    for id in ["astound-super-admin", "astound-sales-uk"] {
        let marketplace = services
            .marketplaces
            .iter()
            .find(|(key, _)| key.as_str() == id)
            .expect("marketplace exists")
            .1;
        assert!(!marketplace.access.default_included);
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
async fn executive_and_uk_membership_grant_only_the_intended_resources() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let path = services_path();
    let services =
        ConfigLoader::load_from_path(&path.join("config/config.yaml")).expect("services");
    groups_yaml_loader::load_groups_from_yaml(&db.pool, &path)
        .await
        .expect("groups");
    acl_yaml_loader::load_from_yaml(&db.pool, &path, &RegisteredEntities::default())
        .await
        .expect("roles");
    acl_yaml_loader::ingest_marketplace_map(&db.pool, &services.marketplaces)
        .await
        .expect("marketplaces");
    member_grants_yaml_loader::load_member_grants_from_yaml(&db.pool, &path)
        .await
        .expect("member grants");
    let project_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id = 'business-imeplmentation')",
    )
    .fetch_one(&*db.pool)
    .await
    .expect("configured project");
    assert!(project_exists);
    let repo = AccessControlRepository::from_pool(Arc::clone(&db.pool));
    let chains = ParentChainIndex::load(&repo, Arc::new(ChainSources::from_services(&services)))
        .await
        .expect("parent policies");
    for (role, uk, project, executive, sales) in [
        ("user", false, false, false, false),
        ("admin", false, false, false, false),
        ("platform_admin", false, false, false, false),
        ("knowledge_worker", false, false, false, false),
        ("project_manager", false, false, false, false),
        ("user", true, false, false, true),
        ("user", false, true, false, false),
        ("super_admin", false, false, true, true),
        ("super_admin", true, true, true, true),
    ] {
        let user = UserId::new(unique("executive-access"));
        let roles = vec!["user".to_owned(), role.to_owned()];
        insert_user_with_roles(&db.pool, user.as_str(), &roles).await;
        if uk {
            sqlx::query(
                "INSERT INTO group_members (group_id,user_id,source) VALUES ('uk',$1,'manual')",
            )
            .bind(user.as_str())
            .execute(&*db.pool)
            .await
            .expect("UK membership");
        }
        if project {
            set_project(&db.pool, user.as_str(), Some("business-imeplmentation")).await;
        }
        let attributes = subject_attributes_for(&db.pool, &user)
            .await
            .expect("attributes");
        for (kind, id, expected) in [
            (EntityKind::Marketplace, "astound-super-admin", executive),
            (EntityKind::Marketplace, "astound-sales-uk", uk),
            (EntityKind::Plugin, "astound-executive-insights", executive),
            (EntityKind::Plugin, "astound-sales-uk", sales),
            (EntityKind::Skill, "executive_portfolio_health", executive),
            (EntityKind::Skill, "executive_blocker_decisions", executive),
            (EntityKind::Skill, "executive_emergency_recovery", executive),
            (EntityKind::Skill, "uk_sales_client_discovery", sales),
            (EntityKind::Skill, "uk_sales_account_report", sales),
            (EntityKind::Skill, "uk_sales_client_research", sales),
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
                "{role}, uk={uk}, project={project}: {id}"
            );
        }
    }
    db.cleanup().await;
}
