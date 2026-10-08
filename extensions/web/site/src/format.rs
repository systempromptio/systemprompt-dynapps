//! Shared display formatting for public-site providers.

use chrono::{DateTime, Utc};

#[doc(hidden)]
pub fn format_date(raw: &str) -> Option<String> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        return Some(dt.format("%B %d, %Y").to_string());
    }
    raw.parse::<DateTime<Utc>>()
        .ok()
        .map(|dt| dt.format("%B %d, %Y").to_string())
}
