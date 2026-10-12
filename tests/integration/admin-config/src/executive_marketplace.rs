//! Shipped executive and connection-scoped Salesforce access through the
//! runtime policy resolver.

use std::path::PathBuf;
use std::sync::Arc;

use systemprompt::identifiers::UserId;
use systemprompt::loader::ConfigLoader;
use systemprompt_security::authz::{
    AccessControlRepository, BulkKeepQuery, ChainSources, EntityKind, ParentChainIndex, allowed_ids,
};
use systemprompt_web_admin_core::authz::{dimensions, subject_attributes_for};
use systemprompt_web_admin_core::repositories::access_rules::declared_load::load_declared_set;
use systemprompt_web_admin_core::repositories::access_rules::sync::{SyncMode, apply_sync};
use systemprompt_web_admin_core::repositories::gateway_catalog::{
    dispatchable_route_ids, registered_routes,
};
use systemprompt_web_admin_identity::repositories::config::groups_yaml_loader;
use systemprompt_web_admin_platform::repositories::connectors::connector_accounts as accounts;
use systemprompt_web_admin_platform::salesforce_orgs;

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
    for id in ["dynapps-super-admin", "dynapps-salesforce"] {
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

#[test]
fn salesforce_catalog_is_exact_and_contains_no_collaboration_channel_content() {
    const EXPECTED: &[&str] = &[
        "account_context",
        "account_tiering",
        "call_follow_up",
        "call_prep",
        "close_plan",
        "customer_health",
        "customer_voice",
        "daily_briefing",
        "deal_advance_gap",
        "deal_review",
        "deal_signals",
        "deal_slip_scenario",
        "draft_outreach",
        "expansion_whitespace",
        "find_internal_answer",
        "forecast_narrative",
        "handle_objection",
        "lead_triage",
        "pipeline_review",
        "query_activity_history",
        "renewal_radar",
        "rep_context",
        "research_prospect",
        "salesforce_hygiene_check",
        "stakeholder_map",
        "team_pipeline",
        "using_display_widget",
        "weekly_wrap",
        "win_loss_review",
    ];
    let services = ConfigLoader::load_from_path(&services_path().join("config/config.yaml"))
        .expect("shipped services load and validate");
    let plugin = services
        .plugins
        .iter()
        .find(|(id, _)| id.as_str() == "dynapps-salesforce")
        .expect("Salesforce plugin exists")
        .1;
    let actual: Vec<&str> = plugin.skills.include.iter().map(String::as_str).collect();
    assert_eq!(actual, EXPECTED);
    assert!(
        !services
            .marketplaces
            .iter()
            .any(|(id, _)| id.as_str() == "dynapps-sales-uk")
    );

    for id in EXPECTED {
        let directory = services_path().join("skills").join(id);
        for file in [directory.join("config.yaml"), directory.join("SKILL.md")] {
            let body = std::fs::read_to_string(&file).expect("Salesforce skill file");
            assert!(
                !body.to_ascii_lowercase().contains("slack"),
                "{} contains excluded collaboration-channel content",
                file.display()
            );
        }
    }
}

#[tokio::test]
#[expect(
    unsafe_code,
    reason = "set_var is unsafe in edition 2024; see the SAFETY note"
)]
async fn executive_and_salesforce_connection_grant_only_the_intended_resources() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let path = services_path();
    let services =
        ConfigLoader::load_from_path(&path.join("config/config.yaml")).expect("services");
    // Why: a connection counts only when its connector is configured — the
    // MCP server enabled in the shipped catalog and its org in the registry —
    // and both are read from process-wide bootstraps, so this test installs
    // the shipped tree rather than the empty fixture the user helper falls
    // back to.
    systemprompt::loader::ServicesBootstrap::init_from_path(&path.join("config/config.yaml"))
        .expect("install shipped services");
    salesforce_orgs::SalesforceOrgRegistry::init_from_path(&path)
        .await
        .expect("install shipped Salesforce orgs");
    // Why: a connection is `not_provisioned` until the org's External Client
    // App credentials resolve; nextest runs each test in its own process, so
    // the environment override reaches only this one.
    // SAFETY: current-thread `#[tokio::test]` and nextest gives each test its
    // own process, so no other thread reads the environment.
    unsafe {
        std::env::set_var(
            salesforce_orgs::SHARED_CLIENT_ID_SECRET.to_uppercase(),
            "test-client-id",
        );
        std::env::set_var(
            salesforce_orgs::SHARED_CLIENT_SECRET.to_uppercase(),
            "test-client-secret",
        );
    }
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
    // super_admin alone — dynapps-super-admin absorbed the old
    // systemprompt-admin marketplace, and splitting the control plane in two is
    // exactly how an administrator ended up holding half of it. `sales` is
    // reached by business-users membership; a verified Salesforce connection
    // or legacy UK membership alone grants nothing.
    for (role, uk, business, connected, project, control_plane, sales) in [
        ("user", false, false, false, false, false, false),
        ("admin", false, false, false, false, true, false),
        ("platform_admin", false, false, false, false, true, false),
        ("knowledge_worker", false, false, false, false, false, false),
        ("project_manager", false, false, false, false, false, false),
        ("user", true, false, false, false, false, false),
        ("user", false, true, false, false, false, true),
        ("user", false, false, true, false, false, false),
        ("user", false, false, false, true, false, false),
        ("super_admin", false, false, false, false, true, false),
        ("super_admin", true, true, true, true, true, true),
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
        if business {
            sqlx::query(
                "INSERT INTO group_members (group_id,user_id,source) VALUES ('business-users',$1,'manual')",
            )
            .bind(user.as_str())
            .execute(&*db.pool)
            .await
            .expect("business-users membership");
        }
        if project {
            set_project(&db.pool, user.as_str(), Some("business-implementation")).await;
        }
        if connected {
            let mut tx = db.pool.begin().await.expect("begin connector update");
            let mut row = accounts::get_locked_account(&mut tx, &user, "salesforce")
                .await
                .expect("connector account");
            row.status = "connected".into();
            row.auth_method = Some("oauth".into());
            row.verified_at = Some(chrono::Utc::now());
            accounts::update_account(&mut tx, &user, &row)
                .await
                .expect("mark connector ready");
            tx.commit().await.expect("commit connector update");
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
            (EntityKind::Marketplace, "dynapps-salesforce", sales),
            (EntityKind::Plugin, "dynapps-super-admin", control_plane),
            (EntityKind::Plugin, "dynapps-salesforce", sales),
            (EntityKind::Skill, "admin_critical_projects", control_plane),
            (EntityKind::Skill, "admin_ai_usage", control_plane),
            (EntityKind::Skill, "admin_daily_brief", control_plane),
            (EntityKind::Skill, "account_context", sales),
            (EntityKind::Skill, "pipeline_review", sales),
            (EntityKind::Skill, "using_display_widget", sales),
            (EntityKind::McpServer, "salesforce", true),
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
                "{role}, uk={uk}, business={business}, connected={connected}, project={project}: {id}"
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
