//! Listing scope: what a caller may see, and what a query string is allowed
//! to narrow it to.
//!
//! The rule that matters is asymmetric. A console caller may ask for any
//! group; anyone else gets their own groups whatever they ask for, and asking
//! for a group they are not in is silently ignored rather than refused — a
//! 403 there would confirm the group exists.

use systemprompt::identifiers::{Email, UserId};
use systemprompt_web_admin::repositories::scope::{
    Participation, ScopeRequest, SubjectScope, Visibility,
};
use systemprompt_web_admin::types::{UserContext, roles_grant_console, roles_grant_manage};

fn owned(list: &[&str]) -> Vec<String> {
    list.iter().map(|g| (*g).to_owned()).collect()
}

// Why: a non-console caller's view is their marketplace participation, so the
// fixture's "groups" are the containers that participation names.
fn ctx(roles: &[&str], groups: &[&str]) -> UserContext {
    let roles: Vec<String> = owned(roles);
    let is_console = roles_grant_console(&roles);
    UserContext {
        user_id: UserId::new("u-1".to_owned()),
        username: "u-1".to_owned(),
        email: Email::try_new("u-1@example.test".to_owned()).expect("fixture email"),
        group_ids: owned(groups),
        project_ids: Vec::new(),
        participation: if is_console || groups.is_empty() {
            Participation::default()
        } else {
            Participation {
                marketplaces: vec![systemprompt::identifiers::MarketplaceId::new("m-1")],
                groups: owned(groups),
                projects: vec!["storefront".to_owned()],
            }
        },
        is_admin: roles_grant_manage(&roles),
        is_console,
        is_platform_admin: roles.iter().any(|r| r == "platform_admin"),
        is_developer: roles.iter().any(|r| r == "developer"),
        roles,
        email_verified: true,
        session_id: None,
    }
}

#[test]
fn a_console_caller_sees_every_group() {
    assert_eq!(
        Visibility::for_user(&ctx(&["admin"], &["europe-devs"])),
        Visibility::All
    );
    assert_eq!(
        Visibility::for_user(&ctx(&["platform_admin"], &[])),
        Visibility::All
    );
}

// Why: `project_manager` used to be a console role; it is now an
// entitlement role like `developer`, scoped by participation.
#[test]
fn a_project_manager_is_scoped_like_everyone_else() {
    assert_eq!(
        Visibility::for_user(&ctx(&["project_manager", "user"], &["uk"])),
        Visibility::Scoped {
            groups: vec!["uk".to_owned()],
            projects: vec!["storefront".to_owned()],
            self_id: "u-1".to_owned(),
        }
    );
}

#[test]
fn everyone_else_sees_the_containers_their_marketplaces_name() {
    assert_eq!(
        Visibility::for_user(&ctx(&["user"], &["europe-devs", "business-users"])),
        Visibility::Scoped {
            groups: vec!["europe-devs".to_owned(), "business-users".to_owned()],
            projects: vec!["storefront".to_owned()],
            self_id: "u-1".to_owned(),
        }
    );
}

#[test]
fn a_user_with_no_participation_narrows_to_nothing_rather_than_everything() {
    let request = ScopeRequest::from_query(&ctx(&["user"], &[]), None, None);
    assert_eq!(request.group_filter(), Some(Vec::new()));
    assert_eq!(request.project_filter(), Some(Vec::new()));
    assert_eq!(
        request.self_id(),
        Some("u-1"),
        "their own rows stay inside the view"
    );
}

#[test]
fn a_console_caller_may_ask_for_any_group() {
    let request = ScopeRequest::from_query(&ctx(&["admin"], &[]), Some("india-devs"), None);
    assert_eq!(request.group, Some("india-devs".to_owned()));
    assert_eq!(request.group_filter(), Some(vec!["india-devs".to_owned()]));
}

#[test]
fn asking_for_a_group_you_are_not_in_is_ignored_not_refused() {
    let request =
        ScopeRequest::from_query(&ctx(&["user"], &["europe-devs"]), Some("india-devs"), None);
    assert_eq!(request.group, None, "the filter is dropped, not honoured");
    assert_eq!(
        request.group_filter(),
        Some(vec!["europe-devs".to_owned()]),
        "and the caller still sees their own group"
    );
}

#[test]
fn asking_for_a_group_you_are_in_narrows_to_it() {
    let request = ScopeRequest::from_query(
        &ctx(&["user"], &["europe-devs", "business-users"]),
        Some("europe-devs"),
        None,
    );
    assert_eq!(request.group_filter(), Some(vec!["europe-devs".to_owned()]));
}

#[test]
fn a_console_caller_may_ask_for_any_project() {
    let request = ScopeRequest::from_query(&ctx(&["admin"], &[]), None, Some("core"));
    assert_eq!(request.project, Some("core".to_owned()));
    assert_eq!(request.project_filter(), Some(vec!["core".to_owned()]));
}

#[test]
fn asking_for_a_project_outside_your_participation_is_ignored_not_refused() {
    let request = ScopeRequest::from_query(&ctx(&["user"], &["europe-devs"]), None, Some("core"));
    assert_eq!(request.project, None, "the filter is dropped, not honoured");
    assert_eq!(
        request.project_filter(),
        Some(vec!["storefront".to_owned()]),
        "and the caller still sees the projects their marketplaces name"
    );
}

#[test]
fn all_binds_null_and_a_list_binds_itself() {
    assert_eq!(SubjectScope::All.as_sql(), None);
    let ids = vec!["u-1".to_owned(), "u-2".to_owned()];
    assert_eq!(SubjectScope::Users(ids.clone()).as_sql(), Some(&ids[..]));
}

#[test]
fn an_empty_user_scope_matches_nothing_and_does_not_widen() {
    let scope = SubjectScope::Users(Vec::new());
    assert_eq!(
        scope.as_sql(),
        Some(&[][..]),
        "an empty id list is a real answer; binding NULL here would open the listing"
    );
}
