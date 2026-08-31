//! The admin who belongs to no organization — the state every real admin was
//! in.
//!
//! Membership in an organization is what the analytics dashboard and the
//! customer usage report scope through, and nothing outside a boot job for the
//! single configured system admin ever wrote that row. So the deployed console
//! showed its admin a profile page with real usage on it, an analytics page
//! reporting zero requests over the same window, and a sidebar link to a report
//! that answered 404 — three surfaces, one user, three different answers.
//!
//! The suite did not catch it because [`crate::principal`] used to seed the
//! membership itself. It no longer does, so these cases run as a deployed admin
//! actually runs: `admin` role, no membership, and a default organization that
//! `plans.yaml` names.

use axum::http::StatusCode;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

// Every link the sidebar renders for an admin. A link shown to a principal who
// cannot load it is a bug the moment it is drawn, and the sidebar draws
// "Customer usage" for any admin without asking whether they can reach it.
const SIDEBAR_LINKS: &[&str] = &[
    "/admin/analytics",
    "/admin/access/users",
    "/admin/access/departments",
    "/admin/reports/customer",
    "/admin/profile",
    // The conversations page is mounted at /admin/history; there is no
    // /admin/conversations route and nothing links to one.
    "/admin/history",
    "/admin/settings",
];

#[tokio::test]
async fn every_sidebar_link_resolves_for_an_admin_in_no_organization() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    for path in SIDEBAR_LINKS {
        let (status, _) = app.call(Call::get(path, Principal::Admin)).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "the sidebar offers {path} to every admin, so every admin must be \
             able to load it — a link that 404s is a bug at the moment it is drawn"
        );
    }
    db.cleanup().await;
}

#[tokio::test]
async fn the_customer_report_falls_back_to_the_default_organization() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app
        .call(Call::get("/admin/reports/customer", Principal::Admin))
        .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "an admin with no membership must get the default organization's report, \
         not the 404 that told them they were not a member of anything"
    );
    assert!(
        !body.contains("You are not a member of an organization"),
        "and must not be told they are unattached when a default organization exists"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn an_admin_reaches_the_instance_wide_analytics_scope() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app
        .call(Call::get("/admin/analytics", Principal::Admin))
        .await;

    assert_eq!(status, StatusCode::OK);
    // Why: the organization filter is rendered only for a caller whose scope
    // spans more than one organization. Its presence is the page stating that
    // this admin is not pinned to a single org — which is what stopped the
    // KPIs matching an empty slug and reporting zeros.
    assert!(
        body.contains("All organizations"),
        "an admin's analytics scope must span the instance, not be pinned to a \
         membership they do not have"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_non_admin_is_still_refused_the_console() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // Why: widening `admin` to system-wide must not widen anything below it.
    // This is the guard on that change.
    for path in ["/admin/analytics", "/admin/reports/customer"] {
        let (status, _) = app.call(Call::get(path, Principal::NonAdmin)).await;
        assert_ne!(
            status,
            StatusCode::OK,
            "{path} must stay closed to a principal without the admin role"
        );
    }
    db.cleanup().await;
}
