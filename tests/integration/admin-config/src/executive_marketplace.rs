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
        "SELECT EXISTS(SELECT 1 FROM projects WHERE id = 'business-implementation')",
    )
    .fetch_one(&*db.pool)
    .await
    .expect("configured project");
    assert!(project_exists);
    let repo = AccessControlRepository::from_pool(Arc::clone(&db.pool));
    let chains = ParentChainIndex::load(&repo, Arc::new(ChainSources::from_services(&services)))
        .await
        .expect("parent policies");
    // Why: `control_plane` is reached by every administrator role, not by
    // super_admin alone — astound-super-admin absorbed the old
    // systemprompt-admin marketplace, and splitting the control plane in two is
    // exactly how an administrator ended up holding half of it. `sales` is the
    // UK bundle, reached by the group or by super_admin's own role grant.
    for (role, uk, project, control_plane, sales) in [
        ("user", false, false, false, false),
        ("admin", false, false, true, false),
        ("platform_admin", false, false, true, false),
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
            set_project(&db.pool, user.as_str(), Some("business-implementation")).await;
        }
        let attributes = subject_attributes_for(&db.pool, &user)
            .await
            .expect("attributes");
        for (kind, id, expected) in [
            (
                EntityKind::Marketplace,
                "astound-super-admin",
                control_plane,
            ),
            (EntityKind::Marketplace, "astound-sales-uk", uk),
            (EntityKind::Plugin, "astound-super-admin", control_plane),
            (EntityKind::Plugin, "astound-sales-uk", sales),
            (EntityKind::Skill, "admin_critical_projects", control_plane),
            (EntityKind::Skill, "admin_ai_usage", control_plane),
            (EntityKind::Skill, "admin_daily_brief", control_plane),
            (EntityKind::Skill, "uk_sales_client_discovery", sales),
            (EntityKind::Skill, "uk_sales_client_prep", sales),
            (EntityKind::Skill, "uk_sales_client_report", sales),
            (EntityKind::Skill, "uk_sales_client_research", sales),
            (EntityKind::McpServer, "google_rag", sales),
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

#[tokio::test]
async fn executive_manual_role_can_be_saved_and_survives_directory_recomputation() {
    use systemprompt_web_admin::repositories::users::roles;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = UserId::new(unique("executive-role"));
    insert_user_with_roles(&db.pool, user.as_str(), &["user".into()]).await;
    roles::set_manual_roles(&db.pool, &user, &["super_admin".into()], &user)
        .await
        .expect("manual role database constraint admits super_admin");
    let effective = roles::recompute_roles(&db.pool, &user, Some(&["user".into()]))
        .await
        .expect("directory sign-in");
    assert!(effective.iter().any(|role| role == "super_admin"));
    db.cleanup().await;
}

#[tokio::test]
async fn rollout_project_correction_preserves_membership_and_defaults() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = UserId::new(unique("project-rename"));
    insert_user_with_roles(&db.pool, user.as_str(), &["user".into()]).await;
    set_project(&db.pool, user.as_str(), Some("business-imeplmentation")).await;
    sqlx::query("INSERT INTO user_scope_defaults (user_id, primary_project_id, source) VALUES ($1,'business-imeplmentation','manual')")
        .bind(user.as_str()).execute(&*db.pool).await.expect("default");
    let migration = include_str!(
        "../../../../extensions/web/schema/migrations/052_executive_access_project_name.sql"
    );
    for _ in 0..2 {
        sqlx::raw_sql(migration)
            .execute(&*db.pool)
            .await
            .expect("idempotent migration");
    }
    let project: String =
        sqlx::query_scalar("SELECT project_id FROM project_members WHERE user_id=$1")
            .bind(user.as_str())
            .fetch_one(&*db.pool)
            .await
            .expect("membership preserved");
    let default: String =
        sqlx::query_scalar("SELECT primary_project_id FROM user_scope_defaults WHERE user_id=$1")
            .bind(user.as_str())
            .fetch_one(&*db.pool)
            .await
            .expect("default preserved");
    assert_eq!(project, "business-implementation");
    assert_eq!(default, project);
    let old: i64 =
        sqlx::query_scalar("SELECT count(*) FROM projects WHERE id='business-imeplmentation'")
            .fetch_one(&*db.pool)
            .await
            .expect("old project removed");
    assert_eq!(old, 0);
    db.cleanup().await;
}
