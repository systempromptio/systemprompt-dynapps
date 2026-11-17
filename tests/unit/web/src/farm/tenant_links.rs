//! Link and filter helpers: "all tenants" is the default, so its URLs carry no
//! tenant, and a row without a tenant of its own never invents one.

use systemprompt_web_admin_agent_farm::client::{ALL_TENANTS, is_all_tenants};
use systemprompt_web_admin_agent_farm::views::{
    href, list_href, row_tenant, task_href, tenant_label,
};

#[test]
fn all_tenants_is_nothing_or_the_star() {
    assert!(is_all_tenants(""));
    assert!(is_all_tenants(ALL_TENANTS));
    assert!(!is_all_tenants("lab-b"));
}

#[test]
fn urls_for_all_tenants_stay_clean_and_a_tenant_is_encoded() {
    assert_eq!(
        href("/admin/farm/tasks", &[("tenant", "*")]),
        "/admin/farm/tasks"
    );
    assert_eq!(
        href("/admin/farm/tasks", &[("tenant", "")]),
        "/admin/farm/tasks"
    );
    assert_eq!(
        href(
            "/admin/farm/tasks",
            &[("tenant", "lab b"), ("cursor", "1&2")]
        ),
        "/admin/farm/tasks?tenant=lab%20b&cursor=1%262"
    );
    // Only the tenant gets this treatment: another value of "*" is kept.
    assert_eq!(href("/x", &[("q", "*")]), "/x?q=%2A");
    assert_eq!(list_href("*", Some("159")), "/admin/farm/tasks?cursor=159");
    assert_eq!(list_href("lab-b", None), "/admin/farm/tasks?tenant=lab-b");
}

#[test]
fn a_row_uses_its_own_tenant_else_the_filter_never_a_guess() {
    assert_eq!(row_tenant(Some("lab-b"), "*").as_deref(), Some("lab-b"));
    assert_eq!(
        row_tenant(Some("lab-b"), "dynapps-internal").as_deref(),
        Some("lab-b")
    );
    assert_eq!(row_tenant(None, "lab-b").as_deref(), Some("lab-b"));
    assert_eq!(row_tenant(Some(""), "lab-b").as_deref(), Some("lab-b"));
    assert_eq!(row_tenant(None, "*"), None);
    assert_eq!(tenant_label(None), "—");
    // Without a tenant the detail link carries none and the page falls back
    // to the credential's home tenant.
    assert_eq!(task_href("abc", None), "/admin/farm/tasks/abc");
    assert_eq!(
        task_href("abc", Some("lab-b")),
        "/admin/farm/tasks/abc?tenant=lab-b"
    );
}
