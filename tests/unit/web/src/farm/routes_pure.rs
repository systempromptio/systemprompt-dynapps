//! Pure halves of the Farm console's `routes` module.

use systemprompt_web_admin_agent_farm::routes::target;

#[test]
fn the_new_location_keeps_path_and_query_and_stays_under_the_new_root() {
    assert_eq!(target(None, None), "/admin/farm");
    assert_eq!(
        target(None, Some("tenant=lab-b")),
        "/admin/farm?tenant=lab-b"
    );
    assert_eq!(
        target(Some("tasks/01a1-2b"), Some("tenant=lab-b&cursor=159")),
        "/admin/farm/tasks/01a1-2b?tenant=lab-b&cursor=159"
    );
    // Nothing a visitor sends can point the redirect elsewhere.
    assert_eq!(
        target(Some("//evil.example"), None),
        "/admin/farm/evil.example"
    );
    assert_eq!(target(Some("../x/../y"), None), "/admin/farm/x/y");
    assert!(target(Some("a/b"), Some("")).ends_with("/a/b"));
}
