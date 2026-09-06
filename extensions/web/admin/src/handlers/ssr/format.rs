//! Display formatting shared by the admin entity pages.
//!
//! The pure value formatters live in `systemprompt_web_shared::format` (so
//! the test workspace can exercise them); this module re-exports them for
//! the ssr pages and keeps the admin-specific helpers.
//!
//! Ids, costs, token counts and durations are rendered in a handful of places
//! across the entity list and detail pages; the rules live here so a cost reads
//! the same on the sessions list as it does on the session it links to.

pub(crate) use systemprompt_web_shared::format::{
    format_cost, format_duration_ms, short_id, short_num,
};

pub(crate) fn format_token_total(total: i64) -> String {
    if total <= 0 {
        return "—".to_owned();
    }
    short_num(total)
}

// Why: A timestamp in the viewer's local zone, to the second.
pub(crate) fn local_time(t: chrono::DateTime<chrono::Utc>) -> String {
    t.with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

// Why: How long a first-to-last window lasted, or an em dash when either end is
// missing — a hook session can arrive with no timestamp at all.
pub(crate) fn format_span(
    start: Option<chrono::DateTime<chrono::Utc>>,
    end: Option<chrono::DateTime<chrono::Utc>>,
) -> String {
    match (start, end) {
        (Some(a), Some(b)) => format_duration_ms((b - a).num_milliseconds().max(0)),
        _ => "\u{2014}".to_owned(),
    }
}
