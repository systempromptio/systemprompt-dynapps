//! `util::project_scope::ProjectScope` — who may see which project. Pure, so
//! the whole rule is pinned here: an admin chooses, everyone else is pinned to
//! their own project, and having no project means seeing nothing.

use systemprompt::identifiers::{Email, UserId};
use systemprompt_web_admin::UserContext;
use systemprompt_web_admin::types::Project;
use systemprompt_web_admin::util::project_scope::ProjectScope;

fn ctx(is_admin: bool, project: Option<Project>) -> UserContext {
    UserContext {
        user_id: UserId::new("u-1".to_owned()),
        username: "u-1".to_owned(),
        email: Email::try_new("u-1@example.test".to_owned()).expect("fixture email"),
        project,
        roles: if is_admin {
            vec!["admin".to_owned(), "user".to_owned()]
        } else {
            vec!["user".to_owned()]
        },
        is_admin,
        email_verified: true,
        session_id: None,
    }
}

#[test]
fn an_admin_with_no_param_sees_all() {
    let scope = ProjectScope::resolve(&ctx(true, Some(Project::Core)), None);
    assert_eq!(scope, ProjectScope::All);
    assert_eq!(scope.as_sql(), None);
    assert_eq!(scope.selected(), None);
}

#[test]
fn an_admin_may_pick_a_project() {
    let scope = ProjectScope::resolve(&ctx(true, Some(Project::Core)), Some("commerce"));
    assert_eq!(scope, ProjectScope::Only(Project::Commerce));
    assert_eq!(scope.as_sql(), Some("commerce"));
    assert_eq!(scope.selected(), Some(Project::Commerce));
}

#[test]
fn an_admin_picking_an_unknown_project_sees_all() {
    assert_eq!(
        ProjectScope::resolve(&ctx(true, None), Some("bogus")),
        ProjectScope::All
    );
    assert_eq!(
        ProjectScope::resolve(&ctx(true, None), Some("")),
        ProjectScope::All
    );
}

#[test]
fn a_user_is_locked_to_their_own_project_whatever_they_ask_for() {
    let user = ctx(false, Some(Project::Core));
    assert_eq!(
        ProjectScope::resolve(&user, Some("commerce")),
        ProjectScope::Only(Project::Core)
    );
    assert_eq!(
        ProjectScope::resolve(&user, None),
        ProjectScope::Only(Project::Core)
    );
}

#[test]
fn a_user_with_no_project_sees_nothing() {
    assert_eq!(
        ProjectScope::resolve(&ctx(false, None), Some("commerce")),
        ProjectScope::Nothing
    );
}

#[test]
fn nothing_binds_the_empty_string() {
    assert_eq!(ProjectScope::Nothing.as_sql(), Some(""));
    assert_eq!(ProjectScope::Nothing.selected(), None);
}
