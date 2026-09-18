//! `/admin/gateway` — the only editing surface in the platform section.
//!
//! The page reads two independent sources: the `gateway.routes:` sequence in
//! `services/ai/gateway.yaml`, which the editor addresses by index, and the
//! resolved route set the dispatcher will actually match. A page that rendered
//! one while claiming to show the other would let an operator reorder a list
//! that is not the list being dispatched, so both halves are asserted here —
//! the Routes tab for the file, the Resolved tab for the dispatcher — along
//! with the Models tab the page opens on, which groups the same routes by
//! provider.

use axum::http::StatusCode;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

#[tokio::test(flavor = "multi_thread")]
async fn the_gateway_page_renders_the_routing_table() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping gateway page suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let mut failures = Vec::new();
    let (status, body) = app
        .call(Call::get("/admin/gateway", Principal::Admin))
        .await;
    if status != StatusCode::OK {
        failures.push(format!(
            "  /admin/gateway -> {} (expected 200): {}",
            status.as_u16(),
            body.chars().take(200).collect::<String>()
        ));
    }

    // Why: the default view is the picture by provider, and the settings
    // form is the only place the enabled flag can be changed from.
    for marker in ["sp-p-gateway__provider", "gateway-settings"] {
        if !body.contains(marker) {
            failures.push(format!("  /admin/gateway rendered without {marker:?}"));
        }
    }

    // Why: the dispatch-order note is what makes the ordinal column mean
    // something; it lives on the Routes tab with the reorder controls.
    let (routes_status, routes_body) = app
        .call(Call::get("/admin/gateway?tab=routes", Principal::Admin))
        .await;
    if routes_status != StatusCode::OK
        || !routes_body.contains("the order the dispatcher tries them")
    {
        failures.push(format!(
            "  /admin/gateway?tab=routes -> {} without the dispatch-order table",
            routes_status.as_u16()
        ));
    }

    // Why: the resolved-only section is the half an operator cannot see in
    // the file. It sits behind its own tab so the routing table keeps the
    // fold, and the tab has to render it.
    let (resolved_status, resolved_body) = app
        .call(Call::get("/admin/gateway?tab=resolved", Principal::Admin))
        .await;
    if resolved_status != StatusCode::OK || !resolved_body.contains("Resolved but not declared") {
        failures.push(format!(
            "  /admin/gateway?tab=resolved -> {} without the resolved-only section",
            resolved_status.as_u16()
        ));
    }

    // Why: the editor addresses routes by their index in the YAML sequence, so
    // a row without one is a row whose edit and delete buttons point nowhere.
    if routes_body.contains("data-route-index=\"0\"") == routes_body.contains("No routes") {
        failures.push(
            "  /admin/gateway?tab=routes showed neither an indexed route row nor the empty state"
                .to_owned(),
        );
    }

    assert!(
        failures.is_empty(),
        "gateway page contract:\n{}",
        failures.join("\n")
    );
}
