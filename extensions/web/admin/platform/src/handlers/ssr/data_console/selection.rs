//! Turning a Logs or SQL browse query into a validated [`Selection`], and a
//! selection back into the page URL that reproduces it.

use chrono::{DateTime, Duration, Utc};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

use super::BrowseQuery;
use crate::repositories::data_console::browse::{Filter, Selection};

fn date(raw: Option<&str>) -> AdminResult<Option<DateTime<Utc>>> {
    raw.filter(|s| !s.is_empty()).map(|s| DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&Utc))
        .map_err(|_error| AdminError::BadRequest("Use an ISO timestamp including its timezone, for example 2026-10-04T02:38:26Z.".into()))).transpose()
}
pub(super) fn selection(query: &BrowseQuery, logs: bool) -> AdminResult<Selection> {
    let mut s = Selection {
        table: if logs {
            "public.logs".into()
        } else {
            query.table.clone().unwrap_or_default()
        },
        time_column: query.time_column.clone().filter(|s| !s.is_empty()),
        from: date(query.from.as_deref())?,
        until: date(query.until.as_deref())?,
        filters: Vec::new(),
    };
    if logs {
        let until = s.until.unwrap_or_else(Utc::now);
        s.until = Some(until);
        let from = s.from.unwrap_or(until - Duration::hours(1));
        s.from = Some(from);
        s.time_column = Some("timestamp".into());
        if until - from > Duration::days(7) || from > until {
            return Err(AdminError::BadRequest(
                "Log browsing ranges must be between zero and seven days.".into(),
            ));
        }
        for (column, value) in [
            ("level", &query.level),
            ("module", &query.module),
            ("context_id", &query.context_filter),
            ("session_id", &query.session_filter),
            ("trace_id", &query.trace_id),
        ] {
            if let Some(value) = value.as_ref().filter(|s| !s.is_empty()) {
                s.filters.push(Filter {
                    column: column.into(),
                    value: value.clone(),
                });
            }
        }
        if query.level.as_ref().is_some_and(|l| {
            !l.is_empty() && !["ERROR", "WARN", "INFO", "DEBUG", "TRACE"].contains(&l.as_str())
        }) {
            return Err(AdminError::BadRequest("Select a valid log level.".into()));
        }
    } else if let (Some(column), Some(value)) = (&query.column, &query.value)
        && !column.is_empty()
    {
        s.filters.push(Filter {
            column: column.clone(),
            value: value.clone(),
        });
    }
    Ok(s)
}
pub(super) fn url(selection: &Selection, logs: bool, cursor: Option<&str>) -> String {
    let mut pairs = vec![("table", selection.table.clone())];
    if let Some(from) = selection.from {
        pairs.push(("from", from.to_rfc3339()));
    }
    if let Some(until) = selection.until {
        pairs.push(("until", until.to_rfc3339()));
    }
    if let Some(column) = &selection.time_column {
        pairs.push(("time_column", column.clone()));
    }
    for f in &selection.filters {
        if logs {
            pairs.push((f.column.as_str(), f.value.clone()));
        } else {
            pairs.push(("column", f.column.clone()));
            pairs.push(("value", f.value.clone()));
        }
    }
    if let Some(cursor) = cursor {
        pairs.push(("cursor", cursor.into()));
    }
    let query = pairs
        .iter()
        .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    format!("/admin/{}?{query}", if logs { "logs" } else { "sql" })
}
