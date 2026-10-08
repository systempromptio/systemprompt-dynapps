//! Wall-clock times render in the viewer's saved timezone.
//!
//! The user-context middleware runs each admin request inside
//! `with_viewer_tz`; these tests pin that every formatter reads that scope,
//! and that anything outside it (or an unknown zone name) renders as UTC.

use chrono::{TimeZone, Utc};
use handlebars::Handlebars;
use systemprompt_web_admin_core::ssr::format::local_time;
use systemprompt_web_admin_core::ssr::viewer_tz::{Tz, parse_tz, viewer_tz, with_viewer_tz};
use systemprompt_web_admin_core::templates::helpers::register_helpers;

fn noon_utc() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 1, 12, 0, 0).unwrap()
}

fn format_date(value: &str) -> String {
    let mut hbs = Handlebars::new();
    register_helpers(&mut hbs);
    hbs.render_template("{{formatDate v}}", &serde_json::json!({ "v": value }))
        .unwrap()
}

#[test]
fn parse_tz_accepts_iana_names() {
    assert_eq!(parse_tz("Europe/Madrid"), Tz::Europe__Madrid);
    assert_eq!(parse_tz(" Asia/Kolkata "), Tz::Asia__Kolkata);
}

#[test]
fn parse_tz_falls_back_to_utc_for_unknown_names() {
    assert_eq!(parse_tz("Foo/Bar"), Tz::UTC);
    assert_eq!(parse_tz(""), Tz::UTC);
}

#[test]
fn outside_a_request_times_render_in_utc() {
    assert_eq!(viewer_tz(), Tz::UTC);
    assert_eq!(local_time(noon_utc()), "2026-03-01 12:00:00");
}

#[tokio::test]
async fn local_time_uses_the_viewer_zone() {
    let madrid = with_viewer_tz(Tz::Europe__Madrid, async { local_time(noon_utc()) }).await;
    assert_eq!(madrid, "2026-03-01 13:00:00");

    let kolkata = with_viewer_tz(Tz::Asia__Kolkata, async { local_time(noon_utc()) }).await;
    assert_eq!(kolkata, "2026-03-01 17:30:00");
}

#[tokio::test]
async fn the_scope_ends_with_the_request() {
    with_viewer_tz(Tz::Europe__Madrid, async {}).await;
    assert_eq!(local_time(noon_utc()), "2026-03-01 12:00:00");
}

#[tokio::test]
async fn format_date_helper_uses_the_viewer_zone() {
    let rendered = with_viewer_tz(Tz::America__New_York, async {
        format_date("2026-03-01T12:00:00Z")
    })
    .await;
    assert_eq!(rendered, "Mar 01, 2026 07:00 AM");
    assert_eq!(format_date("2026-03-01T12:00:00Z"), "Mar 01, 2026 12:00 PM");
}
