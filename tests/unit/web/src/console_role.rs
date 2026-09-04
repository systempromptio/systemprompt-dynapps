//! The `project_manager` semi-admin role: which role sets reach the admin
//! dashboard, how the AD group glob resolves to roles, and the two derived
//! views (`ProjectScope`, history scope) that widen for it.
//!
//! The route gates themselves are pinned by the contract suite; this covers
//! the pure rules underneath them.

use std::collections::BTreeMap;

use systemprompt::identifiers::{Email, UserId};
use systemprompt_web_admin::repositories::analytics::conversations::has_full_history_view;
use systemprompt_web_admin::types::{
    Project, ROLE_ADMIN, ROLE_PROJECT_MANAGER, ROLE_USER, roles_grant_console,
};
use systemprompt_web_admin::util::project_scope::ProjectScope;
use systemprompt_web_admin::{AdfsConfig, UserContext, group_matches_pattern};

fn roles(list: &[&str]) -> Vec<String> {
    list.iter().map(|r| (*r).to_owned()).collect()
}

fn ctx(list: &[&str], project: Option<Project>) -> UserContext {
    let roles = roles(list);
    UserContext {
        user_id: UserId::new("u-1".to_owned()),
        username: "u-1".to_owned(),
        email: Email::try_new("u-1@example.test".to_owned()).expect("fixture email"),
        project,
        is_admin: roles.iter().any(|r| r == ROLE_ADMIN),
        is_console: roles_grant_console(&roles),
        roles,
        email_verified: true,
        session_id: None,
    }
}

#[test]
fn only_admin_and_project_manager_reach_the_console() {
    assert!(roles_grant_console(&roles(&[ROLE_ADMIN])));
    assert!(roles_grant_console(&roles(&[
        ROLE_PROJECT_MANAGER,
        ROLE_USER
    ])));
    assert!(!roles_grant_console(&roles(&[ROLE_USER])));
    assert!(!roles_grant_console(&roles(&["auditor"])));
    assert!(!roles_grant_console(&[]));
}

#[test]
fn a_project_manager_is_not_an_admin() {
    let pm = ctx(&[ROLE_PROJECT_MANAGER, ROLE_USER], Some(Project::Core));
    assert!(pm.is_console);
    assert!(
        !pm.is_admin,
        "is_admin still guards every privileged mutation and must mean the admin role alone"
    );
}

#[test]
fn the_group_glob_matches_prefix_and_suffix() {
    assert!(group_matches_pattern(
        "Systemprompt-ProjectManagers-*",
        "Systemprompt-ProjectManagers-UK"
    ));
    assert!(group_matches_pattern(
        "Systemprompt-ProjectManagers-*",
        "Systemprompt-ProjectManagers-India"
    ));
    assert!(!group_matches_pattern(
        "Systemprompt-ProjectManagers-*",
        "Systemprompt-ProjectManagers"
    ));
    assert!(!group_matches_pattern(
        "Systemprompt-ProjectManagers-*",
        "Systemprompt-Admins"
    ));
    assert!(group_matches_pattern(
        "Systemprompt-Admins",
        "Systemprompt-Admins"
    ));
    assert!(!group_matches_pattern(
        "Systemprompt-Admins",
        "systemprompt-admins"
    ));
}

fn pattern_config() -> AdfsConfig {
    let mut cfg = AdfsConfig::disabled();
    cfg.group_roles = BTreeMap::from([(
        "Systemprompt-Admins".to_owned(),
        vec![ROLE_ADMIN.to_owned(), ROLE_USER.to_owned()],
    )]);
    cfg.group_role_patterns = BTreeMap::from([(
        "Systemprompt-ProjectManagers-*".to_owned(),
        vec![ROLE_PROJECT_MANAGER.to_owned(), ROLE_USER.to_owned()],
    )]);
    cfg.group_projects = BTreeMap::from([("Systemprompt-Core".to_owned(), Project::Core)]);
    cfg
}

#[test]
fn a_regional_pm_group_maps_to_the_role() {
    let cfg = pattern_config();
    assert_eq!(
        cfg.roles_for_groups(&roles(&["Systemprompt-ProjectManagers-Europe"])),
        vec![ROLE_PROJECT_MANAGER.to_owned(), ROLE_USER.to_owned()]
    );
}

#[test]
fn patterns_are_applied_on_top_of_exact_groups() {
    let cfg = pattern_config();
    let granted = cfg.roles_for_groups(&roles(&[
        "Systemprompt-Admins",
        "Systemprompt-ProjectManagers-UK",
    ]));
    assert_eq!(
        granted,
        vec![
            ROLE_ADMIN.to_owned(),
            ROLE_PROJECT_MANAGER.to_owned(),
            ROLE_USER.to_owned()
        ]
    );
}

#[test]
fn an_unmapped_group_still_grants_nothing() {
    let cfg = pattern_config();
    assert!(cfg.roles_for_groups(&roles(&["Domain-Users"])).is_empty());
}

#[test]
fn a_pattern_matched_group_counts_as_mapped() {
    let cfg = pattern_config();
    assert_eq!(
        cfg.mapped_groups(&roles(&["Domain-Users", "Systemprompt-ProjectManagers-UK"])),
        vec!["Systemprompt-ProjectManagers-UK".to_owned()]
    );
}

#[test]
fn a_project_manager_spans_every_project() {
    let pm = ctx(&[ROLE_PROJECT_MANAGER, ROLE_USER], None);
    assert_eq!(ProjectScope::resolve(&pm, None), ProjectScope::All);
    assert_eq!(
        ProjectScope::resolve(&pm, Some("commerce")),
        ProjectScope::Only(Project::Commerce)
    );
}

#[test]
fn a_plain_user_stays_pinned_to_their_own_project() {
    let user = ctx(&[ROLE_USER], Some(Project::Core));
    assert_eq!(
        ProjectScope::resolve(&user, Some("commerce")),
        ProjectScope::Only(Project::Core)
    );
}

#[test]
fn a_project_manager_sees_every_conversation() {
    assert!(has_full_history_view(&ctx(
        &[ROLE_PROJECT_MANAGER, ROLE_USER],
        None
    )));
    assert!(!has_full_history_view(&ctx(
        &[ROLE_USER],
        Some(Project::Core)
    )));
}
