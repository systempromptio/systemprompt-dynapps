//! Resolving the window an export asks for, per the dataset's contract.

use chrono::{DateTime, Duration, NaiveTime, Utc};
use serde::Deserialize;

use super::model::{ExportWindow, Window};
use crate::error::{AdminError, AdminResult};
use crate::handlers::ssr::analysis::time;
use crate::util::time_range::{TimeRangeQuery, parse_time_range};

pub(crate) const RETAINED_DAYS: [u32; 5] = [1, 7, 30, 90, 365];
const MAX_RETAINED_DAYS: i64 = 366;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct WindowQuery {
    preset: Option<String>,
    from: Option<String>,
    to: Option<String>,
    days: Option<u32>,
    start: Option<String>,
    end: Option<String>,
}

pub(crate) fn resolve(kind: Window, query: &WindowQuery) -> AdminResult<Option<ExportWindow>> {
    match kind {
        Window::None => Ok(None),
        Window::Live => Ok(Some(live(query))),
        Window::Days => days_only(query).map(Some),
        Window::Retained => retained(query).map(Some),
    }
}

// Why: the dialog's `datetime-local` controls send `%Y-%m-%dT%H:%M`, which
// the live parser (RFC 3339 only) would reject as a lone bound; every
// accepted spelling is normalised to RFC 3339 first.
fn live(query: &WindowQuery) -> ExportWindow {
    let rfc3339 = |v: &Option<String>| v.as_deref().and_then(time::parse).map(|t| t.to_rfc3339());
    let range = parse_time_range(&TimeRangeQuery {
        from: rfc3339(&query.from),
        to: rfc3339(&query.to),
        preset: query.preset.clone(),
    });
    ExportWindow {
        from: range.from,
        to: range.to,
        days: u32::try_from((range.to - range.from).num_days()).unwrap_or(0),
    }
}

// Why: the snapshot pipeline buckets facts under UTC days and the current day
// is still filling, so a day window ends at the next midnight — the same
// boundary the analysis pages use.
fn next_midnight() -> DateTime<Utc> {
    (Utc::now().date_naive() + Duration::days(1))
        .and_time(NaiveTime::MIN)
        .and_utc()
}

fn days_only(query: &WindowQuery) -> AdminResult<ExportWindow> {
    let days = query.days.unwrap_or(30);
    if !RETAINED_DAYS.contains(&days) {
        return Err(AdminError::BadRequest("Unsupported window".to_owned()));
    }
    let to = next_midnight();
    Ok(ExportWindow {
        from: to - Duration::days(i64::from(days)),
        to,
        days,
    })
}

fn retained(query: &WindowQuery) -> AdminResult<ExportWindow> {
    let start = query.start.as_deref().and_then(time::parse);
    let end = query.end.as_deref().and_then(time::parse);
    if start.is_none() && end.is_none() {
        return days_only(query);
    }
    let to = end.unwrap_or_else(next_midnight);
    let from = start.unwrap_or(to - Duration::days(30));
    if from >= to || to - from > Duration::days(MAX_RETAINED_DAYS) {
        return Err(AdminError::BadRequest(
            "Choose a window between 1 day and 366 days".to_owned(),
        ));
    }
    Ok(ExportWindow {
        from,
        to,
        days: u32::try_from((to - from).num_days()).unwrap_or(0),
    })
}
