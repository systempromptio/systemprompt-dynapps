//! The viewer's timezone for one request.
//!
//! A user saves an IANA zone on their Profile (`user_settings.timezone`). The
//! user-context middleware reads it once and runs the handler inside
//! [`with_viewer_tz`], so every formatter that renders a wall-clock time —
//! [`super::format::local_time`], the chart bucket labels, the `formatDate`
//! template helper — converts through [`in_viewer_tz`] without the zone being
//! threaded through each call. Outside that scope (jobs, exports, tests) the
//! zone is UTC, and so is a saved name chrono-tz does not know: the profile
//! form checks only the shape of the name.

use std::future::Future;

use chrono::{DateTime, TimeZone};
pub use chrono_tz::Tz;

tokio::task_local! {
    static VIEWER_TZ: Tz;
}

pub fn parse_tz(name: &str) -> Tz {
    name.trim().parse::<Tz>().unwrap_or(Tz::UTC)
}

pub async fn with_viewer_tz<F: Future>(tz: Tz, f: F) -> F::Output {
    VIEWER_TZ.scope(tz, f).await
}

pub fn viewer_tz() -> Tz {
    VIEWER_TZ.try_with(|tz| *tz).unwrap_or(Tz::UTC)
}

pub fn in_viewer_tz<Z: TimeZone>(t: &DateTime<Z>) -> DateTime<Tz> {
    t.with_timezone(&viewer_tz())
}
