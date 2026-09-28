//! `GET /admin/api/overview` — the overview body as a fragment.
//!
//! The live layer on `/admin` morphs this fragment over the page, so what
//! matters is that it is the body and only the body: the same partial the page
//! embeds, with the hooks the client keys on, and never a second page shell
//! wrapped around it. A non-console viewer is refused outright, because a
//! fetch that followed the page's redirect would morph profile HTML into the
//! overview.

use axum::http::StatusCode;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

const CHARTS_ON_THE_OVERVIEW: usize = 6;

#[tokio::test(flavor = "multi_thread")]
async fn the_fragment_is_the_page_body_for_every_window() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping the overview fragment suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let mut failures = Vec::new();
    for (query, preset) in [
        ("", "24h"),
        ("?preset=7d", "7d"),
        ("?preset=30d", "30d"),
        ("?preset=junk", "24h"),
    ] {
        let path = format!("/admin/api/overview{query}");
        let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
        if status != StatusCode::OK {
            failures.push(format!("  {path} -> {} (expected 200)", status.as_u16()));
            continue;
        }
        if body.trim_start().starts_with("<!DOCTYPE") || body.contains("<html") {
            failures.push(format!(
                "  {path} rendered a whole page around the fragment"
            ));
        }
        if !body.contains(&format!("data-preset=\"{preset}\"")) {
            failures.push(format!("  {path} did not stamp data-preset=\"{preset}\""));
        }
        let charts = body.matches("data-chart ").count();
        if charts != CHARTS_ON_THE_OVERVIEW {
            failures.push(format!(
                "  {path} carried {charts} charts, expected {CHARTS_ON_THE_OVERVIEW}"
            ));
        }
        for hook in [
            "data-overview-live",
            "data-kpi=\"requests\"",
            "data-live-region=\"kpis\"",
        ] {
            if !body.contains(hook) {
                failures.push(format!(
                    "  {path} lacks the {hook} hook the live layer keys on"
                ));
            }
        }
    }

    let (status, body) = app
        .call(Call::get("/admin?preset=7d", Principal::Admin))
        .await;
    if status != StatusCode::OK {
        failures.push(format!(
            "  /admin?preset=7d -> {} (expected 200)",
            status.as_u16()
        ));
    }
    if body.matches("data-overview-live").count() != 1 {
        failures.push("  /admin embeds the live body other than exactly once".to_owned());
    }
    if !body.contains("data-live-status") {
        failures.push("  /admin lacks the freshness indicator".to_owned());
    }

    let (status, _) = app
        .call(Call::get("/admin/api/overview", Principal::NonAdmin))
        .await;
    if status != StatusCode::FORBIDDEN {
        failures.push(format!(
            "  non-console viewer -> {} (expected 403)",
            status.as_u16()
        ));
    }

    assert!(failures.is_empty(), "\n{}", failures.join("\n"));

    db.cleanup().await;
}
